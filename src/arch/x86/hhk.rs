use crate::arch::x86::multiboot::{MultibootInfo, present, MULTIBOOT_INFO_DRIVE_INFO};
use crate::kernel::mm::page::{
    PG_MAX_ENTRIES, PG_PRESENT, PG_PREM_RW, T_SELF_REF_PERM,
    pd_index, pt_index, pde, pte, v2p, PtdT,
};
use crate::kernel::common::HIGHER_HLF_BASE;
use core::mem;

const PT_TABLE_IDENTITY: usize = 0;       // 使用表 #1
const PT_TABLE_KERNEL: usize = 1;         // 使用表 #2-4 (因此內核最大大小為8MiB)
const PT_TABLE_STACK: usize = 4;          // 使用表 #5

// 這些變數由連結器提供 (見 linker.ld)
extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
    static __init_hhk_end: u8;
    static _k_stack: u8;
}

// 獲取符號值的輔助函數
#[inline]
fn sym_val(sym: &u8) -> usize {
    sym as *const u8 as usize
}

// 計算內核頁數
fn kernel_page_count() -> usize {
    unsafe {
        (sym_val(&__kernel_end) - sym_val(&__kernel_start) + 0x1000 - 1) >> 12
    }
}

// 計算HHK頁數
fn hhk_page_count() -> usize {
    unsafe {
        (sym_val(&__init_hhk_end) - 0x100000 + 0x1000 - 1) >> 12
    }
}

// 計算對應到頁表的地址
fn pt_addr(ptd: *mut PtdT, pt_index: usize) -> *mut PtdT {
    unsafe {
        ptd.add((pt_index + 1) * 1024)
    }
}

// 設置頁目錄條目
fn set_pde(ptd: *mut PtdT, pde_index: usize, pde_val: u32) {
    unsafe {
        *ptd.add(pde_index) = pde_val;
    }
}

// 設置頁表條目
fn set_pte(ptd: *mut PtdT, pt_index: usize, pte_index: usize, pte_val: u32) {
    unsafe {
        *pt_addr(ptd, pt_index).add(pte_index) = pte_val;
    }
}

/// 初始化頁面
#[no_mangle]
pub extern "C" fn _init_page(ptd: *mut PtdT) {
    // 設置第一個頁目錄條目，指向第一個頁表
    unsafe {
        set_pde(ptd, 0, pde(PG_PRESENT, ptd.add(PG_MAX_ENTRIES as usize) as u32));
        
        // 對低1MiB空間進行對等映射（Identity mapping），也包括VGA，方便內核操作
        for i in 0..256 {
            set_pte(ptd, PT_TABLE_IDENTITY, i, pte(PG_PREM_RW, (i << 12) as u32));
        }
        
        // 對等映射hhk_init，這樣一來，當分頁與地址轉換開啟後，
        // 依然能夠照常執行最終的 jmp 指令來跳轉至內核的入口點
        let hhk_pages = hhk_page_count();
        for i in 0..hhk_pages {
            set_pte(
                ptd, 
                PT_TABLE_IDENTITY, 
                256 + i, 
                pte(PG_PREM_RW, (0x100000 + (i << 12)) as u32)
            );
        }
        
        // --- 將內核重映射至高半區 ---
        
        // 計算應當映射進的頁目錄與頁表的條目索引
        let kernel_pde_index = pd_index(sym_val(&__kernel_start));
        let kernel_pte_index = pt_index(sym_val(&__kernel_start));
        let kernel_pg_counts = kernel_page_count();
        
        // 將內核所需要的頁表註冊進頁目錄
        // 分配了3個頁表（12MiB），未雨綢繆
        for i in 0..(PT_TABLE_STACK - PT_TABLE_KERNEL) {
            set_pde(
                ptd,
                kernel_pde_index + i,
                pde(PG_PREM_RW, pt_addr(ptd, PT_TABLE_KERNEL + i) as u32)
            );
        }
        
        // 首先，檢查內核的大小是否可以fit進我們這幾個表（12MiB）
        if kernel_pg_counts > (PT_TABLE_STACK - PT_TABLE_KERNEL) * 1024 {
            // 錯誤：需要更多頁
            // 這裡應該做其他事情而不是進入阻塞
            loop {}
        }
        
        // 計算內核.text段的物理地址
        let kernel_pm = v2p(sym_val(&__kernel_start)) as u32;
        
        // 重映射內核至高半區地址（>=0xC0000000）
        for i in 0..kernel_pg_counts {
            set_pte(
                ptd,
                PT_TABLE_KERNEL,
                kernel_pte_index + i,
                pte(PG_PREM_RW, kernel_pm + (i << 12) as u32)
            );
        }
        
        // 最後一個entry用於循環映射
        set_pde(
            ptd,
            1023,
            pde(T_SELF_REF_PERM, ptd as u32)
        );
    }
}

/// 保存子集數據
#[no_mangle]
pub extern "C" fn _save_subset(destination: *mut u8, base: *const u8, size: usize) -> usize {
    for i in 0..size {
        unsafe {
            *destination.add(i) = *base.add(i);
        }
    }
    size
}

/// 保存multiboot資訊
#[no_mangle]
pub extern "C" fn _save_multiboot_info(info: *const MultibootInfo, destination: *mut u8) {
    unsafe {
        let info_ptr = info as *const u8;
        let info_size = mem::size_of::<MultibootInfo>();
        
        // 複製主結構
        for i in 0..info_size {
            *destination.add(i) = *info_ptr.add(i);
        }
        
        let mut current = info_size;
        
        // 保存memory map信息
        let mmap_dest_addr = destination.add(current) as usize;
        (*(destination as *mut MultibootInfo)).mmap_addr = mmap_dest_addr as u32;
        
        current += _save_subset(
            destination.add(current),
            (*info).mmap_addr as *const u8,
            (*info).mmap_length as usize
        );
        
        // 如果有驅動器信息，也保存
        if present((*info).flags, MULTIBOOT_INFO_DRIVE_INFO) {
            let drives_dest_addr = destination.add(current) as usize;
            (*(destination as *mut MultibootInfo)).drives_addr = drives_dest_addr as u32;
            
            current += _save_subset(
                destination.add(current),
                (*info).drives_addr as *const u8,
                (*info).drives_length as usize
            );
        }
    }
}

/// 初始化HHK
#[no_mangle]
pub extern "C" fn _hhk_init(ptd: *mut PtdT, kpg_size: u32) {
    unsafe {
        // 初始化 kpg 全為0
        // P.s. 真沒想到GRUB會在這裡留下一堆垃圾！ 頁表全亂套了！
        let kpg = ptd as *mut u8;
        for i in 0..kpg_size as usize {
            *kpg.add(i) = 0;
        }
        
        _init_page(ptd);
    }
}
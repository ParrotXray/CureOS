// src/hal/cpu.rs

use core::arch::asm;

/// 32 位元暫存器類型
pub type Reg32 = u32;
/// 16 位元暫存器類型
pub type Reg16 = u16;

/// 通用目的暫存器結構
#[repr(C, packed)]
pub struct GpRegs {
    pub eax: Reg32,
    pub ebx: Reg32,
    pub ecx: Reg32,
    pub edx: Reg32,
    pub edi: Reg32,
    pub ebp: Reg32,
    pub esi: Reg32,
    pub esp: Reg32,
}

/// 段暫存器結構
#[repr(C, packed)]
pub struct SgReg {
    pub ss: Reg16,
    pub es: Reg16,
    pub ds: Reg16,
    pub fs: Reg16,
    pub gs: Reg16,
    pub cs: Reg16,
}

/// 讀取 CR0 暫存器
#[inline]
pub fn cpu_r_cr0() -> Reg32 {
    let val: Reg32;
    unsafe {
        asm!("mov %cr0, {0}", out(reg) val);
    }
    val
}

/// 讀取 CR2 暫存器
#[inline]
pub fn cpu_r_cr2() -> Reg32 {
    let val: Reg32;
    unsafe {
        asm!("mov %cr2, {0}", out(reg) val);
    }
    val
}

/// 讀取 CR3 暫存器
#[inline]
pub fn cpu_r_cr3() -> Reg32 {
    let val: Reg32;
    unsafe {
        asm!("mov %cr3, {0}", out(reg) val);
    }
    val
}

/// 寫入 CR0 暫存器
#[inline]
pub fn cpu_w_cr0(val: Reg32) {
    unsafe {
        asm!("mov {0}, %cr0", in(reg) val);
    }
}

/// 寫入 CR2 暫存器
#[inline]
pub fn cpu_w_cr2(val: Reg32) {
    unsafe {
        asm!("mov {0}, %cr2", in(reg) val);
    }
}

/// 寫入 CR3 暫存器
#[inline]
pub fn cpu_w_cr3(val: Reg32) {
    unsafe {
        asm!("mov {0}, %cr3", in(reg) val);
    }
}

/// 獲取 CPU 型號
/// 
/// # 參數
/// * `model_out` - 輸出緩衝區，至少需要 13 bytes
pub fn cpu_get_model(model_out: &mut [u8]) {
    let out: &mut [u32] = unsafe {
        core::slice::from_raw_parts_mut(model_out.as_mut_ptr() as *mut u32, 3)
    };
    
    let mut eax: u32 = 0;
    let mut ebx: u32;
    let mut ecx: u32;  
    let mut edx: u32;
    
    unsafe {
        asm!("cpuid",
            inout("eax") eax,
            out("ebx") ebx,
            out("ecx") ecx,
            out("edx") edx
        );
    }
    
    out[0] = ebx;
    out[1] = edx;
    out[2] = ecx;
    
    let _max_function_id: u32 = eax;

    // 確保以 null 結尾
    if model_out.len() > 12 {
        model_out[12] = 0;
    }
}

#[allow(dead_code)]
const BRAND_LEAF: u32 = 0x80000000;

/// 檢查是否支持品牌字串
pub fn cpu_brand_string_supported() -> bool {
    let mut eax: u32 = BRAND_LEAF;
    let mut ebx: u32;
    let mut ecx: u32;
    let mut edx: u32;
    
    unsafe {
        asm!("cpuid",
            inout("eax") eax,
            out("ebx") ebx,
            out("ecx") ecx,
            out("edx") edx
        );
    }
    
    // eax >= 0x80000004
    eax >= BRAND_LEAF + 4
}

/// 獲取 CPU 品牌字串
/// 
/// # 參數
/// * `brand_out` - 輸出緩衝區，至少需要 49 bytes
pub fn cpu_get_brand(brand_out: &mut [u8]) {
    if !cpu_brand_string_supported() {
        brand_out[0] = b'?';
        if brand_out.len() > 1 {
            brand_out[1] = 0;
        }
        return;
    }
    
    let out: &mut [u32] = unsafe {
        core::slice::from_raw_parts_mut(brand_out.as_mut_ptr() as *mut u32, 12)
    };
    
    for i in 0..3u32 {
        let mut eax: u32 = BRAND_LEAF + i + 2;
        let mut ebx: u32;
        let mut ecx: u32;
        let mut edx: u32;
        
        unsafe {
            asm!("cpuid",
                inout("eax") eax,
                out("ebx") ebx,
                out("ecx") ecx,
                out("edx") edx
            );
        }
        
        let j = (i * 4) as usize;
        out[j] = eax;
        out[j + 1] = ebx;
        out[j + 2] = ecx;
        out[j + 3] = edx;
    }
    
    // 確保以 null 結尾
    if brand_out.len() > 48 {
        brand_out[48] = 0;
    }
}

/// 讀取 CPU 時間戳計數器 (TSC)
/// 
/// # 返回
/// 時間戳計數值
#[inline]
pub fn cpu_rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdtsc", out("eax") low, out("edx") high);
    }
    ((high as u64) << 32) | (low as u64)
}

/// 執行 CPU 暫停指令
/// 
/// 在等待時減少 CPU 功耗
#[inline]
pub fn cpu_pause() {
    unsafe {
        asm!("pause");
    }
}

/// 停止 CPU 執行，直到下一個中斷發生
#[inline]
pub fn cpu_halt() {
    unsafe {
        asm!("hlt");
    }
}

/// 停止 CPU 並進入低功耗模式
#[inline]
pub fn cpu_idle() {
    unsafe {
        asm!("hlt");
    }
}

/// 啟用中斷
#[inline]
pub fn cpu_enable_interrupts() {
    unsafe {
        asm!("sti");
    }
}

/// 禁用中斷
#[inline]
pub fn cpu_disable_interrupts() {
    unsafe {
        asm!("cli");
    }
}
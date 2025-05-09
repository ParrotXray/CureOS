// src/hal/cpu.rs
use x86::controlregs::{
    cr0, 
    cr0_write, 
    cr2, 
    cr2_write, 
    cr3, 
    cr3_write, 
    cr4, 
    cr4_write, 
    Cr0, 
    Cr4
};
use x86::time::rdtsc;
use x86::cpuid;
use core::hint::spin_loop;
use x86::irq::{enable, disable};
use x86::halt;
use core::arch::asm;

/// 32 位元暫存器類型
#[allow(dead_code)]
pub type Reg32 = u32;
/// 16 位元暫存器類型
#[allow(dead_code)]
pub type Reg16 = u16;

/// 通用目的暫存器結構
#[allow(dead_code)]
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
#[allow(dead_code)]
#[repr(C, packed)]
pub struct SgReg {
    pub ss: Reg16,
    pub es: Reg16,
    pub ds: Reg16,
    pub fs: Reg16,
    pub gs: Reg16,
    pub cs: Reg16,
}

/// Write cr4.
///
/// # Example
///
/// ```no_run
/// use x86::controlregs::*;
/// unsafe {
///   let cr4 = cr4();
///   let cr4 = cr4 | Cr4::CR4_ENABLE_PSE;
///   cr4_write(cr4);
/// }
/// ```

/// 讀取 CR0 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr0() -> Cr0 {
    unsafe { cr0() }
}

/// 讀取 CR2 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr2() -> usize {
    unsafe { cr2() }
}

/// 讀取 CR3 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr3() -> u64 {
    unsafe { cr3() }
}

/// 讀取 CR4 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr4() -> Cr4 {
    unsafe { cr4() }
}

/// 寫入 CR0 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr0(val: Cr0) {
    unsafe { cr0_write(val); }
}

/// 寫入 CR2 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr2(val: u64) {
    unsafe { cr2_write(val); }
}

/// 寫入 CR3 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr3(val: u64) {
    unsafe { cr3_write(val); }
}

/// 寫入 CR4 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr4(val: Cr4) {
    unsafe { cr4_write(val); }
}

/// 獲取 CPU 型號
/// 
/// # 參數
/// * `model_out` - 輸出緩衝區，至少需要 13 bytes
/// # 返回
/// 字符串切片，表示 CPU 供應商資訊
#[allow(dead_code)]
pub fn cpu_get_model(model_out: &mut [u8]) -> &str {
    if model_out.len() < 13 {
        return "Buffer too small";
    }
    
    // 使用 cpuid 取得廠商 ID
    let vendor_id = cpuid::CpuId::new().get_vendor_info();
    
    if let Some(vendor) = vendor_id {
        let vendor_string = vendor.as_str();
        let bytes = vendor_string.as_bytes();
        let copy_len = core::cmp::min(bytes.len(), model_out.len() - 1);
        
        model_out[..copy_len].copy_from_slice(&bytes[..copy_len]);
        model_out[copy_len] = 0; // null

        match core::str::from_utf8(&model_out[..copy_len]) {
            Ok(s) => s,
            Err(_) => "Invalid UTF-8"
        }
    } else {
        model_out[0] = b'?';
        model_out[1] = 0;
        "Unknown Vendor"
    }
}

/// 檢查是否支持品牌字串
#[allow(dead_code)]
pub fn cpu_brand_string_supported() -> bool {
    let cpuid = cpuid::CpuId::new();
    cpuid.get_processor_brand_string().is_some()
}

/// 獲取 CPU 品牌字串
/// 
/// # 參數
/// * `brand_out` - 輸出緩衝區，至少需要 49 bytes
/// 
/// # 返回
/// 字符串切片，表示 CPU 品牌
#[allow(dead_code)]
pub fn cpu_get_brand(brand_out: &mut [u8]) -> &str {
    if brand_out.len() < 49 {
        return "Buffer too small";
    }
    
    let cpuid = cpuid::CpuId::new();
    
    if let Some(brand) = cpuid.get_processor_brand_string() {
        let brand_string = brand.as_str();
        let bytes = brand_string.as_bytes();
        let copy_len = core::cmp::min(bytes.len(), brand_out.len() - 1);
        
        brand_out[..copy_len].copy_from_slice(&bytes[..copy_len]);
        brand_out[copy_len] = 0; // null
        
        match core::str::from_utf8(&brand_out[..copy_len]) {
            Ok(s) => s,
            Err(_) => "Invalid UTF-8"
        }
    } else {
        brand_out[0] = b'?';
        brand_out[1] = 0;
        "Unknown CPU"
    }
}

/// 讀取 CPU 時間戳計數器 (TSC)
/// 
/// # 返回
/// 時間戳計數值
#[allow(dead_code)]
#[inline]
pub fn cpu_rdtsc() -> u64 {
    unsafe { rdtsc() }
}

/// 執行 CPU 暫停指令
/// 
/// 在等待時減少 CPU 功耗
#[allow(dead_code)]
#[inline]
pub fn cpu_pause() {
    spin_loop();
}

/// 停止 CPU 執行，直到下一個中斷發生
#[allow(dead_code)]
#[inline]
pub fn cpu_halt() {
    unsafe { halt(); }
}

/// 停止 CPU 並進入低功耗模式
#[allow(dead_code)]
#[inline]
pub fn cpu_idle() {
unsafe { halt(); }
}

/// 啟用中斷
#[allow(dead_code)]
#[inline]
pub fn cpu_enable_interrupts() {
    unsafe {
        enable();
    }
}

/// 禁用中斷
#[allow(dead_code)]
#[inline]
pub fn cpu_disable_interrupts() {
    unsafe {
        disable();
    }
}
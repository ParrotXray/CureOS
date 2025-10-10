// kernel/src/hal/cpu.rs
use x86_64::registers::control::{Cr0, Cr0Flags, Cr2, Cr3, Cr4, Cr4Flags};
use x86_64::instructions::{interrupts, hlt};
use core::arch::asm;
use x86_64::VirtAddr;

/// 64 位元暫存器類型
#[allow(dead_code)]
pub type Reg64 = u64;
/// 32 位元暫存器類型
#[allow(dead_code)]
pub type Reg32 = u32;
/// 16 位元暫存器類型
#[allow(dead_code)]
pub type Reg16 = u16;

/// 通用目的暫存器結構 (64-bit)
#[allow(dead_code)]
#[repr(C, packed)]
pub struct GpRegs {
    pub rax: Reg64,
    pub rbx: Reg64,
    pub rcx: Reg64,
    pub rdx: Reg64,
    pub rdi: Reg64,
    pub rbp: Reg64,
    pub rsi: Reg64,
    pub rsp: Reg64,
    pub r8: Reg64,
    pub r9: Reg64,
    pub r10: Reg64,
    pub r11: Reg64,
    pub r12: Reg64,
    pub r13: Reg64,
    pub r14: Reg64,
    pub r15: Reg64,
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

// ============ 控制寄存器 ============

/// 讀取 CR0 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr0() -> Cr0Flags {
    Cr0::read()
}

/// 讀取 CR2 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr2() -> u64 {
    Cr2::read_raw()
}

/// 讀取 CR3 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr3() -> u64 {
    Cr3::read().0.start_address().as_u64()
}

/// 讀取 CR4 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr4() -> Cr4Flags {
    Cr4::read()
}

/// 寫入 CR0 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr0(val: Cr0Flags) {
    unsafe { Cr0::write(val); }
}

/// 寫入 CR3 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr3(val: u64) {
    use x86_64::{PhysAddr, structures::paging::PhysFrame};

    let frame = PhysFrame::containing_address(PhysAddr::new(val));
    unsafe {
        Cr3::write(frame, Cr3::read().1);
    }
}

/// 寫入 CR4 暫存器
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr4(val: Cr4Flags) {
    unsafe { Cr4::write(val); }
}

// ============ CPU 資訊 ============

/// 獲取 CPU 供應商 ID
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

    // 使用 raw_cpuid crate
    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();

    if let Some(vendor) = cpuid.get_vendor_info() {
        let vendor_string = vendor.as_str();
        let bytes = vendor_string.as_bytes();
        let copy_len = core::cmp::min(bytes.len(), model_out.len() - 1);

        model_out[..copy_len].copy_from_slice(&bytes[..copy_len]);
        model_out[copy_len] = 0; // null terminator

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
    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();
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

    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();

    if let Some(brand) = cpuid.get_processor_brand_string() {
        let brand_string = brand.as_str();
        let bytes = brand_string.as_bytes();
        let copy_len = core::cmp::min(bytes.len(), brand_out.len() - 1);

        brand_out[..copy_len].copy_from_slice(&bytes[..copy_len]);
        brand_out[copy_len] = 0; // null terminator

        core::str::from_utf8(&brand_out[..copy_len]).unwrap_or_else(|_| "Invalid UTF-8")
    } else {
        brand_out[0] = b'?';
        brand_out[1] = 0;
        "Unknown CPU"
    }
}

// ============ CPU 指令 ============

/// 讀取 CPU 時間戳計數器 (TSC)
///
/// # 返回
/// 時間戳計數值
#[allow(dead_code)]
#[inline]
pub fn cpu_rdtsc() -> u64 {
    unsafe {
        let low: u32;
        let high: u32;
        asm!(
        "rdtsc",
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags)
        );
        ((high as u64) << 32) | (low as u64)
    }
}

/// 執行 CPU 暫停指令 (減少功耗)
#[allow(dead_code)]
#[inline]
pub fn cpu_pause() {
    core::hint::spin_loop();
}

/// 停止 CPU 執行，直到下一個中斷發生
#[allow(dead_code)]
#[inline]
pub fn cpu_halt() {
    hlt();
}

/// 停止 CPU 並進入低功耗模式 (等同於 cpu_halt)
#[allow(dead_code)]
#[inline]
pub fn cpu_idle() {
    hlt();
}

/// 啟用中斷
#[allow(dead_code)]
#[inline]
pub fn cpu_enable_interrupts() {
    interrupts::enable();
}

/// 禁用中斷
#[allow(dead_code)]
#[inline]
pub fn cpu_disable_interrupts() {
    interrupts::disable();
}

/// 檢查中斷是否啟用
#[allow(dead_code)]
#[inline]
pub fn cpu_interrupts_enabled() -> bool {
    interrupts::are_enabled()
}

/// 在禁用中斷的情況下執行閉包
///
/// # 範例
/// ```
/// cpu_without_interrupts(|| {
///     // 臨界區代碼
///     // 中斷被禁用
/// });
/// // 中斷恢復到之前的狀態
/// ```
#[allow(dead_code)]
#[inline]
pub fn cpu_without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    interrupts::without_interrupts(f)
}

/// 無條件觸發斷點異常 (用於調試)
#[allow(dead_code)]
#[inline]
pub fn cpu_breakpoint() {
    x86_64::instructions::interrupts::int3();
}

/// 讀取 RFLAGS 寄存器
#[allow(dead_code)]
#[inline]
pub fn cpu_read_flags() -> u64 {
    x86_64::registers::rflags::read().bits()
}
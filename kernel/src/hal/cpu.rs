// kernel/src/hal/cpu.rs
use x86_64::registers::control::{Cr0, Cr0Flags, Cr2, Cr3, Cr4, Cr4Flags};
use x86_64::instructions::{interrupts, hlt};
use core::arch::asm;
use raw_cpuid::CpuId;
use x86_64::{PhysAddr, structures::paging::PhysFrame};
use x86_64::VirtAddr;

/// 64-bit register type
#[allow(dead_code)]
pub type Reg64 = u64;
/// 32-bit register type
#[allow(dead_code)]
pub type Reg32 = u32;
/// 16-bit register type
#[allow(dead_code)]
pub type Reg16 = u16;

/// General purpose register structure (64-bit)
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

/// Segment register structure
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

/// Read CR0 register
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr0() -> u64 {
    Cr0::read_raw()
}

/// Read CR2 register
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr2() -> u64 {
    Cr2::read_raw()
}

/// Read CR3 register
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr3_frame() -> PhysFrame {
    Cr3::read_raw().0
}

#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr3_flag() -> u16 {
    Cr3::read_raw().1
}

#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr3() -> u64 {
    cpu_r_cr3_addr().as_u64()
}

#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr3_addr() -> PhysAddr {
    cpu_r_cr3_frame().start_address()
}


/// Read CR4 register
#[allow(dead_code)]
#[inline]
pub fn cpu_r_cr4() -> u64 {
    Cr4::read_raw()
}

/// Write to CR0 register
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr0(val: Cr0Flags) {
    unsafe { Cr0::write(val); }
}

/// Write to CR3 register
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr3(val: u64) {
    let frame = PhysFrame::containing_address(PhysAddr::new(val));
    unsafe {
        Cr3::write(frame, Cr3::read().1);
    }
}

/// Write to CR4 register
#[allow(dead_code)]
#[inline]
pub fn cpu_w_cr4(val: Cr4Flags) {
    unsafe { Cr4::write(val); }
}

/// Get the CPU vendor ID
///
/// # Parameters
/// * `model_out` - Output buffer, requires at least 13 bytes
/// # Returns
/// A string slice representing the CPU vendor information
#[allow(dead_code)]
pub fn cpu_get_model(model_out: &mut [u8]) -> &str {
    if model_out.len() < 13 {
        return "Buffer too small";
    }

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

/// Check if brand string is supported
#[allow(dead_code)]
pub fn cpu_brand_string_supported() -> bool {
    let cpuid = CpuId::new();
    cpuid.get_processor_brand_string().is_some()
}

/// Get the CPU brand string
///
/// # Parameters
/// * `brand_out` - Output buffer, requires at least 49 bytes
///
/// # Returns
/// A string slice representing the CPU brand
#[allow(dead_code)]
pub fn cpu_get_brand(brand_out: &mut [u8]) -> &str {
    if brand_out.len() < 49 {
        return "Buffer too small";
    }
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

/// Read the CPU Time Stamp Counter (TSC)
///
/// # Return
/// The timestamp count value
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

/// Execute CPU pause instruction (reduce power consumption)
#[allow(dead_code)]
#[inline]
pub fn cpu_pause() {
    core::hint::spin_loop();
}

/// Stop CPU execution until the next interrupt occurs
#[allow(dead_code)]
#[inline]
pub fn cpu_halt() {
    hlt();
}

/// Stop the CPU and enter low power mode (equivalent to cpu_halt)
#[allow(dead_code)]
#[inline]
pub fn cpu_idle() {
    hlt();
}

/// Enable interrupts
#[allow(dead_code)]
#[inline]
pub fn cpu_enable_interrupts() {
    interrupts::enable();
}

/// Disable interrupts
#[allow(dead_code)]
#[inline]
pub fn cpu_disable_interrupts() {
    interrupts::disable();
}

/// Check if interrupts are enabled
#[allow(dead_code)]
#[inline]
pub fn cpu_interrupts_enabled() -> bool {
    interrupts::are_enabled()
}

/// Execute closure with interrupts disabled
///
/// # Example
/// ```
/// cpu_without_interrupts(|| {
/// // Critical section code
/// // Interrupts disabled
/// });
/// // Restore interrupts to their previous state
/// ```
#[allow(dead_code)]
#[inline]
pub fn cpu_without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    interrupts::without_interrupts(f)
}

/// Unconditionally trigger a breakpoint exception (for debugging)
#[allow(dead_code)]
#[inline]
pub fn cpu_breakpoint() {
    x86_64::instructions::interrupts::int3();
}

/// Read the RFLAGS register
#[allow(dead_code)]
#[inline]
pub fn cpu_read_flags() -> u64 {
    x86_64::registers::rflags::read().bits()
}
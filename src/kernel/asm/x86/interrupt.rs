use core::mem;
use core::ptr;
use x86::segmentation::{SegmentSelector, Descriptor};
use x86::Ring;
use x86::irq::{self, PageFaultError, EXCEPTIONS};
use crate::{print, println};
use crate::hal::cpu;

fn print_exception(vector: u8, error_code: Option<u32>, fault_addr: Option<u32>) {
    if vector < 32 {
        let ex = &EXCEPTIONS[vector as usize];
        crate::println!("CPU Exception: {}", ex);
        
        // 如果有錯誤碼
        if let Some(code) = error_code {
            println!("Error code: 0x{:x}", code);
            
            // 如果是頁錯誤，解析錯誤碼
            if vector == irq::PAGE_FAULT_VECTOR {
                let pf_error = PageFaultError::from_bits_truncate(code);
                println!("Fault details:\n{}", pf_error);
            }
        }
        
        // 如果有故障地址（如頁錯誤）
        if let Some(addr) = fault_addr {
            println!("Fault address: 0x{:x}", addr);
        }
    } else {
        crate::println!("Unhandled interrupt: Vector {}", vector);
    }
    
    crate::println!("System halted");
}

/// isr0
#[no_mangle]
pub fn divide_error_handler(error_code: u32) {
    let ex = &EXCEPTIONS[irq::DIVIDE_ERROR_VECTOR as usize];
    println!("Error code: 0x{:x}", error_code);
    println!("CPU Exception: {}", ex);
    loop {}
}
/// isr1
#[no_mangle]
pub fn debug_error_handler() {
    let ex = &EXCEPTIONS[irq::DEBUG_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr2
#[no_mangle]
pub fn nonmaskable_interrupt_handler() {
    let ex = &EXCEPTIONS[irq::NONMASKABLE_INTERRUPT_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr3
#[no_mangle]
pub fn breakpoint_handler() {
    let ex = &EXCEPTIONS[irq::BREAKPOINT_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr4
#[no_mangle]
pub fn overflow_handler() {
    let ex = &EXCEPTIONS[irq::OVERFLOW_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr5
#[no_mangle]
pub fn bound_range_exceeded_handler() {
    let ex = &EXCEPTIONS[irq::BOUND_RANGE_EXCEEDED_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr6
#[no_mangle]
pub fn invalid_opcode_handler() {
    let ex = &EXCEPTIONS[irq::INVALID_OPCODE_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr7
#[no_mangle]
pub fn device_not_available_handler() {
    let ex = &EXCEPTIONS[irq::DEVICE_NOT_AVAILABLE_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr8
#[no_mangle]
pub fn double_fault_handler() {
    let ex = &EXCEPTIONS[irq::DOUBLE_FAULT_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr9
#[no_mangle]
pub fn coprocessor_segment_overrun_handler() {
    let ex = &EXCEPTIONS[irq::COPROCESSOR_SEGMENT_OVERRUN_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr10
#[no_mangle]
pub fn invalid_tss_handler() {
    let ex = &EXCEPTIONS[irq::INVALID_TSS_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr11
#[no_mangle]
pub fn segment_not_present_handler() {
    let ex = &EXCEPTIONS[irq::SEGMENT_NOT_PRESENT_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr12
#[no_mangle]
pub fn stack_segment_fault_handler() {
    let ex = &EXCEPTIONS[irq::STACK_SEGEMENT_FAULT_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr13
#[no_mangle]
pub fn general_protection_handler(error_code: u32) {
    let ex = &EXCEPTIONS[irq::GENERAL_PROTECTION_FAULT_VECTOR as usize];
    let cr2: u32;
    cr2 = cpu::cpu_r_cr2() as u32;
    println!("CPU Exception: {}", ex);
    println!("Error code: 0x{:x}", error_code);
    println!("Fault address: 0x{:x}", cr2);
    loop {}
}

/// isr14
#[no_mangle]
pub fn page_fault_handler(error_code: u32) {
    let cr2: u32;
    cr2 = cpu::cpu_r_cr2() as u32;
    
    let ex = &EXCEPTIONS[irq::PAGE_FAULT_VECTOR as usize];
    println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
    println!("Error code: 0x{:x}", error_code);
    println!("Fault address: 0x{:x}", cr2);
    
    // 使用錯誤碼
    let error_flags = PageFaultError::from_bits_truncate(error_code);
    println!("Page fault details:\n{}", error_flags);
    
    loop {}
}

/// isr16
#[no_mangle]
pub fn x87_fpu_floating_point_handler() {
    let ex = &EXCEPTIONS[irq::X87_FPU_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr17
#[no_mangle]
pub fn alignment_check_handler() {
    let ex = &EXCEPTIONS[irq::ALIGNMENT_CHECK_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr18
#[no_mangle]
pub fn machine_check_handler() {
    let ex = &EXCEPTIONS[irq::MACHINE_CHECK_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr19
#[no_mangle]
pub fn simd_floating_point_handler() {
    let ex = &EXCEPTIONS[irq::SIMD_FLOATING_POINT_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

/// isr20
#[no_mangle]
pub fn virtualization_handler() {
    let ex = &EXCEPTIONS[irq::VIRTUALIZATION_VECTOR as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}


/// isr15, 21-31
#[no_mangle]
pub fn reserved_handler(vector: u8) {
    let ex = &EXCEPTIONS[vector as usize];
    println!("CPU Exception: {}", ex);
    loop {}
}

#[no_mangle]
pub fn default_interrupt_handler() {
    println!("Unhandled interrupt!");
    loop {}
}
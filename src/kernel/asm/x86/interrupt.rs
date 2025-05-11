use core::mem;
use core::ptr;
use x86::segmentation::{SegmentSelector, Descriptor};
use x86::Ring;
use x86::irq::{self, PageFaultError, EXCEPTIONS};
use crate::{print, println};
use crate::hal::cpu;

/// isr0
#[no_mangle]
pub extern "C" fn divide_error_handler() {
    let ex = &EXCEPTIONS[irq::DIVIDE_ERROR_VECTOR as usize];
    println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
    println!("Type: {}, Source: {}", ex.irqtype, ex.source);
    loop {}
}

#[no_mangle]
pub extern "C" fn page_fault_handler() {
    let cr2: u32;
    cr2 = cpu::cpu_r_cr2() as u32;
    
    let ex = &EXCEPTIONS[irq::PAGE_FAULT_VECTOR as usize];
    crate::println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
    crate::println!("Fault address: 0x{:x}", cr2);
    
    loop {}
}

#[no_mangle]
pub extern "C" fn general_protection_handler() {
    let ex = &EXCEPTIONS[irq::GENERAL_PROTECTION_FAULT_VECTOR as usize];
    crate::println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
    loop {}
}

#[no_mangle]
pub extern "C" fn default_interrupt_handler() {
    crate::println!("Unhandled interrupt!");
    loop {}
}
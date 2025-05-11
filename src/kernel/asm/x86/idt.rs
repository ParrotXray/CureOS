// src/kernel/asm/x86/idt.rs
use core::mem;
use core::ptr;
use x86::dtables::{DescriptorTablePointer, lidt};
use x86::segmentation::{
    self, 
    SegmentSelector, 
    Descriptor, 
    SystemDescriptorTypes32, 
    DescriptorBuilder, 
    BuildDescriptor
};
use x86::Ring;
use x86::irq::{self, EXCEPTIONS};
use x86::segmentation::GateDescriptorBuilder;
use x86::segmentation::TaskGateDescriptorBuilder;
use crate::kernel::asm::x86::interrupt;

pub const IDT_ENTRY_COUNT: usize = 256;

#[no_mangle]
pub static mut _IDT: [Descriptor; IDT_ENTRY_COUNT] = [Descriptor::NULL; IDT_ENTRY_COUNT];

#[no_mangle]
pub static mut _IDT_LIMIT: u16 = (mem::size_of::<[Descriptor; IDT_ENTRY_COUNT]>() - 1) as u16;

pub const KERNEL_CS: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);

pub type InterruptHandler = extern "C" fn() -> ();
pub type ExceptionHandlerWithErrorCode = extern "C" fn(error_code: u32) -> ();

#[no_mangle]
pub fn _set_interrupt_handler(vector: u8, handler: InterruptHandler) {
    unsafe {
        let desc = DescriptorBuilder::interrupt_descriptor(KERNEL_CS, handler as u32)
            .present() 
            .dpl(Ring::Ring0)
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

#[no_mangle]
pub fn _set_trap_handler(vector: u8, handler: InterruptHandler) {
    unsafe {
        let desc = DescriptorBuilder::trap_gate_descriptor(KERNEL_CS, handler as u32)
            .present() 
            .dpl(Ring::Ring0)
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

#[no_mangle]
pub fn _set_task_gate(vector: u8, tss_selector: SegmentSelector) {
    unsafe {
        let desc = DescriptorBuilder::task_gate_descriptor(tss_selector)
            .present() 
            .dpl(Ring::Ring0) 
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

#[no_mangle]
pub fn _init_idt() {
    unsafe {
        for i in 0..IDT_ENTRY_COUNT {
            _IDT[i] = Descriptor::NULL;
        }
    }
}

#[no_mangle]
pub fn _load_idt() {
    unsafe {
         _init_idt();
            
        let idtr = DescriptorTablePointer {
            limit: _IDT_LIMIT,
            base: ptr::addr_of!(_IDT) as *const _,
        };
        lidt(&idtr);

        _setup_idt();
    }
}

#[no_mangle]
pub fn _setup_idt() {
    _set_interrupt_handler(irq::DIVIDE_ERROR_VECTOR, interrupt::divide_error_handler);
    _set_interrupt_handler(irq::DEBUG_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::NONMASKABLE_INTERRUPT_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::BREAKPOINT_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::OVERFLOW_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::BOUND_RANGE_EXCEEDED_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::INVALID_OPCODE_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::DEVICE_NOT_AVAILABLE_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::DOUBLE_FAULT_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::COPROCESSOR_SEGMENT_OVERRUN_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::INVALID_TSS_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::SEGMENT_NOT_PRESENT_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::STACK_SEGEMENT_FAULT_VECTOR, interrupt::default_interrupt_handler);
    _set_interrupt_handler(irq::GENERAL_PROTECTION_FAULT_VECTOR, interrupt::general_protection_handler);
    _set_interrupt_handler(irq::PAGE_FAULT_VECTOR, interrupt::page_fault_handler);
    
    for i in 32..IDT_ENTRY_COUNT as u8 {
        _set_interrupt_handler(i, interrupt::default_interrupt_handler);
    }
    
    // crate::println!("IDT initialized with {} entries", IDT_ENTRY_COUNT);
}
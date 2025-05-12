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
use crate::kernel::asm::x86::{interrupt, segment};

pub const IDT_ENTRY_COUNT: usize = 256;

#[no_mangle]
pub static mut _IDT: [Descriptor; IDT_ENTRY_COUNT] = [Descriptor::NULL; IDT_ENTRY_COUNT];

#[no_mangle]
pub static mut _IDT_LIMIT: u16 = (mem::size_of::<[Descriptor; IDT_ENTRY_COUNT]>() - 1) as u16;

pub type InterruptHandler = fn() -> ();
pub type ExceptionHandlerWithErrorCode = fn(error_code: u32) -> ();

#[no_mangle]
pub fn _set_interrupt_handler(vector: u8, selector: SegmentSelector, handler: InterruptHandler, dpl: Ring) {
    unsafe {
        let desc = DescriptorBuilder::interrupt_descriptor(selector, handler as u32)
            .present() 
            .dpl(dpl)
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

#[no_mangle]
pub fn _set_interrupt_err_handler(vector: u8, selector: SegmentSelector, handler: ExceptionHandlerWithErrorCode, dpl: Ring) {
    unsafe {
        let desc = DescriptorBuilder::interrupt_descriptor(selector, handler as u32)
            .present() 
            .dpl(dpl)
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

#[no_mangle]
pub fn _set_trap_handler(vector: u8, selector: SegmentSelector, handler: InterruptHandler, dpl: Ring) {
    unsafe {
        let desc = DescriptorBuilder::trap_gate_descriptor(selector, handler as u32)
            .present() 
            .dpl(dpl)
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

#[no_mangle]
pub fn _set_task_gate(vector: u8, tss_selector: SegmentSelector, dpl: Ring) {
    unsafe {
        let desc = DescriptorBuilder::task_gate_descriptor(tss_selector)
            .present() 
            .dpl(dpl) 
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
    _set_interrupt_err_handler(irq::DIVIDE_ERROR_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::divide_error_handler, Ring::Ring0);
    _set_interrupt_handler(irq::DEBUG_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::debug_error_handler, Ring::Ring0);
    _set_interrupt_handler(irq::NONMASKABLE_INTERRUPT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::nonmaskable_interrupt_handler, Ring::Ring0);
    _set_interrupt_handler(irq::BREAKPOINT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::breakpoint_handler, Ring::Ring0);
    _set_interrupt_handler(irq::OVERFLOW_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::overflow_handler, Ring::Ring0);
    _set_interrupt_handler(irq::BOUND_RANGE_EXCEEDED_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::bound_range_exceeded_handler, Ring::Ring0);
    _set_interrupt_handler(irq::INVALID_OPCODE_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::invalid_opcode_handler, Ring::Ring0);
    _set_interrupt_handler(irq::DEVICE_NOT_AVAILABLE_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::device_not_available_handler, Ring::Ring0);
    _set_interrupt_handler(irq::DOUBLE_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::double_fault_handler, Ring::Ring0);
    _set_interrupt_handler(irq::COPROCESSOR_SEGMENT_OVERRUN_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::coprocessor_segment_overrun_handler, Ring::Ring0);
    _set_interrupt_handler(irq::INVALID_TSS_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::invalid_tss_handler, Ring::Ring0);
    _set_interrupt_handler(irq::SEGMENT_NOT_PRESENT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::segment_not_present_handler, Ring::Ring0);
    _set_interrupt_handler(irq::STACK_SEGEMENT_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::stack_segment_fault_handler, Ring::Ring0);
    _set_interrupt_err_handler(irq::GENERAL_PROTECTION_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::general_protection_handler, Ring::Ring0);
    _set_interrupt_err_handler(irq::PAGE_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::page_fault_handler, Ring::Ring0);
    
    _set_interrupt_handler(irq::X87_FPU_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::x87_fpu_floating_point_handler, Ring::Ring0);
    _set_interrupt_handler(irq::ALIGNMENT_CHECK_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::alignment_check_handler, Ring::Ring0);
    _set_interrupt_handler(irq::MACHINE_CHECK_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::machine_check_handler, Ring::Ring0);
    _set_interrupt_handler(irq::SIMD_FLOATING_POINT_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::simd_floating_point_handler, Ring::Ring0);
    _set_interrupt_handler(irq::VIRTUALIZATION_VECTOR, segment::KERNEL_CODE_SELECTOR, interrupt::virtualization_handler, Ring::Ring0);

    for i in 32..IDT_ENTRY_COUNT as u8 {
        _set_interrupt_handler(i, segment::KERNEL_CODE_SELECTOR, interrupt::default_interrupt_handler, Ring::Ring0);
    }
    
    // crate::println!("IDT initialized with {} entries", IDT_ENTRY_COUNT);
}
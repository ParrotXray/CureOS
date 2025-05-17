// src/kernel/asm/x86/idt.rs
use core::mem;
use core::ptr;
use x86::dtables::{DescriptorTablePointer, lidt};
use x86::segmentation::{
    SegmentSelector, 
    Descriptor, 
    DescriptorBuilder, 
    BuildDescriptor
};
use x86::Ring;
use x86::irq;
use x86::segmentation::GateDescriptorBuilder;
use x86::segmentation::TaskGateDescriptorBuilder;
use crate::kernel::asm::x86::segment;

pub const IDT_ENTRY_COUNT: usize = 256;

#[no_mangle]
pub static mut _IDT: [Descriptor; IDT_ENTRY_COUNT] = [Descriptor::NULL; IDT_ENTRY_COUNT];

#[no_mangle]
pub static mut _IDT_LIMIT: u16 = (mem::size_of::<[Descriptor; IDT_ENTRY_COUNT]>() - 1) as u16;

pub type InterruptHandler = unsafe extern "C" fn() -> ();

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
    extern "C" {
        fn _asm_isr0();
        fn _asm_isr1();
        fn _asm_isr2();
        fn _asm_isr3();
        fn _asm_isr4();
        fn _asm_isr5();
        fn _asm_isr6();
        fn _asm_isr7();
        fn _asm_isr8();
        fn _asm_isr9();
        fn _asm_isr10();
        fn _asm_isr11();
        fn _asm_isr12();
        fn _asm_isr13();
        fn _asm_isr14();
        fn _asm_isr15();
        fn _asm_isr16();
        fn _asm_isr17();
        fn _asm_isr18();
        fn _asm_isr19();
        fn _asm_isr20();
        fn _asm_isr21();
        fn _asm_isr22();
        fn _asm_isr23();
        fn _asm_isr24();
        fn _asm_isr25();
        fn _asm_isr26();
        fn _asm_isr27();
        fn _asm_isr28();
        fn _asm_isr29();
        fn _asm_isr30();
        fn _asm_isr31();
    }

    _set_interrupt_handler(irq::DIVIDE_ERROR_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr0, Ring::Ring0);
    _set_interrupt_handler(irq::DEBUG_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr1, Ring::Ring0);
    _set_interrupt_handler(irq::NONMASKABLE_INTERRUPT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr2, Ring::Ring0);
    _set_interrupt_handler(irq::BREAKPOINT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr3, Ring::Ring0);
    _set_interrupt_handler(irq::OVERFLOW_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr4, Ring::Ring0);
    _set_interrupt_handler(irq::BOUND_RANGE_EXCEEDED_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr5, Ring::Ring0);
    _set_interrupt_handler(irq::INVALID_OPCODE_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr6, Ring::Ring0);
    _set_interrupt_handler(irq::DEVICE_NOT_AVAILABLE_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr7, Ring::Ring0);
    _set_interrupt_handler(irq::DOUBLE_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr8, Ring::Ring0);
    _set_interrupt_handler(irq::COPROCESSOR_SEGMENT_OVERRUN_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr9, Ring::Ring0);
    _set_interrupt_handler(irq::INVALID_TSS_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr10, Ring::Ring0);
    _set_interrupt_handler(irq::SEGMENT_NOT_PRESENT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr11, Ring::Ring0);
    _set_interrupt_handler(irq::STACK_SEGEMENT_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr12, Ring::Ring0);
    _set_interrupt_handler(irq::GENERAL_PROTECTION_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr13, Ring::Ring0);
    _set_interrupt_handler(irq::PAGE_FAULT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr14, Ring::Ring0);
    _set_interrupt_handler(15, segment::KERNEL_CODE_SELECTOR, _asm_isr15, Ring::Ring0);
    _set_interrupt_handler(irq::X87_FPU_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr16, Ring::Ring0);
    _set_interrupt_handler(irq::ALIGNMENT_CHECK_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr17, Ring::Ring0);
    _set_interrupt_handler(irq::MACHINE_CHECK_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr18, Ring::Ring0);
    _set_interrupt_handler(irq::SIMD_FLOATING_POINT_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr19, Ring::Ring0);
    _set_interrupt_handler(irq::VIRTUALIZATION_VECTOR, segment::KERNEL_CODE_SELECTOR, _asm_isr20, Ring::Ring0);

    _set_interrupt_handler(21, segment::KERNEL_CODE_SELECTOR, _asm_isr21, Ring::Ring0);
    _set_interrupt_handler(22, segment::KERNEL_CODE_SELECTOR, _asm_isr22, Ring::Ring0);
    _set_interrupt_handler(23, segment::KERNEL_CODE_SELECTOR, _asm_isr23, Ring::Ring0);
    _set_interrupt_handler(24, segment::KERNEL_CODE_SELECTOR, _asm_isr24, Ring::Ring0);
    _set_interrupt_handler(25, segment::KERNEL_CODE_SELECTOR, _asm_isr25, Ring::Ring0);
    _set_interrupt_handler(26, segment::KERNEL_CODE_SELECTOR, _asm_isr26, Ring::Ring0);
    _set_interrupt_handler(27, segment::KERNEL_CODE_SELECTOR, _asm_isr27, Ring::Ring0);
    _set_interrupt_handler(28, segment::KERNEL_CODE_SELECTOR, _asm_isr28, Ring::Ring0);
    _set_interrupt_handler(29, segment::KERNEL_CODE_SELECTOR, _asm_isr29, Ring::Ring0);
    _set_interrupt_handler(30, segment::KERNEL_CODE_SELECTOR, _asm_isr30, Ring::Ring0);
    _set_interrupt_handler(31, segment::KERNEL_CODE_SELECTOR, _asm_isr31, Ring::Ring0);


    // for i in 32..IDT_ENTRY_COUNT as u8 {
    //     _set_interrupt_handler(i, segment::KERNEL_CODE_SELECTOR, interrupt::default_interrupt_handler, Ring::Ring0);
    // }
    
    // crate::println!("IDT initialized with {} entries", IDT_ENTRY_COUNT);
}
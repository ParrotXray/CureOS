// src/kernel/asm/x86/idt.rs
use core::mem;
use core::ptr;
use x86::dtables::{DescriptorTablePointer, lidt};
use x86::segmentation::{self, SegmentSelector, Descriptor, SystemDescriptorTypes32, DescriptorBuilder, BuildDescriptor};
use x86::Ring;
use x86::irq::{self, EXCEPTIONS};
use x86::segmentation::GateDescriptorBuilder;
use x86::segmentation::TaskGateDescriptorBuilder;

// 中斷處理常數
pub const IDT_ENTRY_COUNT: usize = 256;

// IDT 表
#[no_mangle]
pub static mut _IDT: [Descriptor; IDT_ENTRY_COUNT] = [Descriptor::NULL; IDT_ENTRY_COUNT];

// IDT 表大小限制
#[no_mangle]
pub static mut _IDT_LIMIT: u16 = (mem::size_of::<[Descriptor; IDT_ENTRY_COUNT]>() - 1) as u16;

// 內核代碼段選擇子
pub const KERNEL_CS: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);

// 異常/中斷處理函數類型
pub type InterruptHandler = extern "C" fn() -> ();
pub type ExceptionHandlerWithErrorCode = extern "C" fn(error_code: u32) -> ();

// 設置中斷處理函數
#[no_mangle]
pub fn set_interrupt_handler(vector: u8, handler: InterruptHandler) {
    unsafe {
        // 使用 DescriptorBuilder 正確地構建中斷描述符
        let desc = segmentation::DescriptorBuilder::interrupt_descriptor(KERNEL_CS, handler as u32)
            .present() // 必須設置存在位!
            .dpl(Ring::Ring0)
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

// 設置陷阱處理函數
#[no_mangle]
pub fn set_trap_handler(vector: u8, handler: InterruptHandler) {
    unsafe {
        // 使用 DescriptorBuilder 正確地構建陷阱描述符
        let desc = segmentation::DescriptorBuilder::trap_gate_descriptor(KERNEL_CS, handler as u32)
            .present() // 必須設置存在位!
            .dpl(Ring::Ring0)
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

#[no_mangle]
pub fn set_task_gate(vector: u8, tss_selector: SegmentSelector) {
    unsafe {
        // 使用 DescriptorBuilder 建立任務門描述符
        let desc = segmentation::DescriptorBuilder::task_gate_descriptor(tss_selector)
            .present() // 必須設置存在位
            .dpl(Ring::Ring0) // 設置特權級，通常為 Ring 0
            .finish();
        
        _IDT[vector as usize] = desc;
    }
}

// 初始化 IDT
#[no_mangle]
pub fn _init_idt() {
    // 初始化所有條目為 NULL
    unsafe {
        for i in 0..IDT_ENTRY_COUNT {
            _IDT[i] = Descriptor::NULL;
        }
    }
}

// 載入 IDT
#[no_mangle]
pub fn _load_idt() {
    unsafe {
         _init_idt();
            
        let idtr = DescriptorTablePointer {
            limit: _IDT_LIMIT,
            base: ptr::addr_of!(_IDT) as *const _,
        };
        lidt(&idtr);

        setup_idt();
    }
}

// 除零錯誤處理函數
#[no_mangle]
pub extern "C" fn divide_error_handler() {
    let ex = &EXCEPTIONS[irq::DIVIDE_ERROR_VECTOR as usize];
    crate::println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
    crate::println!("Type: {}, Source: {}", ex.irqtype, ex.source);
    loop {}
}

// 頁錯誤處理函數
#[no_mangle]
pub extern "C" fn page_fault_handler() {
    let cr2: u32;
    unsafe {
        cr2 = x86::controlregs::cr2() as u32;
    }
    
    let ex = &EXCEPTIONS[irq::PAGE_FAULT_VECTOR as usize];
    crate::println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
    crate::println!("Fault address: 0x{:x}", cr2);
    
    loop {}
}

// 通用保護錯誤處理函數
#[no_mangle]
pub extern "C" fn general_protection_handler() {
    let ex = &EXCEPTIONS[irq::GENERAL_PROTECTION_FAULT_VECTOR as usize];
    crate::println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
    loop {}
}

// 默認中斷處理函數
#[no_mangle]
pub extern "C" fn default_interrupt_handler() {
    crate::println!("Unhandled interrupt!");
    loop {}
}

// 為 CPU 所有異常設置處理函數
#[no_mangle]
pub fn setup_idt() {

    
    // 設置典型的異常處理函數
    set_interrupt_handler(irq::DIVIDE_ERROR_VECTOR, divide_error_handler);
    set_interrupt_handler(irq::DEBUG_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::NONMASKABLE_INTERRUPT_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::BREAKPOINT_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::OVERFLOW_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::BOUND_RANGE_EXCEEDED_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::INVALID_OPCODE_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::DEVICE_NOT_AVAILABLE_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::DOUBLE_FAULT_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::COPROCESSOR_SEGMENT_OVERRUN_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::INVALID_TSS_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::SEGMENT_NOT_PRESENT_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::STACK_SEGEMENT_FAULT_VECTOR, default_interrupt_handler);
    set_interrupt_handler(irq::GENERAL_PROTECTION_FAULT_VECTOR, general_protection_handler);
    set_interrupt_handler(irq::PAGE_FAULT_VECTOR, page_fault_handler);
    
    // 設置其餘中斷為默認處理函數
    for i in 32..IDT_ENTRY_COUNT as u8 {
        set_interrupt_handler(i, default_interrupt_handler);
    }
    
    crate::println!("IDT initialized with {} entries", IDT_ENTRY_COUNT);
}

// 實現對中斷的測試函數
pub fn test_divide_by_zero() {
    crate::println!("Testing divide by zero exception...");
    unsafe {
        core::arch::asm!(
            "mov $0, %eax",
            "mov $1, %ecx",
            "div %eax",
            options(att_syntax)
        );
    }
}
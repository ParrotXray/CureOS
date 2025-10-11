// kernel/src/kernel/asm/x86/interrupt.rs
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use crate::kprintln;

/// Divide Error (#DE)
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: DIVIDE ERROR (#DE)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Debug Exception (#DB)
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: DEBUG (#DB)");
    kprintln!("{:#?}", stack_frame);
}

/// Non-Maskable Interrupt (NMI)
pub extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: NON-MASKABLE INTERRUPT (NMI)");
    kprintln!("{:#?}", stack_frame);
}

/// Breakpoint (#BP)
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: BREAKPOINT (#BP)");
    kprintln!("{:#?}", stack_frame);
}

/// Overflow (#OF)
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: OVERFLOW (#OF)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Bound Range Exceeded (#BR)
pub extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: BOUND RANGE EXCEEDED (#BR)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Invalid Opcode (#UD)
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: INVALID OPCODE (#UD)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Device Not Available (#NM)
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: DEVICE NOT AVAILABLE (#NM)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Double Fault (#DF)
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT (#DF)\nError Code: {}\n{:#?}", error_code, stack_frame);
}

/// Invalid TSS (#TS)
pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprintln!();
    kprintln!("EXCEPTION: INVALID TSS (#TS)");
    kprintln!("Error Code: {:#x}", error_code);
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Segment Not Present (#NP)
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprintln!();
    kprintln!("EXCEPTION: SEGMENT NOT PRESENT (#NP)");
    kprintln!("Error Code: {:#x}", error_code);
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Stack Segment Fault (#SS)
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprintln!();
    kprintln!("EXCEPTION: STACK SEGMENT FAULT (#SS)");
    kprintln!("Error Code: {:#x}", error_code);
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// General Protection Fault (#GP)
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprintln!();
    kprintln!("EXCEPTION: GENERAL PROTECTION FAULT (#GP)");
    kprintln!("Error Code: {:#x}", error_code);
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Page Fault (#PF)
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    kprintln!();
    kprintln!("EXCEPTION: PAGE FAULT (#PF)");
    kprintln!("Accessed Address: {:?}", Cr2::read());
    kprintln!("Error Code: {:?}", error_code);
    kprintln!("  - Present: {}", error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION));
    kprintln!("  - Write: {}", error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE));
    kprintln!("  - User: {}", error_code.contains(PageFaultErrorCode::USER_MODE));
    kprintln!("  - Reserved Write: {}", error_code.contains(PageFaultErrorCode::MALFORMED_TABLE));
    kprintln!("  - Instruction Fetch: {}", error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH));
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// x87 Floating-Point Exception (#MF)
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: x87 FLOATING POINT (#MF)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Alignment Check (#AC)
pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprintln!();
    kprintln!("EXCEPTION: ALIGNMENT CHECK (#AC)");
    kprintln!("Error Code: {:#x}", error_code);
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Machine Check (#MC)
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE CHECK (#MC)\n{:#?}", stack_frame);
}

/// SIMD Floating-Point Exception (#XM/#XF)
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: SIMD FLOATING POINT (#XM/#XF)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Virtualization Exception (#VE)
pub extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    kprintln!("EXCEPTION: VIRTUALIZATION (#VE)");
    kprintln!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

// TODO Timer interrupt, Keyboard interrupt
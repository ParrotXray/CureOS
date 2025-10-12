// kernel/src/kernel/asm/amd64/isr
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use crate::kprintln;
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};

/// Divide Error (#DE)
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: DIVIDE ERROR (#DE)");
    log_error!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Debug Exception (#DB)
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_debug!("EXCEPTION: DEBUG (#DB)");
    log_debug!("{:#?}", stack_frame);
}

/// Non-Maskable Interrupt (NMI)
pub extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_fatal!("EXCEPTION: NON-MASKABLE INTERRUPT (NMI)");
    log_fatal!("{:#?}", stack_frame);
}

/// Breakpoint (#BP)
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_debug!("EXCEPTION: BREAKPOINT (#BP)");
    log_debug!("{:#?}", stack_frame);
}

/// Overflow (#OF)
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: OVERFLOW (#OF)");
    log_error!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Bound Range Exceeded (#BR)
pub extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: BOUND RANGE EXCEEDED (#BR)");
    log_error!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Invalid Opcode (#UD)
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: INVALID OPCODE (#UD)");
    log_error!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Device Not Available (#NM)
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: DEVICE NOT AVAILABLE (#NM)");
    log_error!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Double Fault (#DF)
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    kprintln!();
    log_fatal!("EXCEPTION: DOUBLE FAULT (#DF)");
    log_fatal!("Error Code: {:#x}", error_code);
    log_fatal!("{:#?}", stack_frame);
    panic!("DOUBLE FAULT - System cannot continue");
}

/// Invalid TSS (#TS)
pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprintln!();
    log_fatal!("EXCEPTION: INVALID TSS (#TS)");
    log_fatal!("Error Code: {:#x}", error_code);
    log_fatal!("{:#?}", stack_frame);
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
    log_fatal!("EXCEPTION: SEGMENT NOT PRESENT (#NP)");
    log_fatal!("Error Code: {:#x}", error_code);
    log_fatal!("{:#?}", stack_frame);
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
    log_fatal!("EXCEPTION: STACK SEGMENT FAULT (#SS)");
    log_fatal!("Error Code: {:#x}", error_code);
    log_fatal!("{:#?}", stack_frame);
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
    log_fatal!("EXCEPTION: GENERAL PROTECTION FAULT (#GP)");
    log_fatal!("Error Code: {:#x}", error_code);
    log_fatal!("{:#?}", stack_frame);
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
    log_fatal!("EXCEPTION: PAGE FAULT (#PF)");
    log_fatal!("Accessed Address: {:?}", Cr2::read());
    log_fatal!("Error Code: {:?}", error_code);
    log_fatal!("Present: {}", error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION));
    log_fatal!("Write: {}", error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE));
    log_fatal!("User: {}", error_code.contains(PageFaultErrorCode::USER_MODE));
    log_fatal!("Reserved Write: {}", error_code.contains(PageFaultErrorCode::MALFORMED_TABLE));
    log_fatal!("Instruction Fetch: {}", error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH));
    log_fatal!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// x87 Floating-Point Exception (#MF)
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: x87 FLOATING POINT (#MF)");
    log_error!("{:#?}", stack_frame);
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
    log_error!("EXCEPTION: ALIGNMENT CHECK (#AC)");
    log_error!("Error Code: {:#x}", error_code);
    log_error!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

/// Machine Check (#MC)
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    kprintln!();
    log_error!("EXCEPTION: MACHINE CHECK (#MC)");
    log_error!("{:#?}", stack_frame);
    panic!("MACHINE CHECK - System cannot continue");
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
    log_warn!("EXCEPTION: VIRTUALIZATION (#VE)");
    log_warn!("{:#?}", stack_frame);
    loop {
        crate::hal::cpu::cpu_halt();
    }
}

// TODO Timer interrupt, Keyboard interrupt
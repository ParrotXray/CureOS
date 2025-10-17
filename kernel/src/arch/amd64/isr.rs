use x86_64::instructions::port::Port;
// kernel/src/kernel/asm/amd64/isr
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use x86_64::VirtAddr;
use crate::{drivers, kprintln};
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};
use super::gdt;
use crate::hal::{timer, cpu, lapic, rtc};
use crate::mm::paging;

/// Divide Error (#DE)
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: DIVIDE ERROR (#DE)");
    log_error!("{:#?}", stack_frame);
    loop {
        cpu::cpu_halt();
    }
}

/// Debug Exception (#DB)
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_debug!("EXCEPTION: DEBUG (#DB)");
    log_debug!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_debug!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_debug!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_debug!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_debug!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());
}

/// Non-Maskable Interrupt (NMI)
pub extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_fatal!("EXCEPTION: NON-MASKABLE INTERRUPT (NMI)");
    log_fatal!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_fatal!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_fatal!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_fatal!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_fatal!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());
}

/// Breakpoint (#BP)
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_debug!("EXCEPTION: BREAKPOINT (#BP)");
    log_debug!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_debug!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_debug!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_debug!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_debug!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());
}

/// Overflow (#OF)
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: OVERFLOW (#OF)");
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
    }
}

/// Bound Range Exceeded (#BR)
pub extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: BOUND RANGE EXCEEDED (#BR)");
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
    }
}

/// Invalid Opcode (#UD)
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: INVALID OPCODE (#UD)");
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
    }
}

/// Device Not Available (#NM)
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: DEVICE NOT AVAILABLE (#NM)");
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
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

    log_fatal!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_fatal!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_fatal!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_fatal!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_fatal!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

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

    log_fatal!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_fatal!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_fatal!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_fatal!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_fatal!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
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
    log_fatal!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_fatal!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_fatal!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_fatal!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_fatal!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    let is_external = (error_code & 0x01) != 0;
    let table = if (error_code & 0x02) != 0 { "IDT" } else { "GDT" };
    let index = (error_code >> 3) & 0x1FFF;

    log_fatal!("Segment: {} index {:#x} (external: {})", table, index, is_external);

    log_fatal!("CS: {:#x}", gdt::kernel_code_selector().0);
    log_fatal!("SS: {:#x}", gdt::kernel_data_selector().0);

    loop {
        cpu::cpu_halt();
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
    log_fatal!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_fatal!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_fatal!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_fatal!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_fatal!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
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
    log_fatal!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_fatal!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_fatal!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_fatal!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_fatal!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
    }
}

/// Page Fault (#PF)
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {

    kprintln!();
    log_fatal!("EXCEPTION: PAGE FAULT (#PF)");
    log_fatal!("Accessed Address: {:?}", cpu::cpu_r_cr2());
    log_fatal!("Error Code: {:?}", error_code);
    log_fatal!("Present: {}", error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION));
    log_fatal!("Write: {}", error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE));
    log_fatal!("User: {}", error_code.contains(PageFaultErrorCode::USER_MODE));
    log_fatal!("Reserved Write: {}", error_code.contains(PageFaultErrorCode::MALFORMED_TABLE));
    log_fatal!("Instruction Fetch: {}", error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH));

    log_fatal!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_fatal!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_fatal!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_fatal!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_fatal!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    paging::handle_page_fault(
        VirtAddr::new(cpu::cpu_r_cr2()),
        error_code.bits()
    );

    loop {
        cpu::cpu_halt();
    }
}

/// x87 Floating-Point Exception (#MF)
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: x87 FLOATING POINT (#MF)");
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
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
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
    }
}

/// Machine Check (#MC)
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    kprintln!();
    log_error!("EXCEPTION: MACHINE CHECK (#MC)");
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    panic!("MACHINE CHECK - System cannot continue");
}

/// SIMD Floating-Point Exception (#XM/#XF)
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_error!("EXCEPTION: SIMD FLOATING POINT (#XM/#XF)");
    log_error!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_error!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_error!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_error!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_error!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
    }
}

/// Virtualization Exception (#VE)
pub extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    kprintln!();
    log_warn!("EXCEPTION: VIRTUALIZATION (#VE)");
    log_warn!("instruction pointer: {:#x}", stack_frame.instruction_pointer.as_u64());
    log_warn!("code segment: index: {:#?}, rpl: {:#?}", stack_frame.code_segment.index(), stack_frame.code_segment.rpl());
    log_warn!("cpu flags: {:#x}", stack_frame.cpu_flags.bits());
    log_warn!("stack pointer: {:#x}", stack_frame.stack_pointer.as_u64());
    log_warn!("stack segment: index: {:#?}, rpl: {:#?}", stack_frame.stack_segment.index(), stack_frame.stack_segment.rpl());

    loop {
        cpu::cpu_halt();
    }
}

// TODO Timer interrupt, Keyboard interrupt

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {

    unsafe {
        let mut port = Port::new(0x60);
        let scancode: u8 = port.read();

        drivers::keyboard::handle_scancode(scancode);
    }

    lapic::send_eoi();
}

pub extern "x86-interrupt" fn default_irq_handler(_stack_frame: InterruptStackFrame) {
    lapic::send_eoi();
    log_trace!("Unhandled IRQ");
}


pub extern "x86-interrupt" fn apic_timer_handler(_stack_frame: InterruptStackFrame) {

    // log_info!("Processing of APIC Timer Calibration Phase");
    if timer::is_calibrating() {
        timer::apic_calibration_handler();
    } else {
        timer::timer_tick_handler();
    }

    lapic::send_eoi();
}

pub extern "x86-interrupt" fn rtc_interrupt_handler(_stack_frame: InterruptStackFrame) {
    rtc::handle_interrupt();

    // log_info!("Processing of APIC Timer Calibration Phase");
    if timer::is_calibrating() {
        timer::rtc_calibration_handler();
    }

    lapic::send_eoi();
}


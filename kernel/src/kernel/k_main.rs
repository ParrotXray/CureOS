// kernel/src/kernel/k_main.rs
use crate::hal::{cpu, rtc};
use crate::kprintln;
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};
use crate::mm::{vma, vmm};
use crate::mm::allocator::pmm;

pub fn _kernel_main() -> ! {
    kprintln!();
    kprintln!("=== Welcome to CureOS! ===");
    kprintln!();

    let mut brand_buf = [0u8; 64];
    let mut model_buf = [0u8; 16];
    log_info!("CPU: {} ({})",
        cpu::cpu_get_brand(&mut brand_buf),
        cpu::cpu_get_model(&mut model_buf)
    );

    kprintln!();

    if let Some(stats) = pmm::get_memory_stats() {
        log_info!("Total Memory: {} MiB", stats.total_memory / (1024 * 1024));
        log_info!("Free Memory:  {} MiB", stats.free_memory / (1024 * 1024));
    }

    kprintln!();

    rtc::print_info();

    kprintln!();

    log_debug!("CR0: 0x{:016x}", cpu::cpu_r_cr0());
    log_debug!("CR2: 0x{:016x}", cpu::cpu_r_cr2());
    log_debug!("CR3: 0x{:016x}", cpu::cpu_r_cr3());
    log_debug!("CR4: 0x{:016x}", cpu::cpu_r_cr4());

    kprintln!();

    if let Some(apic_id) = crate::hal::lapic::get_apic_id() {
        log_info!("Configuring hardware interrupts...");
        log_info!("Current CPU APIC ID: {}", apic_id);


        log_info!("Setting up keyboard interrupt (IRQ 1 -> Vector 33)");
        crate::hal::ioapic::set_irq_redirect(
            1,                  // IRQ number (keyboard)
            33,              // Interrupt vector number
            apic_id as u8,         // APIC ID of target CPU
            false,     // Edge triggered (false = edge, true = level)
            false         // Active high (false = high, true = low)
        );

        // Unmask IRQ 1 (enable keyboard interrupt)
        crate::hal::ioapic::unmask_irq(1);
        log_info!("Keyboard interrupt unmasked");
        log_info!("Setting up RTC interrupt (IRQ 8 -> Vector 40)");
        crate::hal::ioapic::set_irq_redirect(
            8,              // RTC is IRQ 8
            40,             // Vector 40
            apic_id as u8,
            false,          // Edge triggered
            false           // Active high
        );

        crate::hal::ioapic::unmask_irq(8);
        log_info!("RTC interrupt unmasked");

        log_info!("Enabling RTC timer interrupt (1024Hz)...");
        // Reset counter
        rtc::reset_tick_count();
        rtc::enable_timer();

        cpu::cpu_enable_interrupts();

        log_info!("CPU interrupts enabled");

        test_rtc_interrupt();

        kprintln!();
        log_info!("Interrupt system ready!");

    } else {
        log_error!("APIC not available, cannot enable keyboard");
    }

    kprintln!();
    log_info!("System initialization complete!");
    log_warn!("Entering idle loop...");
    kprintln!();

    loop {
        cpu::cpu_halt();
    }
}

fn test_rtc_interrupt() {
    log_info!("=== RTC Interrupt Test ===");


    // Wait and check tick count
    log_info!("Waiting for RTC interrupts...");

    let start_count = rtc::get_tick_count();

    // Busy wait for ~1 second (approximately)
    for _ in 0..1000000 {
        cpu::cpu_pause();
    }

    let end_count = rtc::get_tick_count();
    let ticks = end_count - start_count;

    if ticks > 0 {
        log_info!("RTC interrupt working! Received {} ticks", ticks);
        log_info!("Expected: ~1024 ticks/second");
        log_info!("Actual rate: {} Hz", ticks);
    } else {
        log_error!("RTC interrupt NOT working! No ticks received");
    }

    // Live counter display
    log_info!("Live tick counter (press any key to continue):");

    let mut last_count = rtc::get_tick_count();
    let mut seconds = 0;

    for _ in 0..5 {  // Display for 5 seconds
        // Wait approximately 1 second
        for _ in 0..1000000 {
            cpu::cpu_pause();
        }

        let current_count = rtc::get_tick_count();
        let delta = current_count - last_count;
        last_count = current_count;
        seconds += 1;

        kprintln!("  [{}s] Total ticks: {}, Delta: {}, Rate: {} Hz",
                  seconds, current_count, delta, delta);
        // Also show current time
        // if let Some(time) = rtc::get_time() {
        //     kprintln!("       Time: {}", time.format());
        // }
    }

    log_info!("RTC test complete!");
    kprintln!();
}
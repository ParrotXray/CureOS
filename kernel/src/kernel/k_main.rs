// kernel/src/kernel/k_main.rs
use crate::hal::{timer, cpu, rtc};
use crate::{kprint, kprintln, shell};
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};
use crate::mm::{vma, vmm};
use crate::mm::allocator::pmm;

pub fn _kernel_main() -> ! {
    kprintln!();
    kprintln!("=== Welcome to CureOS! ===");
    kprintln!();

    let mut brand = [0u8; 64];
    let cpu_brand = cpu::cpu_get_brand(&mut brand);
    kprintln!("CPU:        {}", cpu_brand);

    if let Some(stats) = pmm::get_memory_stats() {
        kprintln!("Memory:     {} MiB / {} MiB free",
            stats.free_memory / (1024 * 1024),
            stats.total_memory / (1024 * 1024));
        kprintln!("            {}% used",
            (stats.used_memory * 100) / stats.total_memory);
    }

    if let Some((base_freq, running_freq, ticks)) = timer::get_info() {
        kprintln!("Timer:      {} Hz (base: {} Hz)", running_freq, base_freq);
        kprintln!("Ticks:      {}", ticks);
    }

    if let time = rtc::get_time() {
        kprintln!("Date/Time:  {}", time.format());
    }
    
    kprintln!();
    kprintln!("Type 'help' for available commands");
    kprintln!();

    shell::init();

    loop {
        cpu::cpu_halt();
    }
}

pub fn test_apic_timer() {

    log_info!("=== APIC Timer Test ===");

    let start_ticks = timer::get_tick_count();

    cpu::cpu_pause(1000);

    let end_ticks = timer::get_tick_count();
    let elapsed = end_ticks - start_ticks;

    log_info!("Elapsed ticks: {}", elapsed);

    if let Some((_, freq, _)) = timer::get_info() {
        log_info!("Expected ~{} ticks/sec", freq);
        log_info!("Actual rate: {} Hz", elapsed);
    }
}

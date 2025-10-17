// kernel/src/kernel/k_main.rs
use crate::hal::{apic_timer, cpu, rtc};
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

    test_apic_timer();

    log_info!("System initialization complete!");
    log_warn!("Entering idle loop...");
    kprintln!();

    loop {
        cpu::cpu_halt();
    }
}

pub fn test_apic_timer() {

    log_info!("=== APIC Timer Test ===");

    let start_ticks = apic_timer::get_tick_count();

    // 等待約 1 秒
    for _ in 0..1_000_000 {
        cpu::cpu_pause();
    }

    let end_ticks = apic_timer::get_tick_count();
    let elapsed = end_ticks - start_ticks;

    log_info!("Elapsed ticks: {}", elapsed);

    if let Some((_, freq, _)) = apic_timer::get_info() {
        log_info!("Expected ~{} ticks/sec", freq);
        log_info!("Actual rate: {} Hz", elapsed);
    }
}

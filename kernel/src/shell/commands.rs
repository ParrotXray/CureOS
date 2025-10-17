// kernel/src/shell/commands.rs
use crate::{kprintln, tty, hal::{rtc, timer, cpu}, mm::allocator::pmm, log_info};
use crate::hal::io::io_port_wb;

/// help 命令
pub fn cmd_help() {
    kprintln!("  help      - Show this help message");
    kprintln!("  clear     - Clear the screen");
    kprintln!("  time      - Display current date and time");
    kprintln!("  uptime    - Show system uptime");
    kprintln!("  sysinfo   - Display system information");
    kprintln!("  reboot    - Reboot the system");
    kprintln!();
}


pub fn cmd_clear() {
    tty::tty::clear(0x000000);
}

pub fn cmd_time() {
    if let time = rtc::get_time() {
        kprintln!("Current time: {}", time.format());
        kprintln!("{}, {} {}, {}",
            time.weekday_name(),
            time.month_name(),
            time.day,
            time.year
        );
    } else {
        kprintln!("Error: Unable to read RTC");
    }
}

pub fn cmd_uptime() {
    let ticks = timer::get_tick_count();
    let total_seconds = ticks / 100;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    kprintln!("System uptime: {}h {}m {}s ({} ticks)",
        hours, minutes, seconds, ticks);
}

pub fn cmd_echo(text: &str) {
    if text.is_empty() {
        kprintln!();
    } else {
        kprintln!("{}", text);
    }
}

pub fn cmd_sysinfo() {
    kprintln!();
    kprintln!("=== System Information ===");
    kprintln!();

    let mut brand = [0u8; 64];
    let cpu_brand = cpu::cpu_get_brand(&mut brand);
    kprintln!("CPU:        {}", cpu_brand);

    // 記憶體信息
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
}

pub fn cmd_reboot() {
    kprintln!("Rebooting system...");

    cpu::cpu_pause(1000);
    tty::tty::clear(0x000000);

    log_info!("Disabling CPU interrupts");
    cpu::cpu_disable_interrupts();

    log_info!("Disabling RTC Timer");
    rtc::disable_timer();

    unsafe {
        io_port_wb(0x64, 0xFE);

        log_info!("Complete");

        loop {
            cpu::cpu_halt();
        }
    }
}

pub fn cmd_halt() {
    kprintln!("System Statistics:");

    let ticks = timer::get_tick_count();
    let total_seconds = ticks / 100;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    kprintln!("  Uptime:      {}h {}m {}s", hours, minutes, seconds);

    if let Some(stats) = pmm::get_memory_stats() {
        kprintln!("  Memory Used: {} MiB / {} MiB",
            stats.used_memory / (1024 * 1024),
            stats.total_memory / (1024 * 1024));
    }

    if let time = rtc::get_time() {
        kprintln!("  Shutdown at: {}", time.format());
    }

    kprintln!();
    kprintln!("System halted. Safe to power off.");
    kprintln!();

    cpu::cpu_disable_interrupts();

    rtc::disable_timer();

    loop {
        cpu::cpu_halt();
    }
}

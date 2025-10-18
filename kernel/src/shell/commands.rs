// kernel/src/shell/commands.rs
use crate::{kprintln, tty, hal::{rtc, timer, cpu}, log_info};
use crate::hal::io::io_port_wb;
use crate::hal::power;
use crate::mm::{vma, vmm, allocator::pmm};
/// help 命令
pub fn cmd_help() {
    kprintln!("  help               - Show this help message");
    kprintln!("  clear | clr        - Clear the screen");
    kprintln!("  time               - Display current date and time");
    kprintln!("  uptime             - Show system uptime");
    kprintln!("  sysinfo | sys      - Display system information");
    kprintln!("  meminfo | mem      - Display memory information");
    kprintln!("  reboot             - Reboot the system");
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

pub fn cmd_meminfo() {
    kprintln!();
    kprintln!("=== Memory Layout ===");
    kprintln!();

    // 高半核地址空間布局
    kprintln!("Virtual Memory Layout:");
    kprintln!("  User Space:       {:#018x} - {:#018x}",
        0x0u64,
        0x0000_7FFF_FFFF_FFFFu64
    );
    kprintln!("  (Non-canonical):  {:#018x} - {:#018x}",
        0x0000_8000_0000_0000u64,
        0xFFFF_7FFF_FFFF_FFFFu64
    );
    kprintln!("  Physical Map:     {:#018x} - {:#018x}",
        vma::PHYS_MEM_OFFSET,
        vma::HIGHER_HALF_BASE - 1
    );
    kprintln!("  Kernel Base:      {:#018x}", vma::HIGHER_HALF_BASE);
    kprintln!("  Kernel Heap:      {:#018x} - {:#018x} ({} KiB)",
        vma::HEAP_START.as_u64(),
        vma::HEAP_START.as_u64() + vma::HEAP_SIZE as u64,
        vma::HEAP_SIZE / 1024
    );
    kprintln!("  Kernel Dynamic:   {:#018x} - {:#018x} ({} MiB)",
        vma::KERNEL_DYNAMIC_START.as_u64(),
        vma::KERNEL_DYNAMIC_END.as_u64(),
        vma::KERNEL_DYNAMIC_SIZE / (1024 * 1024)
    );
    kprintln!("  Kernel Stack:     {:#018x} - {:#018x} ({} MiB)",
        vma::KERNEL_STACK_START.as_u64(),
        vma::KERNEL_STACK_END.as_u64(),
        vma::KERNEL_STACK_SIZE / (1024 * 1024)
    );
    kprintln!("  Device Mapping:   {:#018x} - {:#018x} ({} MiB)",
        vma::DEVICE_MAPPING_START.as_u64(),
        vma::DEVICE_MAPPING_END.as_u64(),
        vma::DEVICE_MAPPING_SIZE / (1024 * 1024)
    );

    kprintln!();
    kprintln!("Physical Memory (PMM):");

    if let Some(stats) = pmm::get_memory_stats() {
        kprintln!("  Total:            {} MiB ({} frames)",
            stats.total_memory / (1024 * 1024),
            stats.total_frames
        );
        kprintln!("  Used:             {} MiB ({} frames)",
            stats.used_memory / (1024 * 1024),
            stats.allocated_frames
        );
        kprintln!("  Free:             {} MiB ({} frames)",
            stats.free_memory / (1024 * 1024),
            stats.free_frames
        );
        kprintln!("  Usage:            {}%",
            (stats.used_memory * 100) / stats.total_memory
        );
    } else {
        kprintln!("  (PMM not initialized)");
    }

    kprintln!();
    kprintln!("Virtual Memory Manager (VMM):");

    let vmm_stats = vmm::get_vmm_stats();
    kprintln!("  Next Address:     {:#018x}", vmm_stats.next_vaddr.as_u64());
    kprintln!("  Allocated:        {} pages ({} KiB) in {} blocks",
        vmm_stats.allocated_pages,
        vmm_stats.allocated_pages * 4,
        vmm_stats.allocated_blocks_count
    );
    kprintln!("  Free:             {} pages ({} KiB) in {} blocks",
        vmm_stats.free_pages,
        vmm_stats.free_pages * 4,
        vmm_stats.free_blocks_count
    );
    kprintln!("  New Usage:        {} pages ({} KiB)",
        vmm_stats.used_from_new,
        vmm_stats.used_from_new * 4
    );

    kprintln!();
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

pub fn cmd_shutdown() {
    kprintln!("Shutdown system...");
    cpu::cpu_pause(1000);
    tty::tty::clear(0x000000);
    power::shutdown();
}

pub fn cmd_reboot() {
    kprintln!("Rebooting system...");
    cpu::cpu_pause(1000);
    tty::tty::clear(0x000000);

    power::reboot();
}

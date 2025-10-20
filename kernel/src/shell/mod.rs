// kernel/src/shell/mod.rs
use alloc::string::String;
use crate::{kprint, kprintln, tty};
use crate::process::scheduler::Scheduler;
use crate::tty::device;

pub mod commands;
pub mod math;

/// Shell main loop (blocking)
pub fn shell_main(pid: u64, yielder: &corosensei::Yielder<(), ()>) {
    kprintln!();
    kprintln!("Shell started (PID: {})", pid);
    kprintln!("Type 'help' for available commands");
    kprintln!();

    loop {
        // Display the prompt
        tty::tty::write_str("cure > ", 0x00FF00);

        // Blocking read of a line
        let line = read_line_blocking(pid, yielder);

        // Parse into a string
        let cmd = match core::str::from_utf8(&line) {
            Ok(s) => s.trim(),
            Err(_) => {
                kprintln!("Invalid UTF-8 input");
                continue;
            }
        };

        // Execute command
        if !cmd.is_empty() {
            execute_command(cmd);
        }
    }
}

/// Blocking read of a line
fn read_line_blocking(pid: u64, yielder: &corosensei::Yielder<(), ()>) -> alloc::vec::Vec<u8> {
    loop {
        // Try to read
        if let Some(line) = device::read_line(pid) {
            return line;
        }

        // No data, blocking the current process
        Scheduler::block_current();

        // Yield to the scheduler
        yielder.suspend(());

        // After being woken up, continue to try to read
    }
}

fn execute_command(cmd: &str) {
    match cmd {
        "help" => commands::cmd_help(),
        "clear" | "clr" => commands::cmd_clear(),
        "time" => commands::cmd_time(),
        "uptime" => commands::cmd_uptime(),
        "sysinfo" | "sys" => commands::cmd_sysinfo(),
        "meminfo" | "mem" => commands::cmd_meminfo(),
        "reboot" => commands::cmd_reboot(),
        "halt" | "shutdown" | "poweroff" => commands::cmd_shutdown(),
        _ => {
            match math::eval_expression(cmd) {
                Ok(result) => kprintln!("{}", result),
                Err(_) => {
                    kprintln!("Unknown command: '{}'", cmd);
                    kprintln!("Type 'help' for available commands");
                }
            }
        }
    }
}
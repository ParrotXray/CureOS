// kernel/src/shell/mod.rs
use alloc::string::String;
use spin::Mutex;
use crate::{kprint, kprintln, tty, hal::{rtc, timer}};

pub mod commands;
pub mod math;

static COMMAND_BUFFER: Mutex<String> = Mutex::new(String::new());

pub fn init() {
    show_prompt();
}

pub fn show_prompt() {
    tty::tty::write_str("cure > ", 0x00FF00);
}

pub fn process_keyboard_char(c: char) {
    let mut buffer = COMMAND_BUFFER.lock();

    if c == '\n' {
        let cmd = buffer.clone();
        buffer.clear();

        kprintln!();
        execute_command(&cmd);
        show_prompt();
    } else if c == '\x08' {
        if !buffer.is_empty() {
            buffer.pop();
            kprint!("\x08 \x08");
        }
    } else if c.is_ascii_graphic() || c == ' ' {
        buffer.push(c);
        kprint!("{}", c);
    }
}

fn execute_command(cmd: &str) {
    let mut cmd = cmd.trim();

    if cmd.is_empty() {
        return;
    }
    
    match cmd {
        "help" => commands::cmd_help(),
        "clear" | "clr" => commands::cmd_clear(),
        "time" => commands::cmd_time(),
        "uptime" => commands::cmd_uptime(),
        "sysinfo" | "sys"  => commands::cmd_sysinfo(),
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
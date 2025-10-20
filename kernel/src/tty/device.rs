// kernel/src/tty/device
use alloc::{collections::VecDeque, vec::Vec};
use spin::Mutex;
use crate::{kprint, process};

/// TTY device
pub struct Device {
    /// Input buffer (complete line)
    input_buffer: VecDeque<u8>,
    /// The line currently being edited
    line_buffer: Vec<u8>,
    /// Waiting to read the process ID
    waiting_process: Option<u64>,
    /// Whether to echo
    echo: bool,
}

impl Device {
    pub const fn new() -> Self {
        Self {
            input_buffer: VecDeque::new(),
            line_buffer: Vec::new(),
            waiting_process: None,
            echo: true,
        }
    }

    /// Receive a character from the keyboard (interrupt context call)
    pub fn receive_char(&mut self, c: u8) {
        match c {
            b'\n' => {
                // Enter: Complete the current line
                self.line_buffer.push(b'\n');

                // echo newline
                if self.echo {
                    kprint!("\n");
                }

                // Move the entire line to the input buffer
                self.input_buffer.extend(self.line_buffer.drain(..));

                //Wake up the waiting process
                if let Some(pid) = self.waiting_process.take() {
                    process::scheduler::Scheduler::wake_process(pid);
                }
            }

            8 | 127 => {  // Backspace
                if !self.line_buffer.is_empty() {
                    self.line_buffer.pop();
                    if self.echo {
                        kprint!("\x08 \x08");
                    }
                }
            }

            3 => {  // Ctrl+C
                if self.echo {
                    kprint!("^C\n");
                }
                self.line_buffer.clear();

                // Wake up the process (but return a blank line or a special mark)
                if let Some(pid) = self.waiting_process.take() {
                    crate::process::scheduler::Scheduler::wake_process(pid);
                }
            }

            12 => {  // Ctrl+L (clear screen)
                if self.echo {
                    crate::tty::tty::clear(0x000000);
                }
            }

            c if c >= 32 && c < 127 => {
                self.line_buffer.push(c);
                if self.echo {
                    kprint!("{}", c as char);
                }
            }

            _ => {
                // Ignore other characters
            }
        }
    }

    /// Read a line (blocking call)
    /// Returning None indicates a blocking wait.
    pub fn read_line(&mut self, pid: u64) -> Option<Vec<u8>> {
        // If there is a complete line, return immediately
        if let Some(pos) = self.input_buffer.iter().position(|&c| c == b'\n') {
            let mut line = Vec::new();
            for _ in 0..=pos {
                if let Some(c) = self.input_buffer.pop_front() {
                    line.push(c);
                }
            }
            return Some(line);
        }

        // Otherwise, record the waiting process and return None
        self.waiting_process = Some(pid);
        None
    }

    /// Check if there is a complete line to read
    pub fn has_line(&self) -> bool {
        self.input_buffer.iter().any(|&c| c == b'\n')
    }
}

// 全局 TTY 設備
static TTY0: Mutex<Device> = Mutex::new(Device::new());

/// 接收字符（給鍵盤驅動調用）
pub fn receive_char(c: u8) {
    TTY0.lock().receive_char(c);
}

/// 讀取一行（給進程調用）
pub fn read_line(pid: u64) -> Option<Vec<u8>> {
    TTY0.lock().read_line(pid)
}

/// 檢查是否有數據
pub fn has_input() -> bool {
    TTY0.lock().has_line()
}
// src/hal/io.rs
use x86::io::{inb, inl, outb, outl};

// :e - 32 位寄存器（如 eax, edx 等）
// :x - 16 位寄存器（如 ax, dx 等）
// :l - 8 位低位寄存器（如 al, dl 等）
// :h - 8 位高位寄存器（如 ah, dh 等）

/// 向 I/O 端口寫入一個字節
/// 
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[allow(dead_code)]
#[inline]
pub fn io_port_wb(port: u16, value: u8) {
    unsafe {
        outb(port, value);
    }
}

/// 向 I/O 端口寫入一個雙字
/// 
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[allow(dead_code)]
#[inline]
pub fn io_port_wl(port: u16, value: u32) {
    unsafe {
        outl(port, value);
    }
}

/// 從 I/O 端口讀取一個字節
/// 
/// # 參數
/// * `port` - 端口號
/// # 返回
/// 讀取到的值
#[allow(dead_code)]
#[inline]
pub fn io_port_rb(port: u16) -> u8 {
    unsafe {
        inb(port)
    }
}

/// 從 I/O 端口讀取一個雙字
/// 
/// # 參數
/// * `port` - 端口號
/// # 返回
/// 讀取到的值
#[allow(dead_code)]
#[inline]
pub fn io_port_rl(port: u16) -> u32 {
    unsafe {
        inl(port)
    }
}

/// 向 I/O 端口寫入一個字
/// 
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[allow(dead_code)]
#[inline]
pub fn io_port_ww(port: u16, value: u16) {
    unsafe {
        x86::io::outw(port, value);
    }
}

/// 從 I/O 端口讀取一個字
/// 
/// # 參數
/// * `port` - 端口號
/// # 返回
/// 讀取到的值
#[allow(dead_code)]
#[inline]
pub fn io_port_rw(port: u16) -> u16 {
    unsafe {
        x86::io::inw(port)
    }
}
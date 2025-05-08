// src/hal/io.rs
use x86::io;

// :e - 32 位寄存器（如 eax, edx 等）
// :x - 16 位寄存器（如 ax, dx 等）
// :l - 8 位低位寄存器（如 al, dl 等）
// :h - 8 位高位寄存器（如 ah, dh 等）

use core::arch::asm;

/// 向 I/O 端口寫入一個字節
/// 
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[inline]
pub fn io_port_wb(port: u16, value: u8) {
    unsafe {
        asm!(
            "movb {0}, %al",
            "movw {1:x}, %dx",
            "outb %al, %dx",
            in(reg_byte) value,
            in(reg) port,
            out("al") _, out("dx") _,
            options(att_syntax)
        );
    }
}

/// 向 I/O 端口寫入一個雙字
/// 
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[inline]
pub fn io_port_wl(port: u16, value: u32) {
    unsafe {
        asm!(
            "movl {0}, %eax",
            "movw {1:x}, %dx",
            "outl %eax, %dx",
            in(reg) value,
            in(reg) port,
            out("al") _, out("dx") _,
            options(att_syntax)
        );
    }
}

/// 從 I/O 端口讀取一個字節
/// 
/// # 參數
/// * `port` - 端口號
/// # 返回
/// 讀取到的值
#[inline]
pub fn io_port_rb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "movw {1:x}, %dx",
            "inb %dx, %al",
            "movb %al, {0}",
            out(reg_byte) value,
            in(reg) port,
            out("al") _, out("dx") _,
            options(att_syntax)
        );
    }
    value
}

/// 從 I/O 端口讀取一個雙字
/// 
/// # 參數
/// * `port` - 端口號
/// # 返回
/// 讀取到的值
#[inline]
pub fn io_port_rl(port: u16) -> u32 {
    let value: u32;
    unsafe {
        asm!(
            "movw {1:x}, %dx",
            "inl %dx, %eax",
            "movl %eax, {0}",
            out(reg) value,
            in(reg) port,
            out("al") _, out("dx") _,
            options(att_syntax)
        );
    }
    value
}
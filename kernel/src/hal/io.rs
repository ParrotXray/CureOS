// kernel/src/hal/io.rs
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

/// 向 I/O 端口寫入一個字節
///
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[allow(dead_code)]
#[inline]
pub fn io_port_wb(port: u16, value: u8) {
    unsafe {
        Port::new(port).write(value);
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
        Port::new(port).read()
    }
}

/// 向 I/O 端口寫入一個字 (16-bit)
///
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[allow(dead_code)]
#[inline]
pub fn io_port_ww(port: u16, value: u16) {
    unsafe {
        Port::new(port).write(value);
    }
}

/// 從 I/O 端口讀取一個字 (16-bit)
///
/// # 參數
/// * `port` - 端口號
/// # 返回
/// 讀取到的值
#[allow(dead_code)]
#[inline]
pub fn io_port_rw(port: u16) -> u16 {
    unsafe {
        Port::new(port).read()
    }
}

/// 向 I/O 端口寫入一個雙字 (32-bit)
///
/// # 參數
/// * `port` - 端口號
/// * `value` - 要寫入的值
#[allow(dead_code)]
#[inline]
pub fn io_port_wl(port: u16, value: u32) {
    unsafe {
        Port::new(port).write(value);
    }
}

/// 從 I/O 端口讀取一個雙字 (32-bit)
///
/// # 參數
/// * `port` - 端口號
/// # 返回
/// 讀取到的值
#[allow(dead_code)]
#[inline]
pub fn io_port_rl(port: u16) -> u32 {
    unsafe {
        Port::new(port).read()
    }
}
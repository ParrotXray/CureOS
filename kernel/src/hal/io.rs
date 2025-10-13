// kernel/src/hal/io.rs
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

/// Write a byte to an I/O port
///
/// # Parameters
/// * `port` - Port number
/// * `value` - Value to write
#[allow(dead_code)]
#[inline]
pub fn io_port_wb(port: u16, value: u8) {
    unsafe {
        Port::new(port).write(value);
    }
}

/// Read a byte from an I/O port
///
/// # Parameters
/// * `port` - port number
/// # Returns
/// The value read
#[allow(dead_code)]
#[inline]
pub fn io_port_rb(port: u16) -> u8 {
    unsafe {
        Port::new(port).read()
    }
}

/// Write a word (16-bit) to an I/O port
///
/// # Parameters
/// * `port` - Port number
/// * `value` - Value to be written
#[allow(dead_code)]
#[inline]
pub fn io_port_ww(port: u16, value: u16) {
    unsafe {
        Port::new(port).write(value);
    }
}

/// Read a word (16-bit) from an I/O port
///
/// # Parameters
/// * `port` - port number
/// # Returns
/// The value read
#[allow(dead_code)]
#[inline]
pub fn io_port_rw(port: u16) -> u16 {
    unsafe {
        Port::new(port).read()
    }
}

/// Write a double word (32-bit) to an I/O port
///
/// # Parameters
/// * `port` - Port number
/// * `value` - Value to be written
#[allow(dead_code)]
#[inline]
pub fn io_port_wl(port: u16, value: u32) {
    unsafe {
        Port::new(port).write(value);
    }
}

/// Read a double word (32-bit) from an I/O port
///
/// # Parameters
/// * `port` - port number
/// # Returns
/// The value read
#[allow(dead_code)]
#[inline]
pub fn io_port_rl(port: u16) -> u32 {
    unsafe {
        Port::new(port).read()
    }
}
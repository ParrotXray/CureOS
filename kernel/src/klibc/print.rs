// src/klibc/libc/print.rs

use core::fmt;
use crate::tty::tty;

pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        tty::tty_put_str(s, None);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    Writer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::klibc::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => {
        $crate::klibc::print::_print(format_args!("\n"))
    };
    ($($arg:tt)*) => {
        $crate::klibc::print::_print(format_args!("{}\n", format_args!($($arg)*)))
    };
}
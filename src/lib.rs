#![no_std]
#![no_main]

mod kernel;
mod hal;
mod libs;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
#![no_std]
#![no_main]

mod kernel;
mod hal;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
#![no_std]
#![no_main]

mod arch;
mod kernel;
mod hal;
mod libs;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// #[no_mangle]
// #[link_section = ".multiboot"]
// pub static MULTIBOOT_HEADER: [u32; 3] = [
//     arch::x86::multiboot::MULTIBOOT_MAGIC,
//     arch::x86::multiboot::MULTIBOOT_MEMORY_INFO | arch::x86::multiboot::MULTIBOOT_PAGE_ALIGN,
//     arch::x86::multiboot::checksum(arch::x86::multiboot::MULTIBOOT_MEMORY_INFO | arch::x86::multiboot::MULTIBOOT_PAGE_ALIGN),
// ];
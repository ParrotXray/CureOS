// kernel/src/main.rs
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootInfo};

mod kernel;
mod hal;
mod libs;
mod logger;

use kernel::tty::tty;
use kernel::asm::x86::gdt;


entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        tty::init(framebuffer);
        tty::clear(0x000000);

        kprintln!("CureOS Booting...");
        kprintln!("Framebuffer initialized");
        kprintln!("Initializing GDT...");

        gdt::init();
        kprintln!("GDT initialized");

        gdt::print_info();

        kprintln!();
        kprintln!("Welcome to CureOS!");

    } else {
        loop {
            hal::cpu::cpu_halt();
        }
    }

    loop {
        hal::cpu::cpu_halt();
    }
}
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!();
    kprintln!("================================");
    kprintln!("       KERNEL PANIC!"            );
    kprintln!("================================");
    kprintln!();
    kprintln!("{}", info);

    loop {
        hal::cpu::cpu_halt();
    }
}
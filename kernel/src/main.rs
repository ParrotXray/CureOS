// kernel/src/main.rs
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
extern crate alloc;
use alloc::vec::Vec;

use core::panic::PanicInfo;
use core::alloc::Layout;
use bootloader_api::{config, entry_point, BootInfo, BootloaderConfig};
use x86_64::structures::paging::OffsetPageTable;
use x86_64::{
    structures::paging::PageTable,
    VirtAddr,
};

mod kernel;
mod hal;
mod libs;
mod logger;
mod k_init;
mod k_main;

use hal::cpu;
const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(config::Mapping::Dynamic);
    config
};

entry_point!(k_init::kernel_init, config = &CONFIG);


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
        cpu::cpu_halt();
    }
}

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
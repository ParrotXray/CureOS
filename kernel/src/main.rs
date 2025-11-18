// kernel/src/main.rs
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
extern crate alloc;
pub mod hal;
pub mod klibc;
pub mod arch;
pub mod mm;
pub mod tty;
pub mod kernel;
pub mod drivers;
pub mod shell;
pub mod task;
pub mod process;

use hal::cpu;
use alloc::vec::Vec;

use core::panic::PanicInfo;
use core::alloc::Layout;
use bootloader_api::{config, entry_point, BootInfo, BootloaderConfig};
use x86_64::structures::paging::OffsetPageTable;
use x86_64::{
    structures::paging::PageTable,
    VirtAddr,
};

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(config::Mapping::Dynamic);
    config
};

entry_point!(kernel::k_init::_kernel_init, config = &CONFIG);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!();
    kprintln!("================================");
    kprintln!("       KERNEL PANIC!"            );
    kprintln!("================================");
    kprintln!();
    log_fatal!("{}", info);
    kprintln!();
    log_fatal!("CR0: 0x{:016x}", cpu::cpu_r_cr0());
    log_fatal!("CR2: 0x{:016x}", cpu::cpu_r_cr2());
    log_fatal!("CR3: 0x{:016x}", cpu::cpu_r_cr3());
    log_fatal!("CR4: 0x{:016x}", cpu::cpu_r_cr4());

    loop {
        cpu::cpu_halt();
    }
}

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
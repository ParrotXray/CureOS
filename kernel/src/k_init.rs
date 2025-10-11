use bootloader_api::BootInfo;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;
use crate::kernel::asm::x86::{gdt, idt, acpi};
use crate::kernel::mm::{heap, frame_allocator};
use crate::kernel::tty::tty;
use crate::kprintln;
use crate::k_main;
use crate::hal::cpu;

pub fn kernel_init(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        tty::init(framebuffer);
        tty::clear(0x000000);

        kprintln!("CureOS Booting...");
        kprintln!("Framebuffer initialized");

        kprintln!();
        kprintln!("Initializing GDT...");
        gdt::init();
        kprintln!("GDT initialized");
        gdt::print_info();

        kprintln!();
        kprintln!("Initializing IDT...");
        idt::init();
        kprintln!("IDT initialized");
        idt::print_info();

        kprintln!();
        tty::clear(0x000000);
        let physical_memory_offset = if let Some(offset) = boot_info.physical_memory_offset.into_option() {
            kprintln!("Physical Memory Offset: {:#x}", offset);
            offset
        } else {
            panic!("Physical memory offset not provided by bootloader");
        };

        let phys_mem_offset = VirtAddr::new(physical_memory_offset);
        let mut mapper = unsafe {
            OffsetPageTable::new(heap::get_level_4_table(phys_mem_offset), phys_mem_offset)
        };

        let mut frame_allocator = unsafe {
           frame_allocator::BootInfoFrameAllocator::init(&boot_info.memory_regions)
        };

        kprintln!("Initializing heap allocator...");
        heap::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap initialization failed");

        kprintln!("Heap allocator initialized");
        kprintln!("Heap Start: {:#x}", heap::HEAP_START);
        kprintln!("Heap Size: {} KiB", heap::HEAP_SIZE / 1024);

        kprintln!();
        if let Some(rsdp) = boot_info.rsdp_addr.into_option() {
            kprintln!("RSDP Address: {:#x}", rsdp);
            kprintln!("Initializing ACPI...");

            if let Some(acpi_info) = acpi::init(*&rsdp, physical_memory_offset) {
                acpi::print_info(&acpi_info);
            } else {
                kprintln!("Warning: ACPI initialization failed");
            }
        } else {
            kprintln!("Warning: RSDP not provided");
        }


        k_main::kernel_main();

    } else {
        loop {
            cpu::cpu_halt();
        }
    }
}
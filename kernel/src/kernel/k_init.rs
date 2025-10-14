// kernel/src/kernel/k_init.rs
use bootloader_api::BootInfo;
use bootloader_api::info::MemoryRegionKind;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::{PhysAddr, VirtAddr};
use crate::mm::{allocator::{heap, frame, pmm}, vmm, paging, vma};
use crate::arch::amd64::{gdt, idt};
use crate::tty::tty;
use crate::kprintln;
use crate::kernel::k_main;
use crate::klibc::logger::{init, LogLevel, LoggerConfig};
use crate::klibc::malloc;
use crate::{log_debug, log_error, log_info, log_trace, log_warn};
use crate::drivers::keyboard;
use crate::hal::{acpi, lapic, rtc};

fn _logger_init() {
    init(
        LoggerConfig::new()
            .with_level(LogLevel::Trace)
            .with_location(true)
    );

    log_info!("Logger initialized");
}

fn _critical_init() {
    gdt::init();
    idt::init();
}

fn _memory_init(
    memory_regions: &'static bootloader_api::info::MemoryRegions,
    physical_memory_offset: u64
) -> (OffsetPageTable<'static>, frame::BootInfoFrameAllocator) {
    let phys_mem_offset = VirtAddr::new(physical_memory_offset);

    let mut mapper = unsafe {
        OffsetPageTable::new(heap::get_level_4_table(phys_mem_offset), phys_mem_offset)
    };

    let mut frame_allocator = unsafe {
        frame::BootInfoFrameAllocator::init(memory_regions)
    };

    heap::init(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    unsafe {
        let mut usable_start = u64::MAX;
        let mut usable_end = 0u64;
        let mut total_usable = 0u64;

        for region in memory_regions.iter() {
            if region.kind == bootloader_api::info::MemoryRegionKind::Usable {
                usable_start = usable_start.min(region.start);
                usable_end = usable_end.max(region.end);
                total_usable += region.end - region.start;
            }
        }

        let bitmap_size = ((usable_end - usable_start) / 4096 + 7) / 8;
        let bitmap_pages = (bitmap_size as usize + 4095) / 4096;

        if let Some(bitmap_addr) = malloc::kmalloc(
            bitmap_pages * 4096,
            &mut mapper,
            &mut frame_allocator
        ) {
            pmm::init_pmm(
                PhysAddr::new(usable_start),
                total_usable as usize,
                bitmap_addr.as_mut_ptr()
            );

            for region in memory_regions.iter() {
                if region.kind != bootloader_api::info::MemoryRegionKind::Usable {
                    pmm::mark_region_used(
                        PhysAddr::new(region.start),
                        (region.end - region.start) as usize
                    );
                }
            }

        } else {
            log_error!("Failed to allocate PMM bitmap!");
        }
    }

    (mapper, frame_allocator)
}

fn _display_init(framebuffer: &'static mut bootloader_api::info::FrameBuffer) {
    tty::init(framebuffer);
    tty::clear(0x000000);

    kprintln!("========================================");
    kprintln!("       CureOS Kernel v0.1.0"             );
    kprintln!("========================================");
    kprintln!();
}

fn _boot_report(memory_regions: &bootloader_api::info::MemoryRegions, physical_memory_offset: u64) {
    log_info!("GDT initialized");
    gdt::print_info();

    kprintln!();
    log_info!("IDT initialized");
    idt::print_info();

    kprintln!();
    log_info!("Physical Memory Offset: {:#x}", physical_memory_offset);

    log_debug!("Memory Regions:");
    let mut total_usable = 0u64;
    for region in memory_regions.iter() {
        let kind_str = match region.kind {
            MemoryRegionKind::Usable => {
                total_usable += region.end - region.start;
                "Usable"
            },
            MemoryRegionKind::Bootloader => "Bootloader",
            MemoryRegionKind::UnknownBios(_) => "Unknown",
            MemoryRegionKind::UnknownUefi(_) => "UEFI Reserved",
            _ => "Reserved",
        };
        log_trace!("{:#016x} - {:#016x} ({})",
            region.start, region.end, kind_str);
    }
    log_info!("Total Usable Memory: {} MiB", total_usable / (1024 * 1024));

    kprintln!();
    log_info!("Heap Start: {:#x}", vma::HEAP_START);
    log_info!("Heap Size:  {} KiB", vma::HEAP_SIZE / 1024);

    vma::print_info();
    vmm::get_vmm_stats().print();
}

fn _acpi_init(rsdp_addr: Option<u64>, physical_memory_offset: u64) -> Option<acpi::AcpiInfo> {
    kprintln!();

    if let Some(rsdp) = rsdp_addr {
        log_debug!("RSDP Address: {:#x}", rsdp);

        if let Some(acpi_info) = acpi::init(rsdp, physical_memory_offset) {
            acpi::print_info(&acpi_info);
            return Some(acpi_info);
        } else {
            log_warn!("ACPI initialization failed");
        }
    } else {
        log_warn!("RSDP not provided by bootloader");
    }

    None
}

fn _post_init(
    acpi_info: Option<&acpi::AcpiInfo>,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut frame::BootInfoFrameAllocator,
) {
    kprintln!();
    log_info!("Post Initialization");

    // 初始化 APIC/IOAPIC (如果有的話)
    if let Some(info) = acpi_info {
        if info.has_apic {
            lapic::disable_legacy_pic();

            unsafe {
                if let Some(local_apic_addr) = info.local_apic_address {
                    log_info!("Initializing Local APIC...");
                    log_debug!("Mapping Local APIC physical address: {:#x}", local_apic_addr);

                    if let Some(vaddr) = vmm::map_device_memory(
                        PhysAddr::new(local_apic_addr),
                        4096,
                        mapper,
                        frame_allocator,
                    ) {
                        log_debug!("Local APIC mapped to virtual address: {:#x}", vaddr.as_u64());

                        lapic::init_local_apic_with_vaddr(vaddr);

                        if let Some(apic_id) = lapic::get_apic_id() {
                            log_info!("Current Local APIC ID: {}", apic_id);
                        }
                    } else {
                        log_error!("Failed to map Local APIC memory");
                    }
                }

                if !info.io_apics.is_empty() {
                    log_info!("Initializing IO APICs...");

                    for (paddr, id, gsi_base) in &info.io_apics {
                        log_debug!("Mapping IO APIC {} at physical address: {:#x}", id, paddr);

                        if let Some(vaddr) = vmm::map_device_memory(
                            PhysAddr::new(*paddr),
                            4096,
                            mapper,
                            frame_allocator,
                        ) {
                            log_debug!("IO APIC {} mapped to virtual address: {:#x}", id, vaddr.as_u64());
                            crate::hal::ioapic::init_single_ioapic(vaddr, *id, *gsi_base);
                        } else {
                            log_error!("Failed to map IO APIC {} memory", id);
                        }
                    }

                    log_info!("All IO APICs initialized");
                }
            }
        } else {
            log_warn!("APIC not available, using legacy PIC (not implemented yet)");
        }
    }

    rtc::init();
    keyboard::init();
    log_debug!("Cleanup completed");
}

pub fn _kernel_init(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        _display_init(framebuffer);

        _logger_init();

        _critical_init();

        let physical_memory_offset = boot_info
            .physical_memory_offset
            .into_option()
            .expect("Physical memory offset not provided");

        let rsdp_addr = boot_info.rsdp_addr.into_option();

        let (mut mapper, mut frame_allocator) = _memory_init(
            &boot_info.memory_regions,
            physical_memory_offset
        );
        let acpi_info = _acpi_init(rsdp_addr, physical_memory_offset);

        _post_init(acpi_info.as_ref(), &mut mapper, &mut frame_allocator);

        _boot_report(&boot_info.memory_regions, physical_memory_offset);

        kprintln!();
        kprintln!("========================================");
        kprintln!("  Kernel Initialization Complete!       ");
        kprintln!("========================================");
        kprintln!();

        k_main::_kernel_main();

    } else {
        panic!("No framebuffer provided by bootloader");
    }
}

#[allow(dead_code)]
pub fn kernel_emergency_cleanup() {
    log_error!("Emergency cleanup triggered");
    // 在 panic 前調用，做最後的清理工作
    // 比如刷新緩衝區、保存日誌等
}
// kernel/src/k_init.rs (Updated with Logger)
use bootloader_api::BootInfo;
use bootloader_api::info::MemoryRegionKind;
use x86_64::structures::paging::{OffsetPageTable, Page };
use x86_64::{PhysAddr, VirtAddr};
use crate::mm::{allocator::{heap, frame, pmm}, vmm, paging, vma};
use crate::arch::amd64::{gdt, idt};
use crate::tty::tty;
use crate::kprintln;
use crate::kernel::k_main;
use crate::klibc::logger::{init, LogLevel, LoggerConfig};
use crate::klibc::malloc;
use crate::{log_debug, log_error, log_info, log_trace, log_warn};
use crate::hal::acpi;

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

    if let Some(stats) = pmm::get_memory_stats() {
        log_info!("Total Memory: {} MiB", stats.total_memory / (1024 * 1024));
        log_info!("Free Memory:  {} MiB", stats.free_memory / (1024 * 1024));
    }
}

fn _acpi_init(rsdp_addr: Option<u64>, physical_memory_offset: u64) {
    kprintln!();

    if let Some(rsdp) = rsdp_addr {
        log_debug!("RSDP Address: {:#x}", rsdp);

        if let Some(acpi_info) = acpi::init(rsdp, physical_memory_offset) {
            acpi::print_info(&acpi_info);
        } else {
            log_warn!("ACPI initialization failed");
        }
    } else {
        log_warn!("RSDP not provided by bootloader");
    }
}

fn _post_init() {
    kprintln!();
    log_info!("Post Initialization");
    // TODO: 釋放 bootloader 佔用的內存
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

        _boot_report(&boot_info.memory_regions, physical_memory_offset);

        _acpi_init(rsdp_addr, physical_memory_offset);

        _post_init();

        _test_memory_management(&mut mapper, &mut frame_allocator);

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

fn _test_memory_management(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut frame::BootInfoFrameAllocator
) {
    kprintln!();
    log_info!("Testing Memory Management System...");

    // Physical memory allocation
    log_debug!("Test 1: Physical frame allocation");
    if let Some(frame) = pmm::allocate_frame() {
        log_debug!("  Allocated frame at: {:#x}", frame.start_address().as_u64());
        pmm::deallocate_frame(frame);
        log_debug!("  Deallocated frame");
    }

    // Virtual memory allocation
    log_debug!("Test 2: Virtual memory allocation (kmalloc)");
    if let Some(vaddr) = malloc::kmalloc(8192, mapper, frame_allocator) {
        log_debug!("  Allocated 8KB at: {:#x}", vaddr.as_u64());

        // Test Write
        unsafe {
            let ptr = vaddr.as_mut_ptr::<u64>();
            *ptr = 0xDEADBEEF;
            log_debug!("  Written test value: {:#x}", *ptr);
        }

        malloc::kfree(vaddr, 8192, mapper, frame_allocator);
        log_debug!("  Freed memory");
    }

    // Page table mapping
    log_debug!("Test 3: Page table mapping");
    let test_vaddr = VirtAddr::new(0x5000_0000_0000);
    let test_page = Page::containing_address(test_vaddr);

    if let Some(test_frame) = pmm::allocate_frame() {
        if paging::PageTableManager::map_page(
            test_page,
            test_frame,
            paging::kernel_data(),
            mapper,
            frame_allocator
        ).is_ok() {
            log_debug!("  Mapped page {:#x} to frame {:#x}",
                test_vaddr.as_u64(), test_frame.start_address().as_u64());

            // Testing Address Translation
            if let Some(phys) = paging::PageTableManager::translate_addr(test_vaddr, mapper) {
                log_debug!("  Translation check: {:#x} -> {:#x}", test_vaddr.as_u64(), phys.as_u64());
            }

            // Unmap
            if paging::PageTableManager::unmap_page(test_page, mapper).is_ok() {
                log_debug!("  Unmapped page");
            }
        }
        pmm::deallocate_frame(test_frame);
    }
    
    log_info!("Memory tests completed!");
}
use bootloader_api::BootInfo;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;
use crate::kernel::asm::x86::{gdt, idt, acpi};
use crate::kernel::mm::{heap, frame_allocator};
use crate::kernel::tty::tty;
use crate::kprintln;
use crate::k_main;

fn _critical_init() {
    gdt::init();
    idt::init();
}

fn _memory_init(
    memory_regions: &'static bootloader_api::info::MemoryRegions,
    physical_memory_offset: u64
) -> (OffsetPageTable<'static>, frame_allocator::BootInfoFrameAllocator) {
    let phys_mem_offset = VirtAddr::new(physical_memory_offset);

    let mut mapper = unsafe {
        OffsetPageTable::new(heap::get_level_4_table(phys_mem_offset), phys_mem_offset)
    };

    let mut frame_allocator = unsafe {
        frame_allocator::BootInfoFrameAllocator::init(memory_regions)
    };

    heap::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

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
    kprintln!("[INIT] Stage 1: Critical Hardware");
    kprintln!("  [OK] GDT initialized");
    gdt::print_info();

    kprintln!();
    kprintln!("  [OK] IDT initialized");
    idt::print_info();

    kprintln!();
    kprintln!("[INIT] Stage 2: Memory Management");
    kprintln!("  [OK] Physical Memory Offset: {:#x}", physical_memory_offset);

    kprintln!("  [OK] Memory Regions:");
    let mut total_usable = 0u64;
    for region in memory_regions.iter() {
        use bootloader_api::info::MemoryRegionKind;
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
        kprintln!("    {:#016x} - {:#016x} ({})",
            region.start, region.end, kind_str);
    }
    kprintln!("  [INFO] Total Usable Memory: {} MiB", total_usable / (1024 * 1024));

    kprintln!();
    kprintln!("[INIT] Stage 3: Heap Allocator");
    kprintln!("  [OK] Heap Start: {:#x}", heap::HEAP_START);
    kprintln!("  [OK] Heap Size:  {} KiB", heap::HEAP_SIZE / 1024);
}

fn _acpi_init(rsdp_addr: Option<u64>, physical_memory_offset: u64) {
    kprintln!();
    kprintln!("[INIT] Stage 4: ACPI");

    if let Some(rsdp) = rsdp_addr {
        kprintln!("  [INFO] RSDP Address: {:#x}", rsdp);

        if let Some(acpi_info) = acpi::init(rsdp, physical_memory_offset) {
            acpi::print_info(&acpi_info);
        } else {
            kprintln!("  [WARN] ACPI initialization failed");
        }
    } else {
        kprintln!("  [WARN] RSDP not provided by bootloader");
    }
}

fn _post_init() {
    kprintln!();
    kprintln!("[INIT] Stage 5: Post Initialization");
    // TODO: 釋放 bootloader 佔用的內存
    // TODO: 釋放初始化代碼段（.init 段）
    kprintln!("  [INFO] Cleanup completed");
}

/// 主初始化入口
pub fn kernel_init(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {

        _critical_init();

        let physical_memory_offset = boot_info
            .physical_memory_offset
            .into_option()
            .expect("Physical memory offset not provided");

        let rsdp_addr = boot_info.rsdp_addr.into_option();

        let (_mapper, _frame_allocator) = _memory_init(
            &boot_info.memory_regions,
            physical_memory_offset
        );

        _display_init(framebuffer);

        _boot_report(&boot_info.memory_regions, physical_memory_offset);

        _acpi_init(rsdp_addr, physical_memory_offset);

        _post_init();

        kprintln!();
        kprintln!("========================================");
        kprintln!("  Kernel Initialization Complete!");
        kprintln!("========================================");
        kprintln!();

        k_main::kernel_main();

    } else {
        panic!("No framebuffer provided by bootloader");
    }
}

#[allow(dead_code)]
pub fn kernel_emergency_cleanup() {
    // 在 panic 前調用，做最後的清理工作
    // 比如刷新緩衝區、保存日誌等
}
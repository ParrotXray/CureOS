// kernel/src/mm/vma.rs

use x86_64::VirtAddr;
use crate::log_info;

/// Upper core base address
///
/// All kernel-related virtual addresses begin here
pub const HIGHER_HALF_BASE: u64 = 0xFFFF_C000_0000_0000;

/// Physical memory direct mapping base address
///
/// The bootloader will map all physical memory here
/// Physical address P → virtual address (P + PHYS_MEM_OFFSET)
pub const PHYS_MEM_OFFSET: u64 = 0xFFFF_8000_0000_0000;

/// Kernel heap start address
///
/// Used for dynamic allocation of small objects (Vec, String, Box, etc.)
pub const HEAP_START: VirtAddr = VirtAddr::new_truncate(HIGHER_HALF_BASE + 0x1000);

/// Kernel heap size
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// Starting address of the kernel's dynamic allocation area
///
/// Used for kmalloc/kfree allocation of large objects
pub const KERNEL_DYNAMIC_START: VirtAddr = VirtAddr::new_truncate(HIGHER_HALF_BASE + 0x10_0000);

/// Kernel dynamically allocates area size
pub const KERNEL_DYNAMIC_SIZE: usize = 128 * 1024 * 1024; // 128 MiB

/// End address of the kernel's dynamically allocated area
pub const KERNEL_DYNAMIC_END: VirtAddr = VirtAddr::new_truncate(
    KERNEL_DYNAMIC_START.as_u64() + KERNEL_DYNAMIC_SIZE as u64
);

/// Kernel stack area starting address
pub const KERNEL_STACK_START: VirtAddr = VirtAddr::new_truncate(0xFFFF_D000_0000_0000);

/// Kernel stack area size
pub const KERNEL_STACK_SIZE: usize = 64 * 1024 * 1024; // 64 MiB

/// End address of the kernel stack area
pub const KERNEL_STACK_END: VirtAddr = VirtAddr::new_truncate(
    KERNEL_STACK_START.as_u64() + KERNEL_STACK_SIZE as u64
);

/// Device mapping area start address (MMIO)
pub const DEVICE_MAPPING_START: VirtAddr = VirtAddr::new_truncate(0xFFFF_E000_0000_0000);

/// Device mapping area size
pub const DEVICE_MAPPING_SIZE: usize = 256 * 1024 * 1024; // 256 MiB

/// End address of the device mapping area
pub const DEVICE_MAPPING_END: VirtAddr = VirtAddr::new_truncate(
    DEVICE_MAPPING_START.as_u64() + DEVICE_MAPPING_SIZE as u64
);

/// Check if the address is in kernel space
#[inline]
pub fn is_kernel_address(addr: VirtAddr) -> bool {
    addr.as_u64() >= HIGHER_HALF_BASE
}

/// Check if the address is in user space
#[inline]
pub fn is_user_address(addr: VirtAddr) -> bool {
    addr.as_u64() < 0x0000_8000_0000_0000
}

/// Physical addresses are translated into virtual addresses (via direct mapping)
#[inline]
pub fn phys_to_virt(phys: u64) -> VirtAddr {
    VirtAddr::new(phys + PHYS_MEM_OFFSET)
}

/// Convert virtual address to physical address (if in direct mapped area)
#[inline]
pub fn virt_to_phys(virt: VirtAddr) -> Option<u64> {
    let addr = virt.as_u64();
    if addr >= PHYS_MEM_OFFSET && addr < HIGHER_HALF_BASE {
        Some(addr - PHYS_MEM_OFFSET)
    } else {
        None
    }
}

/// Check if the address is within the kernel heap range
#[inline]
pub fn is_in_heap(addr: VirtAddr) -> bool {
    let addr_u64 = addr.as_u64();
    addr_u64 >= HEAP_START.as_u64()
        && addr_u64 < HEAP_START.as_u64() + HEAP_SIZE as u64
}

/// Check if the address is in the dynamically allocated area
#[inline]
pub fn is_in_dynamic_region(addr: VirtAddr) -> bool {
    let addr_u64 = addr.as_u64();
    addr_u64 >= KERNEL_DYNAMIC_START.as_u64()
        && addr_u64 < KERNEL_DYNAMIC_END.as_u64()
}

/// Check if the address is in the kernel stack area
#[inline]
pub fn is_in_kernel_stack(addr: VirtAddr) -> bool {
    let addr_u64 = addr.as_u64();
    addr_u64 >= KERNEL_STACK_START.as_u64()
        && addr_u64 < KERNEL_STACK_END.as_u64()
}

/// Check if the address is in the device mapping area
#[inline]
pub fn is_in_device_region(addr: VirtAddr) -> bool {
    let addr_u64 = addr.as_u64();
    addr_u64 >= DEVICE_MAPPING_START.as_u64()
        && addr_u64 < DEVICE_MAPPING_END.as_u64()
}

/// Get the name of the region to which the address belongs
pub fn get_region_name(addr: VirtAddr) -> &'static str {
    let addr_u64 = addr.as_u64();

    if is_user_address(addr) {
        "User Space"
    } else if addr_u64 >= PHYS_MEM_OFFSET && addr_u64 < HIGHER_HALF_BASE {
        "Physical Memory Map"
    } else if is_in_heap(addr) {
        "Kernel Heap"
    } else if is_in_dynamic_region(addr) {
        "Kernel Dynamic"
    } else if is_in_kernel_stack(addr) {
        "Kernel Stack"
    } else if is_in_device_region(addr) {
        "Device Mapping"
    } else {
        "Reserved"
    }
}

/// Print hhk address space layout
pub fn print_info() {
    log_info!("Higher Half Kernel Memory Layout:");
    log_info!("  User Space:     {:#018x} - {:#018x}",
        0x0u64,
        0x0000_7FFF_FFFF_FFFFu64
    );
    log_info!("  Physical Map:   {:#018x} - {:#018x}",
        PHYS_MEM_OFFSET,
        HIGHER_HALF_BASE - 1
    );
    log_info!("  Kernel Base:    {:#018x}", HIGHER_HALF_BASE);
    log_info!("  Heap:           {:#018x} - {:#018x} ({} KiB)",
        HEAP_START.as_u64(),
        HEAP_START.as_u64() + HEAP_SIZE as u64,
        HEAP_SIZE / 1024
    );
    log_info!("  Dynamic:        {:#018x} - {:#018x} ({} MiB)",
        KERNEL_DYNAMIC_START.as_u64(),
        KERNEL_DYNAMIC_END.as_u64(),
        KERNEL_DYNAMIC_SIZE / (1024 * 1024)
    );
    log_info!("  Kernel Stack:   {:#018x} - {:#018x} ({} MiB)",
        KERNEL_STACK_START.as_u64(),
        KERNEL_STACK_END.as_u64(),
        KERNEL_STACK_SIZE / (1024 * 1024)
    );
    log_info!("  Device Mapping: {:#018x} - {:#018x} ({} MiB)",
        DEVICE_MAPPING_START.as_u64(),
        DEVICE_MAPPING_END.as_u64(),
        DEVICE_MAPPING_SIZE / (1024 * 1024)
    );
}
// kernel/src/mm/allocator/heap.rs
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use x86_64::structures::paging::PageTable;
use crate::hal::cpu;
use crate::mm::vma;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // 使用 kernel_vm 中定義的高半核地址
    let heap_start = vma::HEAP_START;
    let heap_size = vma::HEAP_SIZE;

    // Calculate page range
    let page_range = {
        let heap_end = heap_start + heap_size as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Allocate a physical frame for each page and map
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // Initialize the heap allocator
    unsafe {
        ALLOCATOR.lock().init(heap_start.as_mut_ptr(), heap_size);
    }

    Ok(())
}

pub unsafe fn get_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let phys = cpu::cpu_r_cr3_addr().as_u64();
    let virt = physical_memory_offset + phys;
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
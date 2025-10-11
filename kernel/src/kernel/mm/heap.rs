// kernel/src/mm/heap.rs
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use x86_64::structures::paging::PageTable;
use crate::hal::cpu;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// 堆的起始虛擬位址
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// 堆的大小（100 KiB）
pub const HEAP_SIZE: usize = 100 * 1024;

/// Initialize the heap
///
/// This function will:
/// 1. Allocate physical frames for the heap
/// 2. Map these frames in the page table
/// 3. Initialize the heap allocator
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // Calculate the page range required for the heap
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Allocate a physical frame for each page and map it
    for page in page_range {
        // Allocate a physical frame
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        // Set page flags: exists + writable
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        // Map page to frame
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // Initialize the heap allocator
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

pub unsafe fn get_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let virt = physical_memory_offset + cpu::cpu_r_cr3();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

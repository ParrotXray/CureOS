use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;
use crate::mm::vmm;
use crate::mm::vmm::VMM;

/// Allocate and map kernel memory
pub fn kmalloc<A>(
    size: usize,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut A,
) -> Option<VirtAddr>
where
    A: FrameAllocator<Size4KiB>,
{
    if size == 0 {
        return None;
    }

    let page_count = (size + 4095) / 4096;

    let vaddr = vmm::VMM.lock().allocate_pages(page_count)?;

    let start_page = Page::containing_address(vaddr);

    for i in 0..page_count {
        let page = start_page + i as u64;
        let frame = frame_allocator.allocate_frame()?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .ok()?
                .flush();
        }
    }

    Some(vaddr)
}

/// Release kernel memory
///
/// # Important
/// You must pass in the correct size; it should be the same size as when using kmalloc.
pub fn kfree<A>(
    addr: VirtAddr,
    size: usize,
    mapper: &mut OffsetPageTable,
    _frame_allocator: &mut A,
)
where
    A: FrameAllocator<Size4KiB>,
{
    if size == 0 {
        return;
    }

    let page_count = (size + 4095) / 4096;
    let start_page: Page::<Size4KiB> = Page::containing_address(addr);

    // Unmap page tables
    for i in 0..page_count {
        let page = start_page + i as u64;

        if let Ok((frame, flush)) = mapper.unmap(page) {
            flush.flush();
            // Release the frame back to PMM
            crate::mm::allocator::pmm::deallocate_frame(frame);
        }
    }

    // Freeing virtual address space
    VMM.lock().deallocate_pages(addr, page_count);
}
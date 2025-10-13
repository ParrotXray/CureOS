// kernel/src/mm/paging.rs

use x86_64::{
    structures::paging::{
        Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
        Mapper, FrameAllocator, OffsetPageTable, PageTableIndex,
        mapper::{MapToError, UnmapError},
    },
    VirtAddr, PhysAddr,
};
use x86_64::structures::paging::Translate;
use crate::log_error;
use crate::mm::vma;
/// Page table manager
pub struct PageTableManager;

impl PageTableManager {
    /// Map a single page
    ///
    /// # Parameters
    /// * `page` - Virtual page
    /// * `frame` - Physical page frame
    /// * `flags` - Page table flags
    /// * `mapper` - Page table mapper
    /// * `frame_allocator` - Physical page frame allocator
    pub fn map_page<A>(
        page: Page,
        frame: PhysFrame,
        flags: PageTableFlags,
        mapper: &mut OffsetPageTable,
        frame_allocator: &mut A,
    ) -> Result<(), MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB>,
    {
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)?
                .flush();
        }
        Ok(())
    }

    /// Unmap a single page
    ///
    /// # Parameters
    /// * `page` - The virtual page to unmap
    /// * `mapper` - The page table mapper
    pub fn unmap_page(
        page: Page,
        mapper: &mut OffsetPageTable,
    ) -> Result<PhysFrame, UnmapError> {
        let (frame, flush) = mapper.unmap(page)?;
        flush.flush();
        Ok(frame)
    }

    /// Map memory region
    ///
    /// # Parameters
    /// * `virt_start` - Virtual start address
    /// * `phys_start` - Physical start address
    /// * `size` - Size (bytes)
    /// * `flags` - Page table flags
    /// * `mapper` - Page table mapper
    /// * `frame_allocator` - Physical page frame allocator
    pub fn map_region<A>(
        virt_start: VirtAddr,
        phys_start: PhysAddr,
        size: usize,
        flags: PageTableFlags,
        mapper: &mut OffsetPageTable,
        frame_allocator: &mut A,
    ) -> Result<(), MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB>,
    {
        let page_count = (size + 4095) / 4096;

        for i in 0..page_count {
            let page_addr = virt_start + (i * 4096) as u64;
            let frame_addr = phys_start + (i * 4096) as u64;

            let page = Page::containing_address(page_addr);
            let frame = PhysFrame::containing_address(frame_addr);

            Self::map_page(page, frame, flags, mapper, frame_allocator)?;
        }

        Ok(())
    }

    /// Unmap memory region
    ///
    /// # Parameters
    /// * `virt_start` - Virtual start address
    /// * `size` - Size (bytes)
    /// * `mapper` - Page table mapper
    pub fn unmap_region(
        virt_start: VirtAddr,
        size: usize,
        mapper: &mut OffsetPageTable,
    ) -> Result<(), UnmapError> {
        let page_count = (size + 4095) / 4096;

        for i in 0..page_count {
            let page_addr = virt_start + (i * 4096) as u64;
            let page = Page::containing_address(page_addr);

            Self::unmap_page(page, mapper)?;
        }

        Ok(())
    }

    /// Modify page table flags
    ///
    /// # Parameters
    /// * `page` - Virtual page
    /// * `flags` - New page table flags
    /// * `mapper` - Page table mapper
    pub fn update_flags(
        page: Page,
        flags: PageTableFlags,
        mapper: &mut OffsetPageTable,
    ) -> Result<(), ()> {
        unsafe {
            if let Ok(entry) = mapper.update_flags(page, flags) {
                entry.flush();
                Ok(())
            } else {
                Err(())
            }
        }
    }

    /// Query the physical address corresponding to a virtual address
    ///
    /// # Parameters
    /// * `virt_addr` - Virtual address
    /// * `mapper` - Page table mapper
    pub fn translate_addr(
        virt_addr: VirtAddr,
        mapper: &OffsetPageTable,
    ) -> Option<PhysAddr> {
        mapper.translate_addr(virt_addr)
    }

    /// Check if the page is mapped
    ///
    /// # Parameters
    /// * `page` - Virtual page
    /// * `mapper` - Page table mapper
    pub fn is_mapped(
        page: Page,
        mapper: &OffsetPageTable,
    ) -> bool {
        mapper.translate_page(page).is_ok()
    }

    /// Create a new page table (for the process)
    ///
    /// # Parameters
    /// * `frame_allocator` - Physical page frame allocator
    pub fn create_page_table<A>(
        frame_allocator: &mut A,
    ) -> Option<PhysFrame>
    where
        A: FrameAllocator<Size4KiB>,
    {

        let frame = frame_allocator.allocate_frame()?;

        let phys_addr = frame.start_address();
        let virt_addr = vma::phys_to_virt(phys_addr.as_u64());

        unsafe {
            let page_table = &mut *(virt_addr.as_mut_ptr::<PageTable>());
            page_table.zero();
        }

        Some(frame)
    }

    /// Get the page table entry
    /// Get the page table entry
    pub fn get_entry<'a>(
        page: Page,
        mapper: &'a OffsetPageTable,
    ) -> Option<&'a x86_64::structures::paging::page_table::PageTableEntry> {

        unsafe {
            mapper.translate_page(page).ok()?;

            let l4_table = mapper.level_4_table();

            let p4_index = page.p4_index();
            let p3_index = page.p3_index();
            let p2_index = page.p2_index();
            let p1_index = page.p1_index();

            // Level 4 -> Level 3
            let l4_entry = &l4_table[p4_index];
            let l3_table_addr = vma::phys_to_virt(l4_entry.addr().as_u64());
            let l3_table = &*(l3_table_addr.as_ptr::<PageTable>());

            // Level 3 -> Level 2
            let l3_entry = &l3_table[p3_index];
            if l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                return Some(l3_entry); // 1GB huge page
            }
            let l2_table_addr = vma::phys_to_virt(l3_entry.addr().as_u64());
            let l2_table = &*(l2_table_addr.as_ptr::<PageTable>());

            // Level 2 -> Level 1
            let l2_entry = &l2_table[p2_index];
            if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                return Some(l2_entry); // 2MB huge page
            }
            let l1_table_addr = vma::phys_to_virt(l2_entry.addr().as_u64());
            let l1_table = &*(l1_table_addr.as_ptr::<PageTable>());

            // return Level 1 entry
            Some(&l1_table[p1_index])
        }
    }
}

/// Kernel code segment flag (executable, not writable)
pub const KERNEL_CODE: PageTableFlags = PageTableFlags::PRESENT;

/// Kernel data segment flag (non-executable, writable)
pub fn kernel_data() -> PageTableFlags {
    PageTableFlags::PRESENT | PageTableFlags::WRITABLE
}

/// User code segment flags (executable, non-writable, user accessible)
pub fn user_code() -> PageTableFlags {
    PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE
}

/// User data segment flags (non-executable, writable, user accessible)
pub fn user_data() -> PageTableFlags {
    PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE
}

/// Device mapping flag (non-cacheable)
pub fn device_memory() -> PageTableFlags {
    PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_CACHE
}

/// Page table error handling
pub fn handle_page_fault(virt_addr: VirtAddr, error_code: u64) {
    // TODO: 實現頁面錯誤處理
    // - 檢查 VMA (Virtual Memory Area) 是否合法
    // - 按需分頁: if !present && valid_vma { allocate_page() }
    // - COW: if write && cow_page { copy_page() }
    log_error!("Page fault at {:#x}, error code: {:#x}", virt_addr.as_u64(), error_code);
}
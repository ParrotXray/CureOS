// kernel/src/mm/vmm.rs

use x86_64::{
    structures::paging::{
        Page, PageTableFlags, PhysFrame, Size4KiB,
        Mapper, FrameAllocator, OffsetPageTable,
    },
    VirtAddr, PhysAddr,
};
use spin::Mutex;
use alloc::collections::BTreeMap;
use crate::log_info;
use crate::mm::vma;

/// Memory block information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MemoryBlock {
    /// Starting address
    start: VirtAddr,
    /// Size (pages)
    page_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct VmmStats {
    pub next_vaddr: VirtAddr,
    pub end_vaddr: VirtAddr,
    pub allocated_pages: usize,
    pub free_pages: usize,
    pub used_from_new: usize,
    pub allocated_blocks_count: usize,
    pub free_blocks_count: usize,
}

impl VmmStats {
    pub fn print(&self) {
        log_info!("Allocated: {} pages ({} KiB) in {} blocks",
            self.allocated_pages,
            self.allocated_pages * 4,
            self.allocated_blocks_count
        );
        log_info!("Free: {} pages ({} KiB) in {} blocks",
            self.free_pages,
            self.free_pages * 4,
            self.free_blocks_count
        );
        log_info!("New Usage: {} pages ({} KiB)",
            self.used_from_new,
            self.used_from_new * 4
        );
    }
}

pub static VMM: Mutex<VirtualMemoryManager> = Mutex::new(VirtualMemoryManager::new());

impl MemoryBlock {
    fn new(start: VirtAddr, page_count: usize) -> Self {
        Self { start, page_count }
    }

    fn size_bytes(&self) -> usize {
        self.page_count * 4096
    }

    fn end(&self) -> VirtAddr {
        self.start + self.size_bytes() as u64
    }
}

/// Virtual memory manager (with free list)
pub struct VirtualMemoryManager {
    /// The next allocatable virtual address (for unused areas)
    next_vaddr: VirtAddr,
    /// End address of the dynamically allocated area
    end_vaddr: VirtAddr,
    /// Free blockchain table (key: starting address, value: size)
    free_blocks: BTreeMap<u64, MemoryBlock>,
    /// Allocated block (key: start address, value: size)
    allocated_blocks: BTreeMap<u64, MemoryBlock>,
}

impl VirtualMemoryManager {
    pub const fn new() -> Self {
        Self {
            next_vaddr: vma::KERNEL_DYNAMIC_START,
            end_vaddr: vma::KERNEL_DYNAMIC_END,
            free_blocks: BTreeMap::new(),
            allocated_blocks: BTreeMap::new(),
        }
    }

    /// Allocate contiguous virtual memory pages
    ///
    /// Strategy:
    /// 1. First find the most suitable block in the free list (First Fit)
    /// 2. If no suitable block is found, allocate from an unused area
    pub fn allocate_pages(&mut self, count: usize) -> Option<VirtAddr> {
        if count == 0 {
            return None;
        }

        // 策略 1: 從空閒鏈表中找（First Fit）
        if let Some(addr) = self.allocate_from_free_list(count) {
            return Some(addr);
        }

        // 策略 2: 從未使用區域分配
        self.allocate_from_new_region(count)
    }

    /// Allocate from the free list
    fn allocate_from_free_list(&mut self, count: usize) -> Option<VirtAddr> {
        // Find the first block that is large enough
        let mut found_block: Option<(u64, MemoryBlock)> = None;

        for (&addr, &block) in self.free_blocks.iter() {
            if block.page_count >= count {
                found_block = Some((addr, block));
                break;
            }
        }

        if let Some((addr, block)) = found_block {
            // Remove this free block
            self.free_blocks.remove(&addr);

            let alloc_addr = block.start;
            let alloc_block = MemoryBlock::new(alloc_addr, count);

            // Record Assigned
            self.allocated_blocks.insert(alloc_addr.as_u64(), alloc_block);

            // If there is free space, add it back to the free list
            if block.page_count > count {
                let remaining_start = alloc_addr + (count * 4096) as u64;
                let remaining_count = block.page_count - count;
                let remaining_block = MemoryBlock::new(remaining_start, remaining_count);

                self.free_blocks.insert(remaining_start.as_u64(), remaining_block);
            }

            return Some(alloc_addr);
        }

        None
    }

    /// Unused zone allocation
    fn allocate_from_new_region(&mut self, count: usize) -> Option<VirtAddr> {
        let size = count * 4096;
        let start = self.next_vaddr;
        let end = start + size as u64;

        if end > self.end_vaddr {
            return None;
        }

        let block = MemoryBlock::new(start, count);
        self.allocated_blocks.insert(start.as_u64(), block);

        self.next_vaddr = end;
        Some(start)
    }

    /// Release virtual memory page
    ///
    /// Strategy:
    /// 1. Remove from allocated list
    /// 2. Add to free list
    /// 3. Attempt to merge adjacent free blocks
    pub fn deallocate_pages(&mut self, addr: VirtAddr, count: usize) {
        let addr_u64 = addr.as_u64();

        // Check whether it has actually been allocated
        if let Some(&block) = self.allocated_blocks.get(&addr_u64) {
            // Verify size matches
            if block.page_count != count {
                crate::log_warn!(
                    "VMM: deallocate size mismatch at {:#x}: expected {}, got {}",
                    addr_u64, block.page_count, count
                );
            }

            // Remove from allocated list
            self.allocated_blocks.remove(&addr_u64);

            // Add to free list
            let free_block = MemoryBlock::new(addr, count);
            self.free_blocks.insert(addr_u64, free_block);

            // Try to merge adjacent blocks
            self.coalesce_free_blocks(addr_u64);
        } else {
            crate::log_warn!("VMM: attempt to free unallocated memory at {:#x}", addr_u64);
        }
    }

    /// Merge adjacent free blocks
    fn coalesce_free_blocks(&mut self, addr: u64) {
        let current_block = match self.free_blocks.get(&addr) {
            Some(&block) => block,
            None => return,
        };

        // Try to merge with the following block
        let next_addr = current_block.end().as_u64();
        if let Some(&next_block) = self.free_blocks.get(&next_addr) {
            // merge
            self.free_blocks.remove(&addr);
            self.free_blocks.remove(&next_addr);

            let merged = MemoryBlock::new(
                current_block.start,
                current_block.page_count + next_block.page_count
            );

            self.free_blocks.insert(addr, merged);

            crate::log_debug!("VMM: merged blocks at {:#x}", addr);
        }

        // Try merging with the previous block
        // Find the previous block
        let mut prev_addr_opt: Option<u64> = None;
        for (&prev_addr, &prev_block) in self.free_blocks.iter() {
            if prev_block.end().as_u64() == addr {
                prev_addr_opt = Some(prev_addr);
                break;
            }
        }

        if let Some(prev_addr) = prev_addr_opt {
            let prev_block = self.free_blocks[&prev_addr];
            let current_block = self.free_blocks[&addr];

            self.free_blocks.remove(&prev_addr);
            self.free_blocks.remove(&addr);

            let merged = MemoryBlock::new(
                prev_block.start,
                prev_block.page_count + current_block.page_count
            );

            self.free_blocks.insert(prev_addr, merged);

            crate::log_debug!("VMM: merged with previous block at {:#x}", prev_addr);
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> VmmStats {
        let mut allocated_pages = 0;
        for block in self.allocated_blocks.values() {
            allocated_pages += block.page_count;
        }

        let mut free_pages = 0;
        for block in self.free_blocks.values() {
            free_pages += block.page_count;
        }

        let used_from_new =
            ((self.next_vaddr.as_u64() - vma::KERNEL_DYNAMIC_START.as_u64()) / 4096) as usize;

        VmmStats {
            next_vaddr: self.next_vaddr,
            end_vaddr: self.end_vaddr,
            allocated_pages,
            free_pages,
            used_from_new,
            allocated_blocks_count: self.allocated_blocks.len(),
            free_blocks_count: self.free_blocks.len(),
        }
    }
}

/// Map device memory (MMIO)
pub fn map_device_memory<A>(
    phys_addr: PhysAddr,
    size: usize,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut A,
) -> Option<VirtAddr>
where
    A: FrameAllocator<Size4KiB>,
{
    let page_count = (size + 4095) / 4096;
    let vaddr = VMM.lock().allocate_pages(page_count)?;

    let start_page: Page::<Size4KiB> = Page::containing_address(vaddr);
    let start_frame: PhysFrame::<Size4KiB> = PhysFrame::containing_address(phys_addr);

    for i in 0..page_count {
        let page = start_page + i as u64;
        let frame = PhysFrame::containing_address(
            start_frame.start_address() + (i * 4096) as u64
        );

        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .ok()?
                .flush();
        }
    }

    Some(vaddr)
}

/// Get virtual memory statistics
pub fn get_vmm_stats() -> VmmStats {
    VMM.lock().get_stats()
}

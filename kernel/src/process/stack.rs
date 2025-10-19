// kernel/src/process/stack.rs
use core::num::NonZeroUsize;
use corosensei::stack::{Stack, StackPointer, STACK_ALIGNMENT};
use crate::mm::{KERNEL_MAPPER, FRAME_ALLOCATOR};
use crate::mm::allocator::pmm;
use x86_64::{VirtAddr, structures::paging::{Page, PageTableFlags, Size4KiB, Mapper}};
use crate::{log_debug, log_error};

const PROCESS_STACK_SIZE: usize = 64 * 1024; // 64 KiB

pub struct ProcessStack {
    base: StackPointer,
    limit: StackPointer,
    vaddr: VirtAddr,
    page_count: usize,
}

impl ProcessStack {
    pub fn new() -> Option<Self> {
        let page_count = (PROCESS_STACK_SIZE + 4095) / 4096;

        // 從 VMM 分配虛擬地址
        let vaddr = crate::mm::vmm::VMM.lock().allocate_pages(page_count)?;

        log_debug!("Allocating process stack at {:#x}, {} pages", vaddr.as_u64(), page_count);

        // 映射所有頁面
        let mapper = KERNEL_MAPPER.get()?;
        let allocator = FRAME_ALLOCATOR.get()?;

        for i in 0..page_count {
            let page_vaddr = vaddr + (i * 4096) as u64;
            let page = Page::<Size4KiB>::containing_address(page_vaddr);

            // 分配物理頁面
            let frame = match pmm::allocate_frame() {
                Some(f) => f,
                None => {
                    log_error!("Failed to allocate physical frame for stack page {}", i);
                    Self::cleanup_mapped_pages(vaddr, i);
                    crate::mm::vmm::VMM.lock().deallocate_pages(vaddr, page_count);
                    return None;
                }
            };

            // 映射頁面
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            let mut mapper_guard = mapper.lock();
            let mut alloc_guard = allocator.lock();

            unsafe {
                match mapper_guard.map_to(page, frame, flags, &mut **alloc_guard) {
                    Ok(flush) => {
                        flush.flush();
                    }
                    Err(e) => {
                        log_error!("Failed to map stack page {}: {:?}", i, e);
                        pmm::deallocate_frame(frame);
                        drop(mapper_guard);
                        drop(alloc_guard);
                        Self::cleanup_mapped_pages(vaddr, i);
                        crate::mm::vmm::VMM.lock().deallocate_pages(vaddr, page_count);
                        return None;
                    }
                }
            }

            log_debug!("Mapped stack page {} at {:#x} -> frame {:#x}",
                      i, page_vaddr.as_u64(), frame.start_address().as_u64());
        }

        // 計算對齊的 stack base 和 limit
        let base_addr = vaddr.as_u64() + PROCESS_STACK_SIZE as u64;
        let limit_addr = vaddr.as_u64();

        let base_aligned = base_addr & !(STACK_ALIGNMENT as u64 - 1);
        let limit_aligned = (limit_addr + STACK_ALIGNMENT as u64 - 1) & !(STACK_ALIGNMENT as u64 - 1);

        Some(Self {
            base: NonZeroUsize::new(base_aligned as usize)?,
            limit: NonZeroUsize::new(limit_aligned as usize)?,
            vaddr,
            page_count,
        })
    }

    /// 清理已映射的頁面
    fn cleanup_mapped_pages(vaddr: VirtAddr, count: usize) {
        if let Some(mapper) = KERNEL_MAPPER.get() {
            let mut mapper_guard = mapper.lock();

            for i in 0..count {
                let page_vaddr = vaddr + (i * 4096) as u64;
                let page = Page::<Size4KiB>::containing_address(page_vaddr);

                // 明確指定類型
                if let Ok((frame, flush)) = mapper_guard.unmap(page) {
                    flush.flush();
                    pmm::deallocate_frame(frame);
                }
            }
        }
    }
}

impl Drop for ProcessStack {
    fn drop(&mut self) {
        log_debug!("Dropping process stack at {:#x}", self.vaddr.as_u64());

        // 取消映射並釋放物理頁面
        if let Some(mapper) = KERNEL_MAPPER.get() {
            let mut mapper_guard = mapper.lock();

            for i in 0..self.page_count {
                let page_vaddr = self.vaddr + (i * 4096) as u64;
                let page = Page::<Size4KiB>::containing_address(page_vaddr);

                // 明確指定類型參數
                if let Ok((frame, flush)) = mapper_guard.unmap(page) {
                    flush.flush();
                    pmm::deallocate_frame(frame);
                    log_debug!("Unmapped and freed stack page {}", i);
                }
            }
        }

        // 釋放虛擬地址
        crate::mm::vmm::VMM.lock().deallocate_pages(self.vaddr, self.page_count);
    }
}

unsafe impl Stack for ProcessStack {
    fn base(&self) -> StackPointer {
        self.base
    }

    fn limit(&self) -> StackPointer {
        self.limit
    }
}
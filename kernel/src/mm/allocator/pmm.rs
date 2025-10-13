// kernel/src/mm/allocator/pmm.rs
// Physical Memory Manager with NULL pointer protection

use x86_64::{
    structures::paging::{PhysFrame, Size4KiB, PageSize},
    PhysAddr,
};
use spin::Mutex;
use crate::{log_debug, log_error, log_info, log_trace, log_warn};

/// Physical memory allocator status
pub struct PhysicalMemoryManager {
    /// Bitmap, each bit represents a 4KB page frame
    bitmap: &'static mut [u8],
    /// Total page frames
    total_frames: usize,
    /// Number of allocated page frames
    allocated_frames: usize,
    /// Memory start address
    memory_start: PhysAddr,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_frames: usize,
    pub allocated_frames: usize,
    pub free_frames: usize,
    pub total_memory: usize,
    pub used_memory: usize,
    pub free_memory: usize,
}

/// Global Physical Memory Manager
static PMM: Mutex<Option<PhysicalMemoryManager>> = Mutex::new(None);

impl PhysicalMemoryManager {
    /// Initialize the physical memory manager
    ///
    /// # Safety measures
    /// - Automatically mark page frame 0x0 as used (to prevent null pointers)
    /// - Mark the first 1MB of real mode memory as used
    ///
    /// # Parameters
    /// * `memory_start` - The starting address of available memory
    /// * `memory_size` - The size of available memory (bytes)
    /// * `bitmap_addr` - The address where the bitmap is stored
    pub unsafe fn new(
        memory_start: PhysAddr,
        memory_size: usize,
        bitmap_addr: *mut u8,
    ) -> Self {
        let total_frames = memory_size / 4096;
        let bitmap_size = (total_frames + 7) / 8;

        let bitmap = core::slice::from_raw_parts_mut(bitmap_addr, bitmap_size);
        bitmap.fill(0);

        let mut pmm = Self {
            bitmap,
            total_frames,
            allocated_frames: 0,
            memory_start,
        };

        pmm.mark_reserved_regions();

        pmm
    }

    /// Mark system reserved area
    fn mark_reserved_regions(&mut self) {
        // Mark the system reserved area NULL pointer protection: Mark the first page frame (0x0000 - 0x0FFF)
        if self.memory_start.as_u64() == 0 {
            self.mark_frame_at_index(0);
            log_debug!("Marked frame 0 (NULL pointer protection)");
        }

        // - IVT (Interrupt Vector Table): 0x00000 - 0x003FF
        // - BDA (BIOS Data Area): 0x00400 - 0x004FF
        // - EBDA (Extended BIOS Data Area): 0x80000 - 0x9FFFF
        // - Video memory: 0xA0000 - 0xBFFFF
        // - BIOS ROM: 0xF0000 - 0xFFFFF
        let low_memory_end = 0x100000u64;  // 1MB

        if self.memory_start.as_u64() < low_memory_end {
            let frames_to_reserve = ((low_memory_end - self.memory_start.as_u64()) / 4096) as usize;
            for frame_idx in 0..frames_to_reserve.min(self.total_frames) {
                self.mark_frame_at_index(frame_idx);
            }
            log_debug!("Reserved low memory: {} frames (0x0 - 0x100000)", frames_to_reserve);
        }
    }

    /// Directly mark the page frame of the specified index
    fn mark_frame_at_index(&mut self, frame_idx: usize) {
        if frame_idx < self.total_frames {
            let byte_idx = frame_idx / 8;
            let bit = frame_idx % 8;

            if byte_idx < self.bitmap.len() {
                if (self.bitmap[byte_idx] & (1 << bit)) == 0 {
                    self.bitmap[byte_idx] |= 1 << bit;
                    self.allocated_frames += 1;
                }
            }
        }
    }

    /// Allocate a physical page frame
    ///
    /// # Guarantees
    /// - Never return a page frame at address 0x0
    /// - Never return a page frame in the lower 1MB region
    ///
    /// # Panics
    /// Panics if allocating to page frame address 0 (this shouldn't happen)
    pub fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for (byte_idx, byte) in self.bitmap.iter_mut().enumerate() {
            if *byte != 0xFF {
                for bit in 0..8 {
                    if (*byte & (1 << bit)) == 0 {
                        *byte |= 1 << bit;
                        self.allocated_frames += 1;

                        let frame_idx = byte_idx * 8 + bit;
                        let frame_addr = self.memory_start + (frame_idx * 4096) as u64;

                        if frame_addr.as_u64() == 0 {
                            panic!("FATAL: PMM allocated NULL frame at index {}! This is a critical bug!", frame_idx);
                        }

                        if frame_addr.as_u64() < 0x100000 {
                            panic!("FATAL: PMM allocated reserved low memory frame at {:#x}! This is a critical bug!", frame_addr.as_u64());
                        }

                        return Some(PhysFrame::containing_address(frame_addr));
                    }
                }
            }
        }
        None
    }

    /// Free a physical page frame
    ///
    /// # Safety check
    /// - Do not free a page frame at address 0
    /// - Do not free a page frame below 1MB
    pub fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let frame_addr = frame.start_address();

        if frame_addr.as_u64() < 0x100000 {
            log_warn!("Attempt to free reserved frame at {:#x} - ignored", frame_addr.as_u64());
            return;
        }

        let offset = (frame_addr.as_u64() - self.memory_start.as_u64()) as usize;
        let frame_idx = offset / 4096;

        let byte_idx = frame_idx / 8;
        let bit = frame_idx % 8;

        if byte_idx < self.bitmap.len() {
            self.bitmap[byte_idx] &= !(1 << bit);
            self.allocated_frames = self.allocated_frames.saturating_sub(1);
        }
    }

    /// Mark page frame as used
    pub fn mark_frame_used(&mut self, frame: PhysFrame<Size4KiB>) {
        let frame_addr = frame.start_address();
        let offset = (frame_addr.as_u64() - self.memory_start.as_u64()) as usize;
        let frame_idx = offset / 4096;

        self.mark_frame_at_index(frame_idx);
    }

    /// Mark memory area as used
    pub fn mark_region_used(&mut self, start: PhysAddr, size: usize) {
        if size == 0 {
            return;
        }

        let start_frame = PhysFrame::<Size4KiB>::containing_address(start);
        let end_frame = PhysFrame::<Size4KiB>::containing_address(start + size as u64 - 1u64);

        for frame_addr in (start_frame.start_address().as_u64()..=end_frame.start_address().as_u64())
            .step_by(4096)
        {
            let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(frame_addr));
            self.mark_frame_used(frame);
        }
    }

    /// Check if the page frame is available
    pub fn is_frame_free(&self, frame: PhysFrame<Size4KiB>) -> bool {
        let frame_addr = frame.start_address();

        if frame_addr.as_u64() < 0x100000 {
            return false;
        }

        let offset = (frame_addr.as_u64() - self.memory_start.as_u64()) as usize;
        let frame_idx = offset / 4096;

        let byte_idx = frame_idx / 8;
        let bit = frame_idx % 8;

        if byte_idx < self.bitmap.len() {
            (self.bitmap[byte_idx] & (1 << bit)) == 0
        } else {
            false
        }
    }

    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_frames: self.total_frames,
            allocated_frames: self.allocated_frames,
            free_frames: self.total_frames - self.allocated_frames,
            total_memory: self.total_frames * 4096,
            used_memory: self.allocated_frames * 4096,
            free_memory: (self.total_frames - self.allocated_frames) * 4096,
        }
    }
}

/// Initialize the global physical memory manager
pub unsafe fn init_pmm(
    memory_start: PhysAddr,
    memory_size: usize,
    bitmap_addr: *mut u8,
) {
    let pmm = PhysicalMemoryManager::new(memory_start, memory_size, bitmap_addr);
    *PMM.lock() = Some(pmm);
}

/// Allocate physical page frames
///
/// # Guarantee
/// Never return a page frame at address 0x0 or in the lower 1MB range
///
/// # Panics
/// Panic if the internal allocator returns an invalid page frame
pub fn allocate_frame() -> Option<PhysFrame<Size4KiB>> {
    let frame = PMM.lock().as_mut()?.allocate_frame()?;

    if frame.start_address().as_u64() == 0 {
        panic!("CRITICAL: PMM returned NULL frame!");
    }

    if frame.start_address().as_u64() < 0x100000 {
        panic!("CRITICAL: PMM returned reserved low memory frame at {:#x}!",
               frame.start_address().as_u64());
    }

    Some(frame)
}

/// Release physical page frame
pub fn deallocate_frame(frame: PhysFrame<Size4KiB>) {
    if let Some(pmm) = PMM.lock().as_mut() {
        pmm.deallocate_frame(frame);
    }
}

/// Mark page frame as used
pub fn mark_frame_used(frame: PhysFrame<Size4KiB>) {
    if let Some(pmm) = PMM.lock().as_mut() {
        pmm.mark_frame_used(frame);
    }
}

/// Mark memory area as used
pub fn mark_region_used(start: PhysAddr, size: usize) {
    if let Some(pmm) = PMM.lock().as_mut() {
        pmm.mark_region_used(start, size);
    }
}

/// Check if the page frame is available
pub fn is_frame_free(frame: PhysFrame<Size4KiB>) -> bool {
    PMM.lock()
        .as_ref()
        .map(|pmm| pmm.is_frame_free(frame))
        .unwrap_or(false)
}

/// Get memory statistics
pub fn get_memory_stats() -> Option<MemoryStats> {
    PMM.lock().as_ref().map(|pmm| pmm.get_stats())
}
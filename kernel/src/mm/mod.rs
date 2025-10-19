pub mod allocator;
pub mod paging;
pub mod vmm;
pub mod vma;

use x86_64::structures::paging::OffsetPageTable;
use spin::{Mutex, Once};

// 使用 Once 來保證只初始化一次
pub static KERNEL_MAPPER: Once<Mutex<&'static mut OffsetPageTable<'static>>> = Once::new();
pub static FRAME_ALLOCATOR: Once<Mutex<&'static mut allocator::frame::BootInfoFrameAllocator>> = Once::new();

/// 初始化全局 mapper 和 allocator
pub unsafe fn init_globals(
    mapper: &'static mut OffsetPageTable<'static>,
    allocator: &'static mut allocator::frame::BootInfoFrameAllocator,
) {
    KERNEL_MAPPER.call_once(|| Mutex::new(mapper));
    FRAME_ALLOCATOR.call_once(|| Mutex::new(allocator));
}

/// 獲取全局 mapper（輔助函數）
pub fn with_mapper<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut OffsetPageTable<'static>) -> R,
{
    KERNEL_MAPPER.get().map(|mapper| {
        let mut guard = mapper.lock();
        f(*guard)
    })
}

/// 獲取全局 frame allocator（輔助函數）
pub fn with_frame_allocator<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut allocator::frame::BootInfoFrameAllocator) -> R,
{
    FRAME_ALLOCATOR.get().map(|allocator| {
        let mut guard = allocator.lock();
        f(*guard)
    })
}
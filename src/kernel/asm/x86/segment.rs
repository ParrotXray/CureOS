use x86::segmentation::SegmentSelector;
use x86::Ring;

pub const NULL_SELECTOR: SegmentSelector = SegmentSelector::new(0, Ring::Ring0);
pub const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);
pub const KERNEL_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(2, Ring::Ring0);
pub const USER_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(3, Ring::Ring3);
pub const USER_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(4, Ring::Ring3);

#[no_mangle]
pub static NULL_SEL: u16 = NULL_SELECTOR.bits();
#[no_mangle]
pub static KERNEL_CODE_SEL: u16 = KERNEL_CODE_SELECTOR.bits();
#[no_mangle]
pub static KERNEL_DATA_SEL: u16 = KERNEL_DATA_SELECTOR.bits();
#[no_mangle]
pub static USER_CODE_SEL: u16 = USER_CODE_SELECTOR.bits();
#[no_mangle]
pub static USER_DATA_SEL: u16 = USER_DATA_SELECTOR.bits();
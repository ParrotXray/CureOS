use core::mem;
use core::ptr;
use x86::segmentation::{SegmentSelector, Descriptor};
use x86::Ring;
use x86::irq::{self, PageFaultError, EXCEPTIONS};
use crate::{print, println};

// #[no_mangle]
// pub extern "C" fn divide_error_handler() {
//     let ex = &EXCEPTIONS[irq::DIVIDE_ERROR_VECTOR as usize];
//     println!("CPU Exception: {} ({})", ex.mnemonic, ex.description);
//     println!("Type: {}, Source: {}", ex.irqtype, ex.source);
//     loop {}
// }
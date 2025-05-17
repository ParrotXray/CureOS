pub mod kernel;
pub mod tty;
pub mod asm;
pub mod mm;
pub mod common;

// 重新導出由 boot.S 調用的入口點
// pub use kernel::{_kernel_init, _kernel_main};
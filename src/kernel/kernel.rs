// src/kernel/kernel.rs
use core::arch::asm;

// boot.S 調用的入口點
#[no_mangle]
pub extern "C" fn _kernel_init() {
    // TODO: 加載 GDT
    // TODO: 加載 IDT  
    // TODO: 啟用分頁
}

#[no_mangle]
pub extern "C" fn _kernel_main(_multiboot_info: u32) {
    
    unsafe {
        loop {
            asm!("hlt");
        }
    }
}
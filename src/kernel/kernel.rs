// src/kernel/kernel.rs
use core::arch::asm;
use crate::kernel::tty::tty;

// boot.S 調用的入口點
#[no_mangle]
pub extern "C" fn _kernel_init() {
    // TODO: 加載 GDT
    // TODO: 加載 IDT  
    // TODO: 啟用分頁
    tty::tty_init(tty::VGA_BUFFER_PADDR);
    tty::tty_set_theme(tty::VGA_COLOR_WHITE, tty::VGA_COLOR_BLACK);
}

#[no_mangle]
pub extern "C" fn _kernel_post_init() {

}

#[no_mangle]
pub extern "C" fn _kernel_main() {

    tty::tty_put_str("Loading....!\n");

    // tty::tty_clear();

    tty::tty_put_str("Welcome to Cure OS!\n");

    unsafe {
        loop {
            asm!("hlt");
        }
    }
}
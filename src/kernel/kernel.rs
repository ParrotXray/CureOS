// src/kernel/kernel.rs
use core::arch::asm;
use crate::kernel::tty::tty;
use crate::hal::cpu;
use crate::{print, println};
use crate::kernel::asm::x86::gdt;

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

    println!("Loading....!");
    
    // tty::tty_clear();
    
    println!("Welcome to Cure OS!");

    println!("{0} + {1} = {0}", 1, 2);

    let name = "111";
    let age = 25;
    println!("{}, {}", name, age);

    let value = 42;
    println!("10: {}, 16: {:x}", value, value);
    
    // unsafe {
    //     core::arch::asm!(
    //         "movl $0, %eax",
    //         "movl $0, %ebx",
    //         "divl %ebx",
    //         options(att_syntax)
    //     );
    // }


    loop {
        cpu::cpu_idle();
    }
}
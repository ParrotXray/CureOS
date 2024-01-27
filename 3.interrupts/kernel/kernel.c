#include <system/tty/tty.h>
#include <system/arch/gdt.h>
#include <system/arch/idt.h>

void _kernel_init() {
    //TODO
    _init_gdt();
    _init_idt();
}

void _kernel_main(void* info_table) {

    // remove the warning
    (void)info_table;
    //TODO
    tty_set_theme(VGA_COLOR_GREEN, VGA_COLOR_BLACK);
    tty_put_str("Hello kernel world!\nThis is second line.");

    // return 1 / 0;
    __asm__("int $0");
}
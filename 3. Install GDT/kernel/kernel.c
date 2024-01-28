#include <system/tty/tty.h>
#include <system/arch/gdt.h>

void _kernel_init() {
    //TODO
    _init_gdt();
}

void _kernel_main(void* info_table) {
    //TODO
    tty_set_theme(VGA_COLOR_GREEN, VGA_COLOR_BLACK);
    tty_put_str("Hello kernel world!\nThis is second line.");
}
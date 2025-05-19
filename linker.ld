ENTRY(start_)

SECTIONS {
    . = 0x100000;

    .hhk_init_text BLOCK(4K): {
        * (.multiboot) /* boot.o (.multiboot) */
        * (.text)
    }

    .hhk_init_bss BLOCK(4K): {
        * (COMMON)
        * (.bss)
    }

    .hhk_init_data BLOCK(4K): {
        * (.data)
    }

    .hhk_init_rodata BLOCK(4k): {
        * (.rodata)
    }

     __init_hhk_end = ALIGN(4K);
}
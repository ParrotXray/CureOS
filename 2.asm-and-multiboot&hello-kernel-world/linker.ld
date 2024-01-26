ENTRY(start_)

SECTIONS {
    . = 0x100000;

    .text BLOCK(4K): {
        * (.multiboot) /* boot.o (.multiboot) */
        * (.text)
    }

    .bss BLOCK(4K): {
        * (COMMON)
        * (.bss)
    }

    .data BLOCK(4K): {
        * (.data)
    }

    .rodata BLOCK(4k): {
        * (.rodata)
    }
}
ENTRY(start_)

PHYS_BASE = 0x100000;
VIRT_BASE = 0xC0000000;

SECTIONS {
    . = PHYS_BASE;

    .hhk_init_text BLOCK(4K) : {
        * (.multiboot)
        build/obj/src/arch/x86/*.o (.hhk_init)
        build/obj/src/arch/x86/boot.S.o (.text)
    }

    .hhk_init_data BLOCK(4K) : {
        build/obj/src/arch/x86/*.o (.data)
    }

    .hhk_init_rodata BLOCK(4K) : {
        build/obj/src/arch/x86/*.o (.rodata)
    }

    .hhk_init_bss BLOCK(4K) : {
        build/obj/src/arch/x86/*.o (.bss)
        build/obj/src/arch/x86/*.o (COMMON)
    }

    . = ALIGN(4K);
    __init_hhk_end = .;

    . += VIRT_BASE;

    .text : AT (ADDR(.text) - VIRT_BASE) {
        __kernel_start = .;
        build/src/obj/kernel/*.o (.text)
        build/src/obj/hal/*.o (.text)
    }

    .data : AT (ADDR(.data) - VIRT_BASE) {
        build/src/obj/kernel/*.o (.data)
        build/src/obj/hal/*.o (.data)
    }

    .rodata : AT (ADDR(.rodata) - VIRT_BASE) {
        build/src/obj/kernel/*.o (.rodata)
        build/src/obj/hal/*.o (.rodata)
    }

    .bss : AT (ADDR(.bss) - VIRT_BASE) {
        build/src/obj/kernel/*.o (.bss)
        build/src/obj/hal/*.o (.bss)
    }

    .kpg : AT (ADDR(.kpg) - VIRT_BASE) {
        build/obj/src/arch/x86/*.o (.kpg)
    }

    . = ALIGN(4K);
    __kernel_end = .;
    __heap_start = .;
}

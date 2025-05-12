#include "multiboot.h"

.section .multiboot
    /* Define type as 32 bit */
    .long MB_MAGIC
    .long MB_ALIGNED_4K_MEM_MAP
    .long CHECKSUM(MB_ALIGNED_4K_MEM_MAP)

/* .bss: stack */
.section .bss
    /* According to System V ABI, the stack must be aligned at 16 bytes boundary.*/ 
    .align 16 /* Not divisible by 16, move forward until divisible */ 
    stack_bottom:
        .skip 32708, 0
    stack_top:

/* .text: put executable code */
.section .text 
    .global start_
    start_:
        cli
        
        movl $stack_top, %esp
        /*
            TODO: kernel init
                Load GDT
                Load IDT
                Enable paging  
        */
        call _kernel_init

        /* install gdt */
        call _load_gdt
        // call _init_gdt
        // subl $0x6, %esp
        // movl $_GDT, 2(%esp)
        // movw _GDT_LIMIT, %ax
        // movw %ax, (%esp)
        // lgdt (%esp)
        // addl $0x6, %esp

        call _load_idt

        /* processing register (CS, ES, SS, DS, FS, GS) */
        movw KERNEL_DATA_SEL, %cx /* 0x10 in binary representation is 0001 0000. If you right-shift this number by 3 bits, you will get 0000 0010, so the index is 2. */
        movw %cx, %es
        movw %cx, %ds
        movw %cx, %fs
        movw %cx, %gs
        movw %cx, %ss

        /* Change CS:IP */
        pushw KERNEL_CODE_SEL /* 0x08 in binary representation is 0000 1000. If you right-shift this number by 3 bits, you will get 0000 0001, so the index is 1. */
        pushl $_after_gdt
        retf

    _after_gdt: 
        pushl %ebx
        call _kernel_main
        
    j_:
        hlt
        jmp j_
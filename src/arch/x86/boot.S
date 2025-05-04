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
        movl $stack_top, %esp
        /*
            TODO: kernel init
                Load GDT
                Load IDT
                Enable paging  
        */
        call _kernel_init

        pushl %ebx
        call _kernel_main

        cli
    j_:
        hlt
        jmp j_
#define __ASM__ 1

#include "multiboot.h"

#define MB_FLAGS    MULTIBOOT_MEMORY_INFO | MULTIBOOT_PAGE_ALIGN
#define KPG_SIZE    24*1024

.section .multiboot
    .long MULTIBOOT_MAGIC
    .long MB_FLAGS
    .long CHECKSUM(MB_FLAGS)

/* .bss: stack */
.section .bss
    .global mb_info
    .align 16
    /* Reserve space for Multiboot info struct */
    mb_info:
        .skip 4096
    /* According to System V ABI, the stack address must be 16-byte aligned */
    /* This is just a temporary stack. In _hhk_init we will initialize the kernel-specific stack */
    stack_bottom:
        .skip 16318, 0
    stack_top:

/* 
    1 page directory, 
    5 page tables:
        1. Mapping reserved area and hhk_init
        2-5. Remapping the kernels
*/

.section .kpg
    .global _k_ptd
    _k_ptd:
        .skip KPG_SIZE, 0

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

        call _load_idt
        call _kernel_main
        
    j_:
        hlt
        jmp j_
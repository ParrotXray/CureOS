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

.section .hhk_init
    .global start_
    .type start_, @function     /* Optional, this just give the 
                                 * linker more knowledge about the label 
                                 */
    start_: 
        cli

        movl $stack_top, %esp

        subl $16, %esp

        movl $mb_info, 4(%esp)
        movl %ebx, (%esp)
        call _save_multiboot_info

        movl $(KPG_SIZE), 4(%esp)
        movl $(_k_ptd - 0xC0000000), (%esp)
        call _hhk_init

        movl (%esp), %eax
        andl $0xfffff000, %eax 
        movl %eax, %cr3

        movl %cr0, %eax
        orl $0x80000000, %eax
        movl %eax, %cr0

        addl $16, %esp

        /* 进入高半核！ */
        pushl $hhk_entry_
        ret
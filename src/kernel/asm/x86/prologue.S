/* High core entry point - 0xC0000000 */

/* .text: put executable code */
.section .text 
    .global hhk_entry_
    hhk_entry_:
        
        subl $16, %esp

        /* install gdt */
        call _load_gdt

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
        movl $mb_info, (%esp)
        call _kernel_init
        
        movl $0xffbffff0, %esp

        call _load_idt
        call _kernel_main
        
    j_:
        hlt
        jmp j_
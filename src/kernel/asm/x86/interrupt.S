.macro isr_template vector, no_error_code=1
    .global _asm_isr\vector
    .type _asm_isr\vector, @function
    _asm_isr\vector:
        .if \no_error_code
            pushl $0x0
        .endif
        pushl $\vector
        jmp interrupt_wrapper
.endm

.section .text
    isr_template 0
    isr_template 1
    isr_template 2
    isr_template 3
    isr_template 4
    isr_template 5
    isr_template 6
    isr_template 7
    isr_template 8, 0
    isr_template 9
    isr_template 10, 0
    isr_template 11, 0
    isr_template 12, 0
    isr_template 13, 0
    isr_template 14, 0
    isr_template 15
    isr_template 16
    isr_template 17, 0
    isr_template 18
    isr_template 19
    isr_template 20
    isr_template 21
    isr_template 22
    isr_template 23
    isr_template 24
    isr_template 25
    isr_template 26
    isr_template 27
    isr_template 28
    isr_template 29
    isr_template 30
    isr_template 31

    interrupt_wrapper:

        movl %esp, %eax
        andl $0xfffffff0, %esp
        subl $16, %esp
        movl %eax, (%esp)

        call interrupt_handler
        pop %eax
        movl %eax, %esp
        addl $8, %esp

        iret
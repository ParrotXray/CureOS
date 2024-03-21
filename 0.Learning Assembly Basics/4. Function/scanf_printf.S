.section .data
    integer_print_format: .asciz "%d-%p\n"
    integer_format: .asciz "%d"
    string_print_format: .asciz "%s-%p\n"
    string_format: .asciz "%s"

.section .bss
    buffer: .space 100

.global _start

.section .text
    _start:
        leal (%ebx), %eax
        pushl %eax
        pushl $integer_format
        call scanf
        addl $4, %esp

        pushl (%ebx)
        pushl (%ebx)
        pushl $integer_print_format
        call printf
        addl $8, %esp

        push $buffer
        push $string_format
        call scanf
        addl $4, %esp

        push $buffer
        push $buffer
        push $string_print_format
        call printf
        addl $8, %esp

        pushl $0 
        call exit 

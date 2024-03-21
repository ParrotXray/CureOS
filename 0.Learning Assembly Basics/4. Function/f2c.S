.section .data
    answer_format: .asciz "Converted temperature in Celsius: %d\n"
    input_format: .asciz "Enter the temperature in Fahrenheit: "
    scanf_format: .asciz "%d"

.global _start
.section .text
_start:
    pushl $input_format
    call printf
    addl $4, %esp

    leal (%ebx), %eax # roload %ebx address
    pushl %eax # using (%ebx)
    pushl $scanf_format
    call scanf
    addl $8, %esp

    pushl (%ebx)
    call f2c
    addl $4, %esp

    pushl %eax
    pushl $answer_format
    call printf
    addl $8, %esp

    pushl $0
    call exit

.type f2c, @function
f2c:
    pushl %ebp
    movl %esp, %ebp
    subl $8, %esp
    
    movl 8(%ebp), %eax 
    subl $32, %eax # eax - 32
    imull $5, %eax # eax * 5

    movl %eax, -4(%ebp)
    cmpl $0, %eax
    jg skip_neg

    neg %eax
skip_neg:
    movl $9, %ecx   
    xorl %edx, %edx 
    idivl %ecx   
    cmpl $0, -4(%ebp)
    jge return
    neg %eax
return:
    leave /* movl %ebp, %esp 
             popl %ebp */
    ret

.section .data

.section .text

.globl _start
.globl square

_start:
    pushl $5
    call square
    addl $4, %esp
    movl %eax, %ebx
    jmp exit

exit:
    movl $1, %eax
    int $0x80

.type square, @function
square:
    pushl %ebp
    movl %esp, %ebp
    subl $4, %esp
    movl 8(%ebp), %ebx
    movl %ebx, -4(%ebp)

    movl -4(%ebp), %eax
    imull %ebx, %eax
    movl %eax, -4(%ebp)

square_end:
    movl -4(%ebp), %eax
    movl %ebp, %esp
    popl %ebp
    ret

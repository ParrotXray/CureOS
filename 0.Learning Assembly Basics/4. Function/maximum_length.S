.section .data

data_item:
    .long 3, 67, 34, 225, 45, 75, 54, 34, 44, 33, 22, 11, 66, 12
data_item_length:
    .long 14

.section .text

.globl _start 
_start:
    movl $0, %edi 
    movl data_item_length, %ecx
    movl data_item(, %edi, 4), %eax
    movl %eax, %ebx

start_loop:
    cmpl $0, %ecx
    je loop_exit

    incl %edi 
    decl %ecx
    movl data_item(, %edi, 4), %eax
    cmpl %ebx, %eax
    jle start_loop
    movl %eax, %ebx

    jmp start_loop

loop_exit:
    movl $1, %eax
    int $0x80

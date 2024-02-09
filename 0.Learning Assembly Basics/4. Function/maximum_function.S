.section .data

data_item:
    .long 3, 67, 34, 225, 45, 75, 54, 34, 44, 33, 22, 11, 66, 12
data_item_length:
    .long 14

.section .text

.globl _start 
.globl maximum

_start:
    pushl $data_item # 2
    pushl data_item_length # 1

    call maximum
    addl $8, %esp
    movl %eax, %ebx
    jmp exit

exit:
    movl $1, %eax
    int $0x80

.type maximum, @function
maximum:
    pushl %ebp
    movl %esp, %ebp
    subl $4, %esp

    movl 8(%ebp), %ecx
    movl 12(%ebp), %ebx
    
    movl $0, %edi 
    movl (%ebx, %edi, 4), %eax
    movl %eax, -4(%ebp)

maximum_start_loop:
    cmpl $0, %ecx
    je loop_exit
    incl %edi 
    decl %ecx
    movl (%ebx, %edi, 4), %eax
    cmpl -4(%ebp), %eax    
    jle maximum_start_loop 

    movl %eax, -4(%ebp)     
    jmp maximum_start_loop

loop_exit:
    movl -4(%ebp), %eax
    movl %ebp, %esp
    popl %ebp
    ret
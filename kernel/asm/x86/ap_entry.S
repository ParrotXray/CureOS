#define __ASM__ 1
#include <system/constants.h>

.section .text
    .global ap_entry
    .type ap_entry, @function

ap_entry:
    # 設置棧指針
    # 每個CPU使用不同的棧空間，這裡先使用臨時棧
    movl $ap_stack_top, %esp
    
    # 設置頁目錄寄存器 (CR3)
    # 獲取BSP當前使用的頁目錄
    movl $get_pd, %eax
    call *%eax
    
    # 使用相同的頁目錄
    movl %eax, %esp
    subl $4, %esp
    pushl %eax
    movl $set_pd, %eax
    call *%eax
    
    # 設置段寄存器
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    
    # 呼叫C函數初始化AP
    call smp_ap_ready
    
    # 不應該返回，但如果返回了，進入無限循環
1:  hlt
    jmp 1b

.section .bss
    # 為AP提供臨時棧空間，每個AP應該在初始化過程中獲取自己的棧
    .align 16
ap_stack:
    .space 4096
ap_stack_top:
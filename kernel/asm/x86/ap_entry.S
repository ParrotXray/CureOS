#define __ASM__ 1
#include <system/constants.h>

.section .text
    .global ap_entry
    .type ap_entry, @function

ap_entry:
    # 關閉中斷
    cli
    
    # 設置臨時堆疊
    movl $ap_stack_top, %esp
    
    # 初始化分頁 - 使用與BSP相同的頁目錄
    movl $get_pd, %eax
    call *%eax
    movl %eax, %cr3
    
    # 開啟分頁
    movl %cr0, %eax
    orl $0x80000000, %eax
    movl %eax, %cr0
    
    # 設置段寄存器
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    
    # 跳轉到高半核
    pushl $ap_high_entry
    ret

ap_high_entry:
    # 通知BSP，AP已就緒
    call smp_ap_ready
    
    # 啟用中斷
    sti
    
    # 進入空閒循環
1:  hlt
    jmp 1b

.section .bss
    # 為AP提供臨時堆疊空間
    .align 16
ap_stack:
    .space 4096
ap_stack_top:
#ifndef _SYSTEM_INTERRUPTS_H
#define _SYSTEM_INTERRUPTS_H

#include <stdint.h>

#pragma pack(push, 1)
typedef struct {
    unsigned int vector;
    unsigned int err_code;
    unsigned int eip;
    unsigned short cs;
    unsigned int eflags;
} isr_param;
#pragma pack(pop)

// 定義 IRQ 處理程序類型
typedef void (*irq_handler_t)(isr_param* param);

// 外部 ISR 處理程序
void _asm_isr0();
void _asm_isr1();
void _asm_isr2();
void _asm_isr3();
void _asm_isr4();
void _asm_isr5();
void _asm_isr6();
void _asm_isr7();
void _asm_isr8();
void _asm_isr9();
void _asm_isr10();
void _asm_isr11();
void _asm_isr12();
void _asm_isr13();
void _asm_isr14();
void _asm_isr15();
void _asm_isr16();
void _asm_isr17();
void _asm_isr18();
void _asm_isr19();
void _asm_isr20();
void _asm_isr21();
void _asm_isr22();
void _asm_isr23();
void _asm_isr24();
void _asm_isr25();
void _asm_isr26();
void _asm_isr27();
void _asm_isr28();
void _asm_isr29();
void _asm_isr30();
void _asm_isr31();

// 外部 IRQ 處理程序
void _asm_irq0();
void _asm_irq1();
void _asm_irq2();
void _asm_irq3();
void _asm_irq4();
void _asm_irq5();
void _asm_irq6();
void _asm_irq7();
void _asm_irq8();
void _asm_irq9();
void _asm_irq10();
void _asm_irq11();
void _asm_irq12();
void _asm_irq13();
void _asm_irq14();
void _asm_irq15();

// 系統呼叫處理程序
void _asm_syscall();

// APIC 相關處理程序
void _asm_apic_timer();
void _asm_apic_error();
void _asm_apic_spurious();

// 主中斷處理函數
void interrupt_handler(isr_param* param);

/**
 * @brief 註冊 IRQ 處理程序
 * 
 * @param irq IRQ 編號
 * @param handler 處理程序函數指針
 * @return int 成功返回 1，失敗返回 0
 */
int register_irq_handler(int irq, irq_handler_t handler);

/**
 * @brief 取消註冊 IRQ 處理程序
 * 
 * @param irq IRQ 編號
 * @return int 成功返回 1，失敗返回 0
 */
int unregister_irq_handler(int irq);

#endif /* _SYSTEM_INTERRUPTS_H */
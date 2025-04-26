#include <arch/x86/idt.h>
#include <arch/x86/interrupts.h>
#include <arch/x86/types.h>
#include <stdint.h>
#include <libc/stdio.h>

// 擴展到 256 個中斷向量
#define IDT_ENTRY 256

uint64_t _idt[IDT_ENTRY];
uint16_t _idt_limit = sizeof(_idt) - 1;

// 外部宣告的 ISR 處理程序
extern void _asm_isr0();
extern void _asm_isr1();
extern void _asm_isr2();
extern void _asm_isr3();
extern void _asm_isr4();
extern void _asm_isr5();
extern void _asm_isr6();
extern void _asm_isr7();
extern void _asm_isr8();
extern void _asm_isr9();
extern void _asm_isr10();
extern void _asm_isr11();
extern void _asm_isr12();
extern void _asm_isr13();
extern void _asm_isr14();
extern void _asm_isr15();
extern void _asm_isr16();
extern void _asm_isr17();
extern void _asm_isr18();
extern void _asm_isr19();
extern void _asm_isr20();
extern void _asm_isr21();
extern void _asm_isr22();
extern void _asm_isr23();
extern void _asm_isr24();
extern void _asm_isr25();
extern void _asm_isr26();
extern void _asm_isr27();
extern void _asm_isr28();
extern void _asm_isr29();
extern void _asm_isr30();
extern void _asm_isr31();

// IRQ 處理程序
extern void _asm_irq0();
extern void _asm_irq1();
extern void _asm_irq2();
extern void _asm_irq3();
extern void _asm_irq4();
extern void _asm_irq5();
extern void _asm_irq6();
extern void _asm_irq7();
extern void _asm_irq8();
extern void _asm_irq9();
extern void _asm_irq10();
extern void _asm_irq11();
extern void _asm_irq12();
extern void _asm_irq13();
extern void _asm_irq14();
extern void _asm_irq15();

// 系統呼叫處理程序
extern void _asm_syscall();

// APIC 中斷處理程序
extern void _asm_apic_timer();
extern void _asm_apic_error();
extern void _asm_apic_spurious();

// 建立 ISR 處理程序指針陣列
void (*_asm_isr_handlers[32])() = {
    _asm_isr0, _asm_isr1, _asm_isr2, _asm_isr3,
    _asm_isr4, _asm_isr5, _asm_isr6, _asm_isr7,
    _asm_isr8, _asm_isr9, _asm_isr10, _asm_isr11,
    _asm_isr12, _asm_isr13, _asm_isr14, _asm_isr15,
    _asm_isr16, _asm_isr17, _asm_isr18, _asm_isr19,
    _asm_isr20, _asm_isr21, _asm_isr22, _asm_isr23,
    _asm_isr24, _asm_isr25, _asm_isr26, _asm_isr27,
    _asm_isr28, _asm_isr29, _asm_isr30, _asm_isr31
};

// 建立 IRQ 處理程序指針陣列
void (*_asm_irq_handlers[16])() = {
    _asm_irq0, _asm_irq1, _asm_irq2, _asm_irq3,
    _asm_irq4, _asm_irq5, _asm_irq6, _asm_irq7,
    _asm_irq8, _asm_irq9, _asm_irq10, _asm_irq11,
    _asm_irq12, _asm_irq13, _asm_irq14, _asm_irq15
};

void _set_idt_entry(uint32_t vector, uint16_t seg_selector, void (*isr)(), uint8_t dpl) {
    uintptr_t offset = (uintptr_t)isr;
    _idt[vector] = (offset & 0xffff0000) | IDT_ATTR(dpl);
    _idt[vector] <<= 32;
    _idt[vector] |= (seg_selector << 16) | (offset & 0x0000ffff);
}

void _init_idt() {
    // 初始化 CPU 異常處理 (0-31)
    for (int i = 0; i < 32; i++) {
        _set_idt_entry(i, 0x08, _asm_isr_handlers[i], 0);
    }
    
    // 初始化 IRQ 處理 (32-47)
    for (int i = 0; i < 16; i++) {
        _set_idt_entry(i + 32, 0x08, _asm_irq_handlers[i], 0);
    }
    
    // 系統呼叫 (通常使用 0x80)
    _set_idt_entry(0x80, 0x08, _asm_syscall, 3);  // DPL=3 允許使用者模式呼叫
    
    // APIC 中斷
    _set_idt_entry(0x40, 0x08, _asm_apic_timer, 0);    // APIC Timer
    _set_idt_entry(0xFE, 0x08, _asm_apic_error, 0);    // APIC Error
    _set_idt_entry(0xFF, 0x08, _asm_apic_spurious, 0); // APIC Spurious
}
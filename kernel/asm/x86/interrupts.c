#include <arch/x86/interrupts.h>
#include <system/apic/apic.h>
#include <libc/stdio.h>

// 前向宣告
static void handle_exception(isr_param* param);
static void handle_irq(isr_param* param);
static void handle_syscall(isr_param* param);
static void handle_apic_timer(isr_param* param);
static void handle_apic_error(isr_param* param);
static void handle_apic_spurious(isr_param* param);

// 註冊 IRQ 處理程序的陣列
static irq_handler_t irq_handlers[16] = {0};

// 異常處理程序
void isr0(isr_param* param) {
    tty_clear();
    tty_put_str("!!\tPANIC\t!!");
    tty_put_str("\nDivision Error Exception (#DE)\n");
    printf("EIP: 0x%x, CS: 0x%x, EFLAGS: 0x%x\n", 
           param->eip, param->cs, param->eflags);
    while(1); // 掛起系統
}

void isr1(isr_param* param) {
    printf("Debug Exception (#DB) at EIP: 0x%x\n", param->eip);
}

void isr2(isr_param* param) {
    printf("NMI Interrupt at EIP: 0x%x\n", param->eip);
}

void isr3(isr_param* param) {
    printf("Breakpoint Exception (#BP) at EIP: 0x%x\n", param->eip);
}

void isr4(isr_param* param) {
    printf("Overflow Exception (#OF) at EIP: 0x%x\n", param->eip);
}

void isr5(isr_param* param) {
    printf("BOUND Range Exceeded Exception (#BR) at EIP: 0x%x\n", param->eip);
}

void isr6(isr_param* param) {
    printf("Invalid Opcode Exception (#UD) at EIP: 0x%x\n", param->eip);
    while(1); // 掛起系統
}

void isr7(isr_param* param) {
    printf("Device Not Available Exception (#NM) at EIP: 0x%x\n", param->eip);
}

void isr8(isr_param* param) {
    printf("Double Fault Exception (#DF) at EIP: 0x%x, Error Code: 0x%x\n", 
           param->eip, param->err_code);
    while(1); // 掛起系統
}

void isr9(isr_param* param) {
    printf("Coprocessor Segment Overrun at EIP: 0x%x\n", param->eip);
}

void isr10(isr_param* param) {
    printf("Invalid TSS Exception (#TS) at EIP: 0x%x, Error Code: 0x%x\n", 
           param->eip, param->err_code);
}

void isr11(isr_param* param) {
    printf("Segment Not Present (#NP) at EIP: 0x%x, Error Code: 0x%x\n", 
           param->eip, param->err_code);
}

void isr12(isr_param* param) {
    printf("Stack Fault Exception (#SS) at EIP: 0x%x, Error Code: 0x%x\n", 
           param->eip, param->err_code);
}

void isr13(isr_param* param) {
    printf("General Protection Exception (#GP) at EIP: 0x%x, Error Code: 0x%x\n", 
           param->eip, param->err_code);
    while(1); // 掛起系統
}

void isr14(isr_param* param) {
    // 獲取引起頁錯誤的地址
    uint32_t fault_addr;
    __asm__ volatile("movl %%cr2, %0" : "=r"(fault_addr));
    
    printf("Page Fault Exception (#PF) at EIP: 0x%x\n", param->eip);
    printf("Fault Address: 0x%x, Error Code: 0x%x\n", fault_addr, param->err_code);
    
    // 解析錯誤碼
    printf("Error details: %s, %s, %s%s%s\n",
           (param->err_code & 0x1) ? "Page Protection Violation" : "Non-present Page",
           (param->err_code & 0x2) ? "Write" : "Read",
           (param->err_code & 0x4) ? "User Mode" : "Supervisor Mode",
           (param->err_code & 0x8) ? ", Reserved Bit Set" : "",
           (param->err_code & 0x10) ? ", Instruction Fetch" : "");
    
    while(1); // 掛起系統
}

// 其餘異常處理程序，這裡簡化處理
void isr15(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr16(isr_param* param) { printf("FPU Error (#MF) at EIP: 0x%x\n", param->eip); }
void isr17(isr_param* param) { printf("Alignment Check Exception (#AC) at EIP: 0x%x\n", param->eip); }
void isr18(isr_param* param) { printf("Machine Check Exception (#MC) at EIP: 0x%x\n", param->eip); }
void isr19(isr_param* param) { printf("SIMD FP Exception (#XF) at EIP: 0x%x\n", param->eip); }
void isr20(isr_param* param) { printf("Virtualization Exception (#VE) at EIP: 0x%x\n", param->eip); }
void isr21(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr22(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr23(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr24(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr25(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr26(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr27(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr28(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr29(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr30(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }
void isr31(isr_param* param) { printf("Reserved Exception at EIP: 0x%x\n", param->eip); }

// 主中斷處理函數
void interrupt_handler(isr_param* param) {
    // 根據中斷向量號分派到不同的處理程序
    if (param->vector < 32) {
        // CPU 異常
        handle_exception(param);
    } else if (param->vector >= 32 && param->vector < 48) {
        // IRQ
        handle_irq(param);
    } else if (param->vector == 0x80) {
        // 系統呼叫
        handle_syscall(param);
    } else if (param->vector == 0x40) {
        // APIC 計時器
        handle_apic_timer(param);
    } else if (param->vector == 0xFE) {
        // APIC 錯誤
        handle_apic_error(param);
    } else if (param->vector == 0xFF) {
        // APIC 虛假中斷
        handle_apic_spurious(param);
    }
}

// 異常處理分派函數
static void handle_exception(isr_param* param) {
    // 分派到相應的 ISR 處理程序
    switch (param->vector) {
        case 0: isr0(param); break;
        case 1: isr1(param); break;
        case 2: isr2(param); break;
        case 3: isr3(param); break;
        case 4: isr4(param); break;
        case 5: isr5(param); break;
        case 6: isr6(param); break;
        case 7: isr7(param); break;
        case 8: isr8(param); break;
        case 9: isr9(param); break;
        case 10: isr10(param); break;
        case 11: isr11(param); break;
        case 12: isr12(param); break;
        case 13: isr13(param); break;
        case 14: isr14(param); break;
        case 15: isr15(param); break;
        case 16: isr16(param); break;
        case 17: isr17(param); break;
        case 18: isr18(param); break;
        case 19: isr19(param); break;
        case 20: isr20(param); break;
        case 21: isr21(param); break;
        case 22: isr22(param); break;
        case 23: isr23(param); break;
        case 24: isr24(param); break;
        case 25: isr25(param); break;
        case 26: isr26(param); break;
        case 27: isr27(param); break;
        case 28: isr28(param); break;
        case 29: isr29(param); break;
        case 30: isr30(param); break;
        case 31: isr31(param); break;
    }
}

// IRQ 處理函數
static void handle_irq(isr_param* param) {
    int irq_num = param->vector - 32;
    
    // 呼叫註冊的 IRQ 處理程序
    if (irq_handlers[irq_num]) {
        irq_handlers[irq_num](param);
    } else {
        printf("Unhandled IRQ %d\n", irq_num);
    }
    
    // 發送 EOI 信號
    if (acpi_is_supported()) {
        // 使用 APIC
        apic_send_eoi();
    } else {
        // 使用傳統 PIC
        if (irq_num >= 8) {
            // 發送 EOI 到從 PIC
            io_port_wb(0xA0, 0x20);
        }
        // 發送 EOI 到主 PIC
        io_port_wb(0x20, 0x20);
    }
}

// 系統呼叫處理函數
static void handle_syscall(isr_param* param) {
    // 這裡實現系統呼叫處理邏輯
    printf("System call not implemented yet\n");
}

// APIC 計時器中斷處理函數
static void handle_apic_timer(isr_param* param) {
    // 計時器中斷處理邏輯 (如時間片輪轉)
    static int tick_count = 0;
    
    tick_count++;
    if (tick_count % 100 == 0) {
        printf("APIC Timer tick: %d\n", tick_count);
    }
    
    // 發送 EOI
    apic_send_eoi();
}

// APIC 錯誤中斷處理函數
static void handle_apic_error(isr_param* param) {
    // 讀取 APIC 錯誤狀態寄存器
    uint32_t esr = apic_read(APIC_ESR);
    printf("APIC Error: 0x%x\n", esr);
    
    // 清除錯誤狀態
    apic_write(APIC_ESR, 0);
    
    // 發送 EOI
    apic_send_eoi();
}

// APIC 虛假中斷處理函數
static void handle_apic_spurious(isr_param* param) {
    printf("APIC Spurious Interrupt\n");
    // 對於虛假中斷，不需要發送 EOI
}

/**
 * @brief 註冊 IRQ 處理程序
 * 
 * @param irq IRQ 編號
 * @param handler 處理程序函數指針
 * @return int 成功返回 1，失敗返回 0
 */
int register_irq_handler(int irq, irq_handler_t handler) {
    if (irq < 0 || irq >= 16) {
        return 0;
    }
    
    irq_handlers[irq] = handler;
    return 1;
}

/**
 * @brief 取消註冊 IRQ 處理程序
 * 
 * @param irq IRQ 編號
 * @return int 成功返回 1，失敗返回 0
 */
int unregister_irq_handler(int irq) {
    if (irq < 0 || irq >= 16) {
        return 0;
    }
    
    irq_handlers[irq] = NULL;
    return 1;
}
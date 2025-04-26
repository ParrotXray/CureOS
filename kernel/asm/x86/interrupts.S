.macro isr_template vector, no_error_code = 1
    .global _asm_isr\vector
    .type _asm_isr\vector, @function
    _asm_isr\vector:
        .if \no_error_code
            pushl $0x0
        .endif
        pushl $\vector
        jmp interrupt_wrapper
.endm

.macro irq_template irqnum
    .global _asm_irq\irqnum
    .type _asm_irq\irqnum, @function
    _asm_irq\irqnum:
        pushl $0x0
        pushl $(32 + \irqnum)
        jmp interrupt_wrapper
.endm

.section .text
    # 處理 CPU 異常 (0-31)
    isr_template 0
    isr_template 1
    isr_template 2
    isr_template 3
    isr_template 4
    isr_template 5
    isr_template 6
    isr_template 7
    isr_template 8, 0  # 雙重錯誤有錯誤碼
    isr_template 9
    isr_template 10, 0 # 無效 TSS 有錯誤碼
    isr_template 11, 0 # 段不存在有錯誤碼
    isr_template 12, 0 # 堆疊段錯誤有錯誤碼
    isr_template 13, 0 # 一般保護錯誤有錯誤碼
    isr_template 14, 0 # 頁錯誤有錯誤碼
    isr_template 15
    isr_template 16
    isr_template 17, 0 # 對齊檢查有錯誤碼
    isr_template 18
    isr_template 19
    isr_template 20
    isr_template 21
    isr_template 22
    isr_template 23
    isr_template 24
    isr_template 25
    isr_template 26
    isr_template 27
    isr_template 28
    isr_template 29
    isr_template 30
    isr_template 31
    
    # 處理 IRQ (0-15)
    irq_template 0
    irq_template 1
    irq_template 2
    irq_template 3
    irq_template 4
    irq_template 5
    irq_template 6
    irq_template 7
    irq_template 8
    irq_template 9
    irq_template 10
    irq_template 11
    irq_template 12
    irq_template 13
    irq_template 14
    irq_template 15
    
    # 系統呼叫
    .global _asm_syscall
    .type _asm_syscall, @function
    _asm_syscall:
        pushl $0x0
        pushl $0x80
        jmp interrupt_wrapper
    
    # APIC 計時器中斷
    .global _asm_apic_timer
    .type _asm_apic_timer, @function
    _asm_apic_timer:
        pushl $0x0
        pushl $0x40
        jmp interrupt_wrapper
    
    # APIC 錯誤中斷
    .global _asm_apic_error
    .type _asm_apic_error, @function
    _asm_apic_error:
        pushl $0x0
        pushl $0xFE
        jmp interrupt_wrapper
    
    # APIC 虛假中斷
    .global _asm_apic_spurious
    .type _asm_apic_spurious, @function
    _asm_apic_spurious:
        pushl $0x0
        pushl $0xFF
        jmp interrupt_wrapper
    
    interrupt_wrapper:
        movl %esp, %eax
        andl $0xfffffff0, %esp
        subl $16, %esp
        movl %eax, (%esp)

        call interrupt_handler
        pop %eax
        movl %eax, %esp
        addl $8, %esp

        iret
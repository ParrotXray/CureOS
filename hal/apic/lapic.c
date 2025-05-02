#include <hal/apic/lapic.h>
#include <hal/io.h>
#include <system/mm/vmm.h>
#include <system/mm/page.h>
#include <libc/stdio.h>

// 本地APIC映射的虛擬地址
static uintptr_t lapic_base_vaddr = 0;

// CPU功能檢測
static int cpu_has_apic() {
    uint32_t eax, ebx, ecx, edx;
    
    // 使用CPUID檢測APIC支持
    __asm__ volatile("cpuid" 
                     : "=a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx) 
                     : "a"(1));
    
    // EDX的第9位(bit 9)表示APIC支持
    return (edx & (1 << 9)) != 0;
}

// 在MSR中啟用APIC
static void enable_apic_in_msr() {
    uint32_t lo, hi;
    
    // 讀取IA32_APIC_BASE MSR (0x1B)
    __asm__ volatile("rdmsr" : "=a"(lo), "=d"(hi) : "c"(0x1B));
    
    // 設置APIC使能位 (bit 11)
    lo |= (1 << 11);
    
    // 寫回MSR
    __asm__ volatile("wrmsr" : : "a"(lo), "d"(hi), "c"(0x1B));
}

// 禁用傳統的PIC (8259A)
void pic_disable() {
    // PIC (8259A) 端口
    #define PIC1_COMMAND 0x20
    #define PIC1_DATA    0x21
    #define PIC2_COMMAND 0xA0
    #define PIC2_DATA    0xA1

    // PIC命令
    #define PIC_EOI      0x20
    #define ICW1_ICW4    0x01
    #define ICW1_INIT    0x10
    #define ICW4_8086    0x01
    
    // 初始化PIC
    io_port_wb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_port_wb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    
    // 設置中斷向量偏移
    io_port_wb(PIC1_DATA, 0x20); // 設置主PIC的向量偏移為32
    io_port_wb(PIC2_DATA, 0x28); // 設置從PIC的向量偏移為40
    
    // 設置級聯
    io_port_wb(PIC1_DATA, 4);    // 告訴主PIC從PIC連接到IRQ2
    io_port_wb(PIC2_DATA, 2);    // 告訴從PIC它的級聯標識
    
    // 設置8086模式
    io_port_wb(PIC1_DATA, ICW4_8086);
    io_port_wb(PIC2_DATA, ICW4_8086);
    
    // 掩蔽所有中斷
    io_port_wb(PIC1_DATA, 0xFF);
    io_port_wb(PIC2_DATA, 0xFF);
    
    printf("[LAPIC] Legacy PIC disabled\n");
}

int lapic_init(uintptr_t lapic_addr) {
    printf("[LAPIC] Initializing Local APIC\n");
    
    // 檢查CPU是否支持APIC
    if (!cpu_has_apic()) {
        printf("[LAPIC] CPU does not support APIC\n");
        return 0;
    }
    
    // 映射本地APIC到虛擬地址空間
    lapic_base_vaddr = 0xFFFE0000; // 通常使用一個固定的虛擬地址
    void* mapped_addr = vmm_map_page((void*)lapic_base_vaddr, (void*)lapic_addr, PG_PREM_RW, PG_PREM_RW);
    if (mapped_addr != (void*)lapic_base_vaddr) {
        printf("[LAPIC] Failed to map Local APIC to virtual address\n");
        return 0;
    }
    
    printf("[LAPIC] Local APIC mapped to virtual address 0x%x\n", lapic_base_vaddr);
    
    // 禁用舊的PIC
    pic_disable();
    
    // 啟用MSR中的APIC
    enable_apic_in_msr();
    
    // 啟用本地APIC
    lapic_write(LAPIC_SVR, lapic_read(LAPIC_SVR) | LAPIC_SVR_ENABLE | 0xFF);
    
    // 顯示APIC版本信息
    uint32_t version = lapic_read(LAPIC_VERSION);
    printf("[LAPIC] Local APIC Version: %d, Max LVT entries: %d\n", 
           version & 0xFF, ((version >> 16) & 0xFF) + 1);
    
    // 配置LINT0和LINT1
    lapic_configure_lint(0, 0, LAPIC_LVT_DELIVERY_EXTINT, 0); // LINT0作為ExtINT
    lapic_configure_lint(1, 0, LAPIC_LVT_DELIVERY_NMI, 0);    // LINT1作為NMI
    
    // 配置APIC錯誤處理
    lapic_write(LAPIC_ESR, 0); // 清除錯誤狀態
    lapic_write(LAPIC_LVT_ERROR, 0xFE); // 錯誤中斷向量0xFE
    
    // 配置任務優先級
    lapic_write(LAPIC_TPR, 0); // 允許所有外部中斷
    
    // 配置邏輯目標寄存器
    lapic_write(LAPIC_LDR, (lapic_read(LAPIC_LDR) & 0x00FFFFFF) | (1 << 24)); // CPU 0
    
    // 配置目標格式寄存器
    lapic_write(LAPIC_DFR, 0xFFFFFFFF); // 平面模式
    
    printf("[LAPIC] Local APIC initialized successfully\n");
    return 1;
}

uint32_t lapic_read(uint32_t reg) {
    if (!lapic_base_vaddr) {
        printf("[LAPIC] Error: Local APIC not initialized\n");
        return 0;
    }
    return *((volatile uint32_t*)(lapic_base_vaddr + reg));
}

void lapic_write(uint32_t reg, uint32_t value) {
    if (!lapic_base_vaddr) {
        printf("[LAPIC] Error: Local APIC not initialized\n");
        return;
    }
    *((volatile uint32_t*)(lapic_base_vaddr + reg)) = value;
}

uint32_t lapic_get_id() {
    return lapic_read(LAPIC_ID);
}

void lapic_send_eoi() {
    lapic_write(LAPIC_EOI, 0);
}

void lapic_configure_timer(uint8_t vector, uint32_t mode, uint32_t initial_count) {
    // 通常分頻為1 (值為0xB)，這裡使用值為0x3（除數為16）提高穩定性
    lapic_write(LAPIC_TIMER_DIVIDE_CONFIG, 0x3);
    
    // 設置LVT Timer項
    lapic_write(LAPIC_LVT_TIMER, vector | mode);
    
    // 記錄設置的初始計數值
    printf("[LAPIC Timer] Configure with initial count: %u\n", initial_count);
    
    // 設置初始計數值
    lapic_write(LAPIC_TIMER_INITIAL_COUNT, initial_count);
}

void lapic_stop_timer() {
    // 停止定時器 (設置掩碼位)
    lapic_write(LAPIC_LVT_TIMER, lapic_read(LAPIC_LVT_TIMER) | LAPIC_LVT_MASKED);
    
    // 清零計數值
    lapic_write(LAPIC_TIMER_INITIAL_COUNT, 0);
}

void lapic_configure_lint(int lint, uint8_t vector, uint32_t delivery_mode, int masked) {
    uint32_t value = vector | delivery_mode;
    if (masked) {
        value |= LAPIC_LVT_MASKED;
    }
    
    if (lint == 0) {
        lapic_write(LAPIC_LVT_LINT0, value);
    } else if (lint == 1) {
        lapic_write(LAPIC_LVT_LINT1, value);
    }
}

void lapic_send_ipi(uint8_t dest_apic_id, uint32_t delivery_mode, uint8_t vector) {
    printf("[LAPIC] Sending IPI: dest=%d, mode=%x, vector=%d\n", 
           dest_apic_id, delivery_mode, vector);
           
    // 設置ICR高32位(目標APIC ID)
    lapic_write(LAPIC_ICR_HIGH, ((uint32_t)dest_apic_id) << 24);
    
    // 清除ICR的Delivery Status位(確保之前的IPI已完成)
    while (lapic_read(LAPIC_ICR_LOW) & LAPIC_ICR_STATUS_PENDING) {
        __asm__ volatile("pause");
    }
    
    // 設置ICR低32位(傳遞模式、向量等)
    uint32_t icr_low = vector | delivery_mode | LAPIC_ICR_LEVEL_ASSERT;
    if (delivery_mode != LAPIC_ICR_DELIVERY_INIT && 
        delivery_mode != LAPIC_ICR_DELIVERY_STARTUP) {
        icr_low |= LAPIC_ICR_TRIGGER_EDGE;
    }
    
    lapic_write(LAPIC_ICR_LOW, icr_low);
    
    // 等待IPI發送完成
    while (lapic_read(LAPIC_ICR_LOW) & LAPIC_ICR_STATUS_PENDING) {
        __asm__ volatile("pause");
    }
}

void lapic_broadcast_ipi(uint32_t delivery_mode, uint8_t vector, int exclude_self) {
    // 設置ICR低32位(傳遞模式、向量等)
    uint32_t icr_low = vector | delivery_mode | LAPIC_ICR_LEVEL_ASSERT;
    
    // 設置目標
    if (exclude_self) {
        icr_low |= LAPIC_ICR_DESTINATION_ALL_BUT_SELF;
    } else {
        icr_low |= LAPIC_ICR_DESTINATION_ALL;
    }
    
    // 發送IPI
    lapic_write(LAPIC_ICR_LOW, icr_low);
    
    // 等待IPI發送完成
    while (lapic_read(LAPIC_ICR_LOW) & LAPIC_ICR_STATUS_PENDING) {
        __asm__ volatile("pause");
    }
}
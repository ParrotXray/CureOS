#include <hal/apic/apic.h>
#include <hal/acpi/acpi.h>
#include <system/mm/vmm.h>
#include <system/mm/page.h>
#include <hal/io.h>
#include <hal/apic/irq_setup.h>
#include <hal/apic/apic_timer.h>
#include <libc/stdio.h>

// 本地APIC映射的虛擬地址
static uintptr_t lapic_base_vaddr = 0;

// IO APIC資訊
static uint8_t io_apic_id = 0;
static uintptr_t io_apic_phys_addr = 0;
static uintptr_t io_apic_virt_addr = 0;

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
    
    printf("[APIC] Legacy PIC disabled\n");
}

int apic_init() {
    printf("[APIC] Initializing APIC\n");
    
    // 檢查CPU是否支持APIC
    if (!cpu_has_apic()) {
        printf("[APIC] CPU does not support APIC\n");
        return 0;
    }
    
    // 檢查ACPI是否提供APIC信息
    if (!acpi_is_supported()) {
        printf("[APIC] ACPI not supported, cannot get APIC information\n");
        return 0;
    }
    
    // 獲取本地APIC地址
    uintptr_t lapic_phys_addr = acpi_get_local_apic_addr();
    if (lapic_phys_addr == 0) {
        printf("[APIC] Failed to get Local APIC address\n");
        return 0;
    }
    printf("[APIC] Local APIC physical address: 0x%x\n", lapic_phys_addr);
    
    // 獲取IO APIC信息
    if (!acpi_get_io_apic_info(&io_apic_id, &io_apic_phys_addr)) {
        printf("[APIC] Failed to get IO APIC information\n");
        return 0;
    }
    printf("[APIC] IO APIC ID: %d, physical address: 0x%x\n", io_apic_id, io_apic_phys_addr);
    
    // 映射本地APIC到虛擬地址空間
    lapic_base_vaddr = 0xFFFE0000; // 通常使用一個固定的虛擬地址
    void* mapped_addr = vmm_map_page((void*)lapic_base_vaddr, (void*)lapic_phys_addr, PG_PREM_RW, PG_PREM_RW);
    if (mapped_addr != (void*)lapic_base_vaddr) {
        printf("[APIC] Failed to map Local APIC to virtual address\n");
        return 0;
    }
    
    // 映射IO APIC到虛擬地址空間
    io_apic_virt_addr = 0xFFFD0000; // 另一個固定的虛擬地址
    mapped_addr = vmm_map_page((void*)io_apic_virt_addr, (void*)io_apic_phys_addr, PG_PREM_RW, PG_PREM_RW);
    if (mapped_addr != (void*)io_apic_virt_addr) {
        printf("[APIC] Failed to map IO APIC to virtual address\n");
        return 0;
    }
    
    // 禁用舊的PIC
    pic_disable();
    
    // 啟用MSR中的APIC
    enable_apic_in_msr();
    
    // 啟用本地APIC
    apic_write(APIC_SVR, apic_read(APIC_SVR) | APIC_SVR_ENABLE | 0xFF);
    
    // 顯示APIC版本信息
    uint32_t version = apic_read(APIC_VERSION);
    printf("[APIC] Local APIC Version: %d, Max LVT entries: %d\n", 
           version & 0xFF, ((version >> 16) & 0xFF) + 1);
    
    // 配置LINT0和LINT1
    apic_configure_lint(0, 0, APIC_LVT_DELIVERY_EXTINT, 0); // LINT0作為ExtINT
    apic_configure_lint(1, 0, APIC_LVT_DELIVERY_NMI, 0);    // LINT1作為NMI
    
    // 配置APIC錯誤處理
    apic_write(APIC_ESR, 0); // 清除錯誤狀態
    apic_write(APIC_LVT_ERROR, 0xFE); // 錯誤中斷向量0xFE
    
    // 配置APIC虛假中斷處理
    apic_write(APIC_SVR, apic_read(APIC_SVR) | 0xFF); // 虛假中斷向量0xFF
    
    // 配置任務優先級
    apic_write(APIC_TPR, 0); // 允許所有外部中斷
    
    // 配置邏輯目標寄存器
    apic_write(APIC_LDR, (apic_read(APIC_LDR) & 0x00FFFFFF) | (1 << 24)); // CPU 0
    
    // 配置目標格式寄存器
    apic_write(APIC_DFR, 0xFFFFFFFF); // 平面模式
    
    // 初始化IO APIC
    int max_redirs = io_apic_max_redirection_entry(io_apic_virt_addr);
    printf("[APIC] IO APIC supports %d redirection entries\n", max_redirs + 1);
    
    // 配置IO APIC中斷重定向
    configure_io_apic_irqs();
    
    // 初始化APIC計時器
    apic_timer_init();
    
    printf("[APIC] APIC initialized successfully\n");
    return 1;
}

uint32_t apic_read(uint32_t reg) {
    return *((volatile uint32_t*)(lapic_base_vaddr + reg));
}

void apic_write(uint32_t reg, uint32_t value) {
    *((volatile uint32_t*)(lapic_base_vaddr + reg)) = value;
}

uint32_t apic_get_id() {
    return apic_read(APIC_ID);
}

void apic_send_eoi() {
    apic_write(APIC_EOI, 0);
}

// 修改計時器設置函數 - 確保它使用正確的頻率
void apic_configure_timer(uint8_t vector, uint32_t mode, uint32_t initial_count) {
    // 通常分頻為1 (值為0xB)
    apic_write(APIC_TIMER_DIVIDE_CONFIG, 0x3);  // 除數為16，提高穩定性
    
    // 設置LVT Timer項
    apic_write(APIC_LVT_TIMER, vector | mode);
    
    // 記錄設置的初始計數值
    printf("[APIC Timer] Configure with initial count: %u\n", initial_count);
    
    // 設置初始計數值
    apic_write(APIC_TIMER_INITIAL_COUNT, initial_count);
}


void apic_stop_timer() {
    // 停止定時器 (設置掩碼位)
    apic_write(APIC_LVT_TIMER, apic_read(APIC_LVT_TIMER) | APIC_LVT_MASKED);
    
    // 清零計數值
    apic_write(APIC_TIMER_INITIAL_COUNT, 0);
}

void apic_configure_lint(int lint, uint8_t vector, uint32_t delivery_mode, int masked) {
    uint32_t value = vector | delivery_mode;
    if (masked) {
        value |= APIC_LVT_MASKED;
    }
    
    if (lint == 0) {
        apic_write(APIC_LVT_LINT0, value);
    } else if (lint == 1) {
        apic_write(APIC_LVT_LINT1, value);
    }
}

uint32_t io_apic_read(uintptr_t io_apic_base, uint8_t reg) {
    // 選擇要讀取的寄存器
    *((volatile uint32_t*)(io_apic_base + IOAPIC_IOREGSEL)) = reg;
    
    // 從窗口讀取值
    return *((volatile uint32_t*)(io_apic_base + IOAPIC_IOWIN));
}

void io_apic_write(uintptr_t io_apic_base, uint8_t reg, uint32_t value) {
    // 選擇要寫入的寄存器
    *((volatile uint32_t*)(io_apic_base + IOAPIC_IOREGSEL)) = reg;
    
    // 通過窗口寫入值
    *((volatile uint32_t*)(io_apic_base + IOAPIC_IOWIN)) = value;
}

int io_apic_max_redirection_entry(uintptr_t io_apic_base) {
    uint32_t ver = io_apic_read(io_apic_base, IOAPIC_REG_VER);
    return (ver >> 16) & 0xFF;
}

void io_apic_setup_redirection(uintptr_t io_apic_base, uint8_t irq, uint8_t vector, 
                              uint8_t delivery_mode, uint8_t dest_mode, uint8_t dest, 
                              uint32_t flags) {
    uint32_t low = vector | (delivery_mode << 8) | (dest_mode << 11) | flags;
    uint32_t high = ((uint32_t)dest) << 24;
    
    uint32_t ioredtbl = IOAPIC_REG_REDTBL_BASE + 2 * irq;
    
    // 寫入高32位
    io_apic_write(io_apic_base, ioredtbl + 1, high);
    
    // 寫入低32位
    io_apic_write(io_apic_base, ioredtbl, low);
}

void io_apic_mask_irq(uintptr_t io_apic_base, uint8_t irq) {
    uint32_t ioredtbl = IOAPIC_REG_REDTBL_BASE + 2 * irq;
    uint32_t value = io_apic_read(io_apic_base, ioredtbl);
    
    // 設置掩碼位
    io_apic_write(io_apic_base, ioredtbl, value | IOAPIC_REDTBL_MASKED);
}

void io_apic_unmask_irq(uintptr_t io_apic_base, uint8_t irq) {
    uint32_t ioredtbl = IOAPIC_REG_REDTBL_BASE + 2 * irq;
    uint32_t value = io_apic_read(io_apic_base, ioredtbl);
    
    // 清除掩碼位
    io_apic_write(io_apic_base, ioredtbl, value & ~IOAPIC_REDTBL_MASKED);
}

void apic_send_ipi(uint8_t dest_apic_id, uint32_t delivery_mode, uint8_t vector) {
    printf("[APIC] Sending IPI: dest=%d, mode=%x, vector=%d\n", 
           dest_apic_id, delivery_mode, vector);
           
    // 設置ICR高32位(目標APIC ID)
    apic_write(APIC_ICR_HIGH, ((uint32_t)dest_apic_id) << 24);
    
    // 清除ICR的Delivery Status位(確保之前的IPI已完成)
    while (apic_read(APIC_ICR_LOW) & APIC_ICR_STATUS_PENDING) {
        __asm__ volatile("pause");
    }
    
    // 設置ICR低32位(傳遞模式、向量等)
    uint32_t icr_low = vector | delivery_mode | APIC_ICR_LEVEL_ASSERT;
    if (delivery_mode != APIC_ICR_DELIVERY_INIT && 
        delivery_mode != APIC_ICR_DELIVERY_STARTUP) {
        icr_low |= APIC_ICR_TRIGGER_EDGE;
    }
    
    apic_write(APIC_ICR_LOW, icr_low);
    
    // 等待IPI發送完成
    while (apic_read(APIC_ICR_LOW) & APIC_ICR_STATUS_PENDING) {
        __asm__ volatile("pause");
    }
}

void apic_broadcast_ipi(uint32_t delivery_mode, uint8_t vector, int exclude_self) {
    // 設置ICR低32位(傳遞模式、向量等)
    uint32_t icr_low = vector | delivery_mode | APIC_ICR_LEVEL_ASSERT;
    
    // 設置目標
    if (exclude_self) {
        icr_low |= APIC_ICR_DESTINATION_ALL_BUT_SELF;
    } else {
        icr_low |= APIC_ICR_DESTINATION_ALL;
    }
    
    // 發送IPI
    apic_write(APIC_ICR_LOW, icr_low);
    
    // 等待IPI發送完成
    while (apic_read(APIC_ICR_LOW) & APIC_ICR_STATUS_PENDING) {
        __asm__ volatile("pause");
    }
}

// 取得已映射的 IO APIC 基址
uintptr_t apic_get_io_apic_base(void) { return io_apic_virt_addr; }
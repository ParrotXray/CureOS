#include <system/apic/apic.h>
#include <system/acpi/acpi.h>
#include <system/apic/irq_setup.h>
#include <arch/x86/interrupts.h>
#include <libc/stdio.h>

// 標準IRQ向量基址 (IRQ 0-15 映射到中斷向量 32-47)
#define IRQ_BASE_VECTOR 32

// IRQ描述結構
typedef struct {
    const char* name;       // IRQ名稱
    uint16_t vector;         // 映射的中斷向量
    uint16_t delivery_mode;  // 傳遞模式
    uint16_t trigger_mode;   // 觸發模式 (邊緣或電平)
    uint16_t polarity;       // 極性 (高電平或低電平)
    uint16_t masked;         // 是否被掩蔽
} irq_config_t;

// 標準ISA IRQ配置
static irq_config_t isa_irq_config[16] = {
    { "PIT Timer",      IRQ_BASE_VECTOR + 0,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 0: PIT
    { "Keyboard",       IRQ_BASE_VECTOR + 1,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 0 }, // IRQ 1: 鍵盤
    { "Cascade",        IRQ_BASE_VECTOR + 2,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 2: 從PIC級聯
    { "COM2",           IRQ_BASE_VECTOR + 3,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 3: 串口2
    { "COM1",           IRQ_BASE_VECTOR + 4,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 4: 串口1
    { "LPT2",           IRQ_BASE_VECTOR + 5,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 5: 並口2
    { "Floppy",         IRQ_BASE_VECTOR + 6,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 6: 軟盤
    { "LPT1",           IRQ_BASE_VECTOR + 7,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 7: 並口1
    { "RTC",            IRQ_BASE_VECTOR + 8,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 8: 實時時鐘
    { "Free",           IRQ_BASE_VECTOR + 9,  0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 9: 可用
    { "Free",           IRQ_BASE_VECTOR + 10, 0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 10: 可用
    { "Free",           IRQ_BASE_VECTOR + 11, 0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 11: 可用
    { "PS/2 Mouse",     IRQ_BASE_VECTOR + 12, 0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 12: PS/2鼠標
    { "FPU",            IRQ_BASE_VECTOR + 13, 0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 13: 浮點運算單元
    { "Primary ATA",    IRQ_BASE_VECTOR + 14, 0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }, // IRQ 14: 主ATA
    { "Secondary ATA",  IRQ_BASE_VECTOR + 15, 0, IOAPIC_REDTBL_TRIGGER_EDGE,  0, 1 }  // IRQ 15: 從ATA
};

// 中斷重定向覆蓋表 (從ACPI MADT獲取)
typedef struct {
    uint8_t irq_source;
    uint32_t gsi;
    uint16_t flags;
} irq_override_t;

// 最多支持16個IRQ重定向覆蓋
static irq_override_t irq_overrides[16];
static int override_count = 0;

// 根據覆蓋表更新IRQ配置
static void update_irq_config_from_overrides() {
    for (int i = 0; i < override_count; i++) {
        uint8_t irq = irq_overrides[i].irq_source;
        uint16_t flags = irq_overrides[i].flags;
        
        if (irq < 16) {
            // 更新觸發模式
            if (flags & 0x2) { // 電平觸發
                isa_irq_config[irq].trigger_mode = IOAPIC_REDTBL_TRIGGER_LEVEL;
            } else { // 邊緣觸發
                isa_irq_config[irq].trigger_mode = IOAPIC_REDTBL_TRIGGER_EDGE;
            }
            
            // 更新極性
            if (flags & 0x4) { // 低電平有效
                isa_irq_config[irq].polarity = IOAPIC_REDTBL_INTPOL_LOW;
            } else { // 高電平有效
                isa_irq_config[irq].polarity = IOAPIC_REDTBL_INTPOL_HIGH;
            }
            
            printf("[IRQ] Override for IRQ %d: GSI=%d, TriggerMode=%s, Polarity=%s\n",
                   irq, irq_overrides[i].gsi,
                   (isa_irq_config[irq].trigger_mode == IOAPIC_REDTBL_TRIGGER_LEVEL) ? "Level" : "Edge",
                   (isa_irq_config[irq].polarity == IOAPIC_REDTBL_INTPOL_LOW) ? "Low" : "High");
        }
    }
}

// 從ACPI獲取中斷重定向信息
static void parse_interrupt_overrides() {
    // 獲取MADT表
    acpi_madt_t* madt = (acpi_madt_t*)acpi_find_table(ACPI_MADT_SIGNATURE);
    if (!madt) {
        printf("[IRQ] MADT not found, using default IRQ configuration\n");
        return;
    }
    
    // 解析MADT中的重定向項
    override_count = 0;
    
    uintptr_t current = (uintptr_t)madt + sizeof(acpi_madt_t);
    uintptr_t end = (uintptr_t)madt + madt->header.length;
    
    while (current < end && override_count < 16) {
        acpi_madt_record_header_t* record = (acpi_madt_record_header_t*)current;
        
        if (record->type == ACPI_MADT_TYPE_INT_SRC_OVERRIDE) {
            acpi_madt_int_src_override_t* override = (acpi_madt_int_src_override_t*)record;
            
            // 保存覆蓋信息
            irq_overrides[override_count].irq_source = override->source;
            irq_overrides[override_count].gsi = override->global_system_interrupt;
            irq_overrides[override_count].flags = override->flags;
            
            override_count++;
        }
        
        current += record->length;
    }
    
    printf("[IRQ] Found %d interrupt source overrides\n", override_count);
}

// 配置IO APIC中斷重定向
void configure_io_apic_irqs() {
    printf("[IRQ] Configuring IO APIC IRQs\n");
    
    // 獲取IO APIC地址
    uint8_t io_apic_id;
    uintptr_t io_apic_addr;
    
    if (!acpi_get_io_apic_info(&io_apic_id, &io_apic_addr)) {
        printf("[IRQ] Failed to get IO APIC information\n");
        return;
    }
    
    // 從ACPI獲取中斷重定向信息
    parse_interrupt_overrides();
    
    // 更新IRQ配置
    update_irq_config_from_overrides();
    
    // 獲取當前CPU的APIC ID
    uint32_t apic_id = apic_get_id() >> 24;
    
    // 配置每個IRQ
    for (int i = 0; i < 16; i++) {
        irq_config_t* config = &isa_irq_config[i];
        
        // 查找此IRQ是否有覆蓋
        uint32_t gsi = i;  // 默認為IRQ號
        
        for (int j = 0; j < override_count; j++) {
            if (irq_overrides[j].irq_source == i) {
                gsi = irq_overrides[j].gsi;
                break;
            }
        }
        
        // 設置中斷重定向表中的標誌
        uint32_t flags = config->trigger_mode | config->polarity;
        if (config->masked) {
            flags |= IOAPIC_REDTBL_MASKED;
        }
        
        // 配置IO APIC
        io_apic_setup_redirection(
            io_apic_addr,
            gsi,                 // GSI號 (可能被覆蓋)
            config->vector,      // 向量號
            config->delivery_mode,
            IOAPIC_REDTBL_DESTMOD_PHYSICAL,
            apic_id,
            flags
        );
        
        printf("[IRQ] Configured IRQ %d (%s): Vector=%d, GSI=%d, %s\n",
               i, config->name, config->vector, gsi,
               config->masked ? "Masked" : "Unmasked");
    }
    
    printf("[IRQ] IO APIC configuration complete\n");
}

// 啟用特定IRQ
void enable_irq(uint8_t irq) {
    if (irq >= 16) return;
    
    // 獲取IO APIC地址
    uint8_t io_apic_id;
    uintptr_t io_apic_addr;
    
    if (!acpi_get_io_apic_info(&io_apic_id, &io_apic_addr)) {
        return;
    }
    
    // 查找此IRQ是否有覆蓋
    uint32_t gsi = irq;
    
    for (int j = 0; j < override_count; j++) {
        if (irq_overrides[j].irq_source == irq) {
            gsi = irq_overrides[j].gsi;
            break;
        }
    }
    
    // 取消掩蔽
    io_apic_unmask_irq(io_apic_addr, gsi);
    isa_irq_config[irq].masked = 0;
    
    printf("[IRQ] Enabled IRQ %d (%s)\n", irq, isa_irq_config[irq].name);
}

// 禁用特定IRQ
void disable_irq(uint8_t irq) {
    if (irq >= 16) return;
    
    // 獲取IO APIC地址
    uint8_t io_apic_id;
    uintptr_t io_apic_addr;
    
    if (!acpi_get_io_apic_info(&io_apic_id, &io_apic_addr)) {
        return;
    }
    
    // 查找此IRQ是否有覆蓋
    uint32_t gsi = irq;
    
    for (int j = 0; j < override_count; j++) {
        if (irq_overrides[j].irq_source == irq) {
            gsi = irq_overrides[j].gsi;
            break;
        }
    }
    
    // 掩蔽
    io_apic_mask_irq(io_apic_addr, gsi);
    isa_irq_config[irq].masked = 1;
    
    printf("[IRQ] Disabled IRQ %d (%s)\n", irq, isa_irq_config[irq].name);
}
#include <hal/apic/apic.h>
#include <hal/acpi/acpi.h>
#include <hal/apic/irq_setup.h>
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

    // 取得 IO APIC 已映射的虛擬基址
    uintptr_t io_apic_base = apic_get_io_apic_base();
    if (!io_apic_base) {
        printf("[IRQ] IO APIC base not mapped\n");
        return;
    }

    // 從 ACPI 解析 Interrupt Source Override
    parse_interrupt_overrides();
    update_irq_config_from_overrides();

    // 取得 IO APIC 能處理的最大 redirection entry
    int max_entry = io_apic_max_redirection_entry(io_apic_base);

    // 取得本 CPU 的 APIC ID (高 8 位)
    uint8_t dest_apic = (apic_get_id() >> 24) & 0xFF;

    // 依序配置 ISA IRQ 0-15
    for (uint8_t irq = 0; irq < 16; irq++) {

        // GSI (可能被 Override)
        uint32_t gsi = irq;
        for (int j = 0; j < override_count; j++) {
            if (irq_overrides[j].irq_source == irq) {
                gsi = irq_overrides[j].gsi;
                break;
            }
        }
        if (gsi > max_entry) continue;          // 超出 IO APIC 範圍，跳過

        irq_config_t* cfg = &isa_irq_config[irq];

        uint32_t flags = cfg->trigger_mode | cfg->polarity;
        if (cfg->masked) flags |= IOAPIC_REDTBL_MASKED;

        io_apic_setup_redirection(
            io_apic_base,
            gsi,
            cfg->vector,
            cfg->delivery_mode,
            IOAPIC_REDTBL_DESTMOD_PHYSICAL,
            dest_apic,
            flags
        );

        printf("[IRQ] Configured IRQ %d (%s): Vector=%u, GSI=%u, %s\n",
               irq, cfg->name, cfg->vector, gsi,
               cfg->masked ? "Masked" : "Unmasked");
    }

    printf("[IRQ] IO APIC configuration complete\n");
}

// 啟用特定 IRQ
void enable_irq(uint8_t irq)
{
    if (irq >= 16) return;

    uintptr_t io_apic_addr = apic_get_io_apic_base();
    if (!io_apic_addr) return;               // 尚未映射

    // 查找是否有 ACPI override
    uint32_t gsi = irq;
    for (int j = 0; j < override_count; j++) {
        if (irq_overrides[j].irq_source == irq) {
            gsi = irq_overrides[j].gsi;
            break;
        }
    }

    io_apic_unmask_irq(io_apic_addr, gsi);
    isa_irq_config[irq].masked = 0;

    printf("[IRQ] Enabled IRQ %d (%s)\n", irq, isa_irq_config[irq].name);
}

// 禁用特定 IRQ
void disable_irq(uint8_t irq)
{
    if (irq >= 16) return;

    uintptr_t io_apic_addr = apic_get_io_apic_base();
    if (!io_apic_addr) return;

    uint32_t gsi = irq;
    for (int j = 0; j < override_count; j++) {
        if (irq_overrides[j].irq_source == irq) {
            gsi = irq_overrides[j].gsi;
            break;
        }
    }

    io_apic_mask_irq(io_apic_addr, gsi);
    isa_irq_config[irq].masked = 1;

    printf("[IRQ] Disabled IRQ %d (%s)\n", irq, isa_irq_config[irq].name);
}

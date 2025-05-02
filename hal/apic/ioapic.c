#include <hal/apic/ioapic.h>
#include <system/mm/vmm.h>
#include <system/mm/page.h>
#include <libc/stdio.h>

// IO APIC資訊
static uint8_t io_apic_id = 0;
static uintptr_t io_apic_phys_addr = 0;
static uintptr_t io_apic_virt_addr = 0;

int ioapic_init(uint8_t apic_id, uintptr_t apic_addr) {
    printf("[IOAPIC] Initializing IO APIC\n");
    
    // 保存IO APIC信息
    io_apic_id = apic_id;
    io_apic_phys_addr = apic_addr;
    
    // 映射IO APIC到虛擬地址空間
    io_apic_virt_addr = 0xFFFD0000; // 固定的虛擬地址
    void* mapped_addr = vmm_map_page((void*)io_apic_virt_addr, (void*)io_apic_phys_addr, PG_PREM_RW, PG_PREM_RW);
    if (mapped_addr != (void*)io_apic_virt_addr) {
        printf("[IOAPIC] Failed to map IO APIC to virtual address\n");
        return 0;
    }
    
    // 顯示IO APIC版本信息
    uint32_t ver = ioapic_read(IOAPIC_REG_VER);
    int max_entries = (ver >> 16) & 0xFF;
    printf("[IOAPIC] ID: %d, Version: %d, Max Redirection Entries: %d\n", 
           io_apic_id, ver & 0xFF, max_entries + 1);
    
    printf("[IOAPIC] IO APIC initialized successfully\n");
    return 1;
}

uintptr_t ioapic_get_base(void) {
    return io_apic_virt_addr;
}

uint8_t ioapic_get_id(void) {
    return io_apic_id;
}

uint32_t ioapic_read(uint8_t reg) {
    if (!io_apic_virt_addr) {
        printf("[IOAPIC] Error: IO APIC not initialized\n");
        return 0;
    }
    
    // 選擇要讀取的寄存器
    *((volatile uint32_t*)(io_apic_virt_addr + IOAPIC_IOREGSEL)) = reg;
    
    // 從窗口讀取值
    return *((volatile uint32_t*)(io_apic_virt_addr + IOAPIC_IOWIN));
}

void ioapic_write(uint8_t reg, uint32_t value) {
    if (!io_apic_virt_addr) {
        printf("[IOAPIC] Error: IO APIC not initialized\n");
        return;
    }
    
    // 選擇要寫入的寄存器
    *((volatile uint32_t*)(io_apic_virt_addr + IOAPIC_IOREGSEL)) = reg;
    
    // 通過窗口寫入值
    *((volatile uint32_t*)(io_apic_virt_addr + IOAPIC_IOWIN)) = value;
}

int ioapic_max_redirection_entry(void) {
    uint32_t ver = ioapic_read(IOAPIC_REG_VER);
    return (ver >> 16) & 0xFF;
}

void ioapic_setup_redirection(uint8_t irq, uint8_t vector, 
                             uint8_t delivery_mode, uint8_t dest_mode, 
                             uint8_t dest, uint32_t flags) {
    uint32_t low = vector | (delivery_mode << 8) | (dest_mode << 11) | flags;
    uint32_t high = ((uint32_t)dest) << 24;
    
    uint32_t ioredtbl = IOAPIC_REG_REDTBL_BASE + 2 * irq;
    
    // 寫入高32位
    ioapic_write(ioredtbl + 1, high);
    
    // 寫入低32位
    ioapic_write(ioredtbl, low);
    
    printf("[IOAPIC] Setup IRQ %d: vector=%d, delivery_mode=%d, dest=%d, flags=0x%x\n",
           irq, vector, delivery_mode, dest, flags);
}

void ioapic_mask_irq(uint8_t irq) {
    uint32_t ioredtbl = IOAPIC_REG_REDTBL_BASE + 2 * irq;
    uint32_t value = ioapic_read(ioredtbl);
    
    // 設置掩碼位
    ioapic_write(ioredtbl, value | IOAPIC_REDTBL_MASKED);
    
    printf("[IOAPIC] Masked IRQ %d\n", irq);
}

void ioapic_unmask_irq(uint8_t irq) {
    uint32_t ioredtbl = IOAPIC_REG_REDTBL_BASE + 2 * irq;
    uint32_t value = ioapic_read(ioredtbl);
    
    // 清除掩碼位
    ioapic_write(ioredtbl, value & ~IOAPIC_REDTBL_MASKED);
    
    printf("[IOAPIC] Unmasked IRQ %d\n", irq);
}
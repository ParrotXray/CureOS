#include <hal/apic/apic.h>
#include <hal/apic/lapic.h>
#include <hal/apic/ioapic.h>
#include <hal/acpi/acpi.h>
#include <hal/apic/irq_setup.h>
#include <hal/apic/apic_timer.h>
#include <libc/stdio.h>

int apic_init() {
    printf("[APIC] Initializing APIC\n");
    
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
    uint8_t io_apic_id = 0;
    uintptr_t io_apic_phys_addr = 0;
    if (!acpi_get_io_apic_info(&io_apic_id, &io_apic_phys_addr)) {
        printf("[APIC] Failed to get IO APIC information\n");
        return 0;
    }
    printf("[APIC] IO APIC ID: %d, physical address: 0x%x\n", io_apic_id, io_apic_phys_addr);
    
    // 初始化Local APIC
    if (!lapic_init(lapic_phys_addr)) {
        printf("[APIC] Failed to initialize Local APIC\n");
        return 0;
    }
    
    // 初始化IO APIC
    if (!ioapic_init(io_apic_id, io_apic_phys_addr)) {
        printf("[APIC] Failed to initialize IO APIC\n");
        return 0;
    }
    
    // 配置IO APIC中斷重定向
    configure_io_apic_irqs();
    
    // 初始化APIC計時器
    apic_timer_init();
    
    printf("[APIC] APIC initialized successfully\n");
    return 1;
}

// 提供兼容性函數
uint32_t apic_read(uint32_t reg) {
    return lapic_read(reg);
}

void apic_write(uint32_t reg, uint32_t value) {
    lapic_write(reg, value);
}

uint32_t apic_get_id() {
    return lapic_get_id();
}

void apic_send_eoi() {
    lapic_send_eoi();
}

void apic_configure_timer(uint8_t vector, uint32_t mode, uint32_t initial_count) {
    lapic_configure_timer(vector, mode, initial_count);
}

void apic_stop_timer() {
    lapic_stop_timer();
}

void apic_configure_lint(int lint, uint8_t vector, uint32_t delivery_mode, int masked) {
    lapic_configure_lint(lint, vector, delivery_mode, masked);
}

void apic_send_ipi(uint8_t dest_apic_id, uint32_t delivery_mode, uint8_t vector) {
    lapic_send_ipi(dest_apic_id, delivery_mode, vector);
}

void apic_broadcast_ipi(uint32_t delivery_mode, uint8_t vector, int exclude_self) {
    lapic_broadcast_ipi(delivery_mode, vector, exclude_self);
}

uintptr_t apic_get_io_apic_base(void) {
    return ioapic_get_base();
}
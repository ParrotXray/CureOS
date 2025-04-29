#ifndef _SYSTEM_IRQ_SETUP_H
#define _SYSTEM_IRQ_SETUP_H

#include <stdint.h>

/**
 * @brief 配置IO APIC中斷重定向
 * 
 * 根據ACPI信息設置IRQ路由
 */
void configure_io_apic_irqs();

/**
 * @brief 啟用特定IRQ
 * 
 * @param irq IRQ號 (0-15)
 */
void enable_irq(uint8_t irq);

/**
 * @brief 禁用特定IRQ
 * 
 * @param irq IRQ號 (0-15)
 */
void disable_irq(uint8_t irq);

#endif /* _SYSTEM_IRQ_SETUP_H */
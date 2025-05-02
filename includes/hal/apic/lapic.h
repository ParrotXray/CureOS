#ifndef _SYSTEM_LAPIC_H
#define _SYSTEM_LAPIC_H

#include <stdint.h>

// Local APIC寄存器偏移量 (基於APIC基地址)
#define LAPIC_ID                  0x20  // Local APIC ID寄存器
#define LAPIC_VERSION             0x30  // Local APIC版本寄存器
#define LAPIC_TPR                 0x80  // Task Priority寄存器
#define LAPIC_APR                 0x90  // Arbitration Priority寄存器
#define LAPIC_PPR                 0xA0  // Processor Priority寄存器
#define LAPIC_EOI                 0xB0  // EOI寄存器
#define LAPIC_RRD                 0xC0  // Remote Read寄存器
#define LAPIC_LDR                 0xD0  // Logical Destination寄存器
#define LAPIC_DFR                 0xE0  // Destination Format寄存器
#define LAPIC_SVR                 0xF0  // Spurious Interrupt Vector寄存器
#define LAPIC_ISR                 0x100 // In-Service寄存器
#define LAPIC_TMR                 0x180 // Trigger Mode寄存器
#define LAPIC_IRR                 0x200 // Interrupt Request寄存器
#define LAPIC_ESR                 0x280 // Error Status寄存器
#define LAPIC_ICR_LOW             0x300 // Interrupt Command寄存器低位
#define LAPIC_ICR_HIGH            0x310 // Interrupt Command寄存器高位
#define LAPIC_LVT_TIMER           0x320 // LVT Timer寄存器
#define LAPIC_LVT_THERMAL         0x330 // LVT Thermal Sensor寄存器
#define LAPIC_LVT_PERF            0x340 // LVT Performance Counter寄存器
#define LAPIC_LVT_LINT0           0x350 // LVT LINT0寄存器
#define LAPIC_LVT_LINT1           0x360 // LVT LINT1寄存器
#define LAPIC_LVT_ERROR           0x370 // LVT Error寄存器
#define LAPIC_TIMER_INITIAL_COUNT 0x380 // Timer Initial Count寄存器
#define LAPIC_TIMER_CURRENT_COUNT 0x390 // Timer Current Count寄存器
#define LAPIC_TIMER_DIVIDE_CONFIG 0x3E0 // Timer Divide Configuration寄存器

// 本地APIC標誌位
#define LAPIC_SVR_ENABLE          0x100  // APIC軟件使能位

// LVT傳遞模式
#define LAPIC_LVT_DELIVERY_FIXED    0x000
#define LAPIC_LVT_DELIVERY_SMI      0x200
#define LAPIC_LVT_DELIVERY_NMI      0x400
#define LAPIC_LVT_DELIVERY_EXTINT   0x700
#define LAPIC_LVT_DELIVERY_INIT     0x500
#define LAPIC_LVT_DELIVERY_STARTUP  0x600

// LVT掩碼位
#define LAPIC_LVT_MASKED           0x10000

// APIC定時器模式
#define LAPIC_TIMER_ONESHOT        0x00000
#define LAPIC_TIMER_PERIODIC       0x20000
#define LAPIC_TIMER_TSC_DEADLINE   0x40000

// ICR傳遞模式
#define LAPIC_ICR_DELIVERY_FIXED    0x000
#define LAPIC_ICR_DELIVERY_LOWEST   0x100
#define LAPIC_ICR_DELIVERY_SMI      0x200
#define LAPIC_ICR_DELIVERY_NMI      0x400
#define LAPIC_ICR_DELIVERY_INIT     0x500
#define LAPIC_ICR_DELIVERY_STARTUP  0x600

// ICR目標模式
#define LAPIC_ICR_PHYSICAL         0x00000
#define LAPIC_ICR_LOGICAL          0x00800

// ICR級別
#define LAPIC_ICR_LEVEL_ASSERT     0x04000
#define LAPIC_ICR_LEVEL_DEASSERT   0x00000

// ICR觸發模式
#define LAPIC_ICR_TRIGGER_EDGE     0x00000
#define LAPIC_ICR_TRIGGER_LEVEL    0x08000

// ICR狀態
#define LAPIC_ICR_STATUS_IDLE      0x00000
#define LAPIC_ICR_STATUS_PENDING   0x01000

// ICR目標簡寫
#define LAPIC_ICR_DESTINATION_SELF         0x40000
#define LAPIC_ICR_DESTINATION_ALL          0x80000
#define LAPIC_ICR_DESTINATION_ALL_BUT_SELF 0xC0000

/**
 * @brief 初始化Local APIC
 * 
 * @param lapic_addr Local APIC物理地址
 * @return int 成功返回1，失敗返回0
 */
int lapic_init(uintptr_t lapic_addr);

/**
 * @brief 讀取Local APIC寄存器
 * 
 * @param reg 寄存器偏移量
 * @return uint32_t 寄存器值
 */
uint32_t lapic_read(uint32_t reg);

/**
 * @brief 寫入Local APIC寄存器
 * 
 * @param reg 寄存器偏移量
 * @param value 寄存器值
 */
void lapic_write(uint32_t reg, uint32_t value);

/**
 * @brief 獲取Local APIC ID
 * 
 * @return uint32_t APIC ID
 */
uint32_t lapic_get_id();

/**
 * @brief 發送EOI(End of Interrupt)信號
 */
void lapic_send_eoi();

/**
 * @brief 配置Local APIC定時器
 * 
 * @param vector 中斷向量
 * @param mode 定時器模式
 * @param initial_count 初始計數值
 */
void lapic_configure_timer(uint8_t vector, uint32_t mode, uint32_t initial_count);

/**
 * @brief 停止Local APIC定時器
 */
void lapic_stop_timer();

/**
 * @brief 配置Local APIC中斷
 * 
 * @param lint LINT編號 (0或1)
 * @param vector 中斷向量
 * @param delivery_mode 傳遞模式
 * @param masked 是否被掩蔽
 */
void lapic_configure_lint(int lint, uint8_t vector, uint32_t delivery_mode, int masked);

/**
 * @brief 發送處理器間中斷(IPI)
 * 
 * @param dest_apic_id 目標APIC ID
 * @param delivery_mode 傳遞模式
 * @param vector 中斷向量
 */
void lapic_send_ipi(uint8_t dest_apic_id, uint32_t delivery_mode, uint8_t vector);

/**
 * @brief 廣播處理器間中斷(IPI)到所有處理器
 * 
 * @param delivery_mode 傳遞模式
 * @param vector 中斷向量
 * @param exclude_self 是否排除自己
 */
void lapic_broadcast_ipi(uint32_t delivery_mode, uint8_t vector, int exclude_self);

/**
 * @brief 禁用PIC(8259A)
 * 必須在啟用APIC前禁用舊的PIC控制器
 */
void pic_disable();

#endif /* _SYSTEM_LAPIC_H */
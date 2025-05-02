#ifndef _SYSTEM_APIC_H
#define _SYSTEM_APIC_H

#include <stdint.h>

/**
 * @brief 初始化APIC子系統
 * 
 * @return int 成功返回1，失敗返回0
 */
int apic_init();

/**
 * @brief 讀取本地APIC寄存器 (兼容舊代碼)
 * 
 * @param reg 寄存器偏移量
 * @return uint32_t 寄存器值
 */
uint32_t apic_read(uint32_t reg);

/**
 * @brief 寫入本地APIC寄存器 (兼容舊代碼)
 * 
 * @param reg 寄存器偏移量
 * @param value 寄存器值
 */
void apic_write(uint32_t reg, uint32_t value);

/**
 * @brief 獲取本地APIC ID (兼容舊代碼)
 * 
 * @return uint32_t APIC ID
 */
uint32_t apic_get_id();

/**
 * @brief 發送EOI(End of Interrupt)信號 (兼容舊代碼)
 */
void apic_send_eoi();

/**
 * @brief 配置本地APIC定時器 (兼容舊代碼)
 * 
 * @param vector 中斷向量
 * @param mode 定時器模式
 * @param initial_count 初始計數值
 */
void apic_configure_timer(uint8_t vector, uint32_t mode, uint32_t initial_count);

/**
 * @brief 停止本地APIC定時器 (兼容舊代碼)
 */
void apic_stop_timer();

/**
 * @brief 配置本地APIC中斷 (兼容舊代碼)
 * 
 * @param lint LINT編號 (0或1)
 * @param vector 中斷向量
 * @param delivery_mode 傳遞模式
 * @param masked 是否被掩蔽
 */
void apic_configure_lint(int lint, uint8_t vector, uint32_t delivery_mode, int masked);

/**
 * @brief 發送處理器間中斷(IPI) (兼容舊代碼)
 * 
 * @param dest_apic_id 目標APIC ID
 * @param delivery_mode 傳遞模式
 * @param vector 中斷向量
 */
void apic_send_ipi(uint8_t dest_apic_id, uint32_t delivery_mode, uint8_t vector);

/**
 * @brief 廣播處理器間中斷(IPI)到所有處理器 (兼容舊代碼)
 * 
 * @param delivery_mode 傳遞模式
 * @param vector 中斷向量
 * @param exclude_self 是否排除自己
 */
void apic_broadcast_ipi(uint32_t delivery_mode, uint8_t vector, int exclude_self);

/**
 * @brief 取得已映射的 IO APIC 基址 (兼容舊代碼)
 *
 * @return uintptr_t IO APIC的虛擬基址
 */
uintptr_t apic_get_io_apic_base(void);

#endif /* _SYSTEM_APIC_H */
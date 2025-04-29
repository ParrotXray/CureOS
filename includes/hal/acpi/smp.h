#ifndef _SYSTEM_SMP_H
#define _SYSTEM_SMP_H

#include <stdint.h>

// SMP (對稱多處理) 相關定義

// CPU信息結構
typedef struct {
    uint8_t present;           // CPU是否存在
    uint8_t is_bsp;            // 是否是BSP(啟動處理器)
    uint8_t apic_id;           // 本地APIC ID
    uint8_t acpi_id;           // ACPI處理器ID
    uint32_t lapic_version;    // 本地APIC版本
} cpu_info_t;

/**
 * @brief 初始化SMP子系統
 * 
 * @return int 成功返回1，失敗返回0
 */
int smp_init();

/**
 * @brief 啟動所有應用處理器
 * 
 * @return int 成功返回1，失敗返回0
 */
int smp_start_aps();

/**
 * @brief 獲取當前CPU的APIC ID
 * 
 * @return uint8_t APIC ID
 */
uint8_t smp_get_current_apic_id();

/**
 * @brief 獲取CPU數量
 * 
 * @return int CPU數量
 */
int smp_get_cpu_count();

/**
 * @brief 獲取指定索引的CPU信息
 * 
 * @param index CPU索引
 * @return cpu_info_t* CPU信息，失敗則NULL
 */
cpu_info_t* smp_get_cpu_info(int index);

/**
 * @brief 發送處理器間中斷到指定CPU
 * 
 * @param cpu_index 目標CPU索引
 * @param vector 中斷向量
 * @return int 成功返回1，失敗返回0
 */
int smp_send_ipi(int cpu_index, uint8_t vector);

/**
 * @brief 廣播處理器間中斷到所有CPU
 * 
 * @param vector 中斷向量
 * @param exclude_self 是否排除自己
 */
void smp_broadcast_ipi(uint8_t vector, int exclude_self);

/**
 * @brief AP處理器準備就緒時調用
 * 在AP的啟動代碼中調用此函數通知BSP
 */
void smp_ap_ready();

#endif /* _SYSTEM_SMP_H */
#ifndef _SYSTEM_APIC_TIMER_H
#define _SYSTEM_APIC_TIMER_H

#include <stdint.h>

// APIC Timer相關常數
#define APIC_TIMER_VECTOR    0x40
#define APIC_TIMER_IRQ       0

// 校準相關常數
#define CALIBRATION_LOOPS    10000000
#define PIT_FREQ            1193182
#define PIT_DIVISOR         1193
#define PIT_FREQUENCY       (PIT_FREQ / PIT_DIVISOR) // 約1000Hz

// PIT相關端口
#define PIT_CHANNEL0        0x40
#define PIT_CHANNEL1        0x41
#define PIT_CHANNEL2        0x42
#define PIT_COMMAND         0x43

/**
 * @brief 初始化並校準APIC計時器
 */
void apic_timer_init();

/**
 * @brief 獲取系統啟動後的毫秒數
 * 
 * @return uint32_t 毫秒數
 */
uint32_t apic_timer_get_ms();

/**
 * @brief 暫停指定的毫秒數
 * 
 * @param ms 毫秒數
 */
void apic_timer_sleep(uint32_t ms);

/**
 * @brief 獲取當前計時器計數值
 * 
 * @return uint32_t 計數值
 */
uint32_t apic_timer_get_current_count();

/**
 * @brief 設置計時器的中斷頻率
 * 
 * @param hz 頻率（赫茲）
 */
void apic_timer_set_frequency(uint32_t hz);


#endif /* _SYSTEM_APIC_TIMER_H */
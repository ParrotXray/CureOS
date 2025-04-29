#ifndef __SYSTEM_RTC_H
#define __SYSTEM_RTC_H

#include <hal/io.h>
#include <stdint.h>

// RTC 連接埠
#define RTC_INDEX_PORT 0x70
#define RTC_TARGET_PORT 0x71

// NMI 控制位
#define WITH_NMI_DISABLED 0x80

// 世紀常數
#define RTC_CURRENT_CENTURY 20

// RTC 日期時間寄存器
#define RTC_REG_SEC 0x00
#define RTC_REG_MIN 0x02
#define RTC_REG_HRS 0x04
#define RTC_REG_WDY 0x06  // 星期幾 (1-7)
#define RTC_REG_DAY 0x07  // 日期 (1-31)
#define RTC_REG_MTH 0x08  // 月份 (1-12)
#define RTC_REG_YRS 0x09  // 年 (0-99)

// RTC 控制寄存器
#define RTC_REG_A 0x0A
#define RTC_REG_B 0x0B
#define RTC_REG_C 0x0C
#define RTC_REG_D 0x0D

// 寄存器 B 資料格式位
#define RTC_BIN_ENCODED(reg)    ((reg) & 0x04)
#define RTC_24HRS_ENCODED(reg)  ((reg) & 0x02)

// RTC 定時器相關
#define RTC_TIMER_BASE_FREQUENCY    1024
#define RTC_TIMER_ON                0x40

// 頻率設定 (寄存器 A)
#define RTC_FREQUENCY_1024HZ    0b110
#define RTC_DIVIDER_33KHZ       (0b010 << 4)

// 日期時間結構
typedef struct {
    uint8_t second;
    uint8_t minute;
    uint8_t hour;
    uint8_t day;
    uint8_t month;
    uint16_t year;
    uint8_t weekday;
} rtc_datetime_t;

/**
 * @brief 初始化 RTC 
 */
void rtc_init();

/**
 * @brief 讀取 RTC 寄存器
 *
 * @param reg_selector 寄存器選擇器
 * @return uint8_t 寄存器值
 */
uint8_t rtc_read_reg(uint8_t reg_selector);

/**
 * @brief 寫入 RTC 寄存器
 *
 * @param reg_selector 寄存器選擇器
 * @param val 要寫入的值
 */
void rtc_write_reg(uint8_t reg_selector, uint8_t val);

/**
 * @brief 從 BCD 轉換為十進制
 *
 * @param bcd BCD 編碼值
 * @return uint8_t 十進制值
 */
uint8_t bcd2dec(uint8_t bcd);

/**
 * @brief 從十進制轉換為 BCD
 *
 * @param dec 十進制值
 * @return uint8_t BCD 編碼值
 */
uint8_t dec2bcd(uint8_t dec);

/**
 * @brief 啟用 RTC 定時器
 */
void rtc_enable_timer();

/**
 * @brief 停用 RTC 定時器
 */
void rtc_disable_timer();

/**
 * @brief 設置 RTC 定時器頻率
 *
 * @param frequency 頻率選擇器 (1-15)
 */
void rtc_set_timer_frequency(uint8_t frequency);

/**
 * @brief 讀取當前 RTC 日期時間
 *
 * @param datetime 輸出日期時間結構
 */
void rtc_get_datetime(rtc_datetime_t* datetime);

/**
 * @brief 設置 RTC 日期時間
 *
 * @param datetime 日期時間結構
 */
void rtc_set_datetime(rtc_datetime_t* datetime);

/**
 * @brief 檢查 RTC 更新是否完成
 *
 * @return int 1 表示更新完成，0 表示更新進行中
 */
int rtc_is_update_done();

/**
 * @brief 獲取系統時間戳（自 1970-01-01 00:00:00 以來的秒數）
 * @return uint32_t 時間戳
 */
uint32_t rtc_get_timestamp();


#endif /* __SYSTEM_RTC_H */
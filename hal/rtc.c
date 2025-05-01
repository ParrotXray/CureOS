/**
 * @file rtc.c
 * @brief RTC & CMOS 驅動實現
 * @version 0.1
 * @date 2025-04-29
 * 
 * 參考: MC146818A & Intel Series 500 PCH datasheet
 */
#include <hal/rtc.h>
#include <hal/apic/apic.h>
#include <arch/x86/interrupts.h>
#include <libc/string.h>
#include <libc/stdio.h>

// RTC 中斷向量
#define RTC_IRQ         8
#define RTC_INT_VECTOR  (32 + RTC_IRQ)

// 紀錄上次讀取的時間
static rtc_datetime_t last_read_time;

// RTC 中斷處理程序 
static void rtc_irq_handler(isr_param* param);

void
rtc_init() {
    // 讀取寄存器 A
    uint8_t regA = rtc_read_reg(RTC_REG_A | WITH_NMI_DISABLED);
    
    // 設置時鐘頻率 (1024Hz) 和分頻器 (33KHz)
    regA = (regA & ~0x7f) | RTC_FREQUENCY_1024HZ | RTC_DIVIDER_33KHZ;
    rtc_write_reg(RTC_REG_A | WITH_NMI_DISABLED, regA);

    // 讀取寄存器 B
    uint8_t regB = rtc_read_reg(RTC_REG_B | WITH_NMI_DISABLED);
    
    // 確保二進制模式(DM=1)和24小時制(24/12=1)
    regB |= 0x04;  // 二進制模式 
    regB |= 0x02;  // 24小時制
    
    // 啟用周期性中斷 (PIE=1)
    regB |= 0x40;
    
    // printf("[RTC] Setting REG_B to 0x%02x\n", regB);
    
    // 寫回寄存器 B
    rtc_write_reg(RTC_REG_B | WITH_NMI_DISABLED, regB);

    // 檢查設置是否成功
    regB = rtc_read_reg(RTC_REG_B | WITH_NMI_DISABLED);
    // printf("[RTC] REG_B after write: 0x%02x\n", regB);

    // 註冊 IRQ 處理程序
    register_irq_handler(RTC_IRQ, rtc_irq_handler);
    
    // 讀取並清除寄存器 C (確保中斷正常工作)
    rtc_read_reg(RTC_REG_C);
    
    // 預先讀取一次時間
    rtc_get_datetime(&last_read_time);
    
    // 啟用 RTC IRQ
    if (acpi_is_supported()) {
        // 使用 IO APIC 啟用 IRQ
        enable_irq(RTC_IRQ);
    }
    
    // 顯示日期時間
    char time_str[32];
    sprintf(time_str, "%u:%u:%u %u/%u/%u", 
            last_read_time.hour, last_read_time.minute, last_read_time.second,
            last_read_time.day, last_read_time.month, last_read_time.year);
    printf("[RTC] RTC initialized, current time: %s\n", time_str);
        
    // 默認情況下停用定時器
    rtc_disable_timer();
}

uint8_t
rtc_read_reg(uint8_t reg_selector)
{
    io_port_wb(RTC_INDEX_PORT, reg_selector);
    return io_port_rb(RTC_TARGET_PORT);
}

void
rtc_write_reg(uint8_t reg_selector, uint8_t val)
{
    io_port_wb(RTC_INDEX_PORT, reg_selector);
    io_port_wb(RTC_TARGET_PORT, val);
}

uint8_t
bcd2dec(uint8_t bcd)
{
    return ((bcd & 0xF0) >> 4) * 10 + (bcd & 0x0F);
}

uint8_t
dec2bcd(uint8_t dec)
{
    return ((dec / 10) << 4) | (dec % 10);
}

void
rtc_enable_timer() {
    uint8_t regB = rtc_read_reg(RTC_REG_B | WITH_NMI_DISABLED);
    rtc_write_reg(RTC_REG_B | WITH_NMI_DISABLED, regB | RTC_TIMER_ON);
    printf("[RTC] RTC timer enabled\n");
}

void
rtc_disable_timer() {
    uint8_t regB = rtc_read_reg(RTC_REG_B | WITH_NMI_DISABLED);
    rtc_write_reg(RTC_REG_B | WITH_NMI_DISABLED, regB & ~RTC_TIMER_ON);
    printf("[RTC] RTC timer disabled\n");
}

void
rtc_set_timer_frequency(uint8_t frequency) {
    // 頻率值範圍 0-15，對應頻率 2^freq Hz
    frequency &= 0x0F;
    
    uint8_t regA = rtc_read_reg(RTC_REG_A | WITH_NMI_DISABLED);
    regA = (regA & 0xF0) | frequency;
    rtc_write_reg(RTC_REG_A | WITH_NMI_DISABLED, regA);
    
    // 計算實際頻率
    int actual_freq = 1 << frequency;
    if (frequency == 0) actual_freq = 0;  // 特殊情況，頻率為 0 表示關閉
    
    printf("[RTC] Timer frequency set to %d Hz\n", actual_freq);
}

int rtc_is_update_done() {
    // 檢查寄存器 A 的 UIP (更新中) 位
    uint8_t regA = rtc_read_reg(RTC_REG_A | WITH_NMI_DISABLED);
    return !(regA & 0x80);  // 如果 UIP=0，則更新完成
}

void rtc_get_datetime(rtc_datetime_t* datetime) {
    // 等待任何正在進行的 RTC 更新完成
    while (!rtc_is_update_done()) {
        __asm__ volatile("pause");
    }
    
    // 讀取寄存器 B 來確定格式
    uint8_t regB = rtc_read_reg(RTC_REG_B | WITH_NMI_DISABLED);
    
    // 讀取時間日期寄存器
    uint8_t seconds = rtc_read_reg(RTC_REG_SEC | WITH_NMI_DISABLED);
    uint8_t minutes = rtc_read_reg(RTC_REG_MIN | WITH_NMI_DISABLED);
    uint8_t hours = rtc_read_reg(RTC_REG_HRS | WITH_NMI_DISABLED);
    uint8_t day = rtc_read_reg(RTC_REG_DAY | WITH_NMI_DISABLED);
    uint8_t month = rtc_read_reg(RTC_REG_MTH | WITH_NMI_DISABLED);
    uint8_t year = rtc_read_reg(RTC_REG_YRS | WITH_NMI_DISABLED);
    uint8_t weekday = rtc_read_reg(RTC_REG_WDY | WITH_NMI_DISABLED);
    
    // 輸出原始值
    // printf("[RTC] Raw values - Seconds: 0x%x, Minutes: 0x%x, Hours: 0x%x\n", 
    //        seconds, minutes, hours);
    // printf("[RTC] Raw values - Day: 0x%x, Month: 0x%x, Year: 0x%x, Format: 0x%x\n",
    //        day, month, year, regB);
    
    // 若為 BCD 格式轉為二進制
    if (!(regB & 0x04)) {  // 檢查 DM 位 (Data Mode)
        seconds = bcd2dec(seconds);
        minutes = bcd2dec(minutes);
        hours = bcd2dec(hours);
        day = bcd2dec(day);
        month = bcd2dec(month);
        year = bcd2dec(year);
    }
    
    // 處理 12 小時制轉換為 24 小時制
    if (!(regB & 0x02) && (hours & 0x80)) {  // 檢查 24/12 位
        hours = ((hours & 0x7F) + 12) % 24;
    }
    
    // 填充日期時間結構
    datetime->second = seconds;
    datetime->minute = minutes;
    datetime->hour = hours;
    datetime->day = day;
    datetime->month = month;
    datetime->weekday = weekday;

    if (year < 70) {
        datetime->year = 2000 + year;  // 2000-2069
    } else {
        datetime->year = 1900 + year;  // 1970-1999
    }
    
    // 輸出轉換後的值
    // printf("[RTC] Converted - Seconds: %u, Minutes: %u, Hours: %u\n", 
    //        seconds, minutes, hours);
    // printf("[RTC] Converted - Day: %u, Month: %u, Year: %u\n",
    //        day, month, year);
}

void rtc_set_datetime(rtc_datetime_t* datetime) {
    // 等待任何正在進行的 RTC 更新完成
    while (!rtc_is_update_done()) {
        __asm__ volatile("pause");
    }
    
    // 停用 NMI 和週期中斷
    uint8_t regB = rtc_read_reg(RTC_REG_B | WITH_NMI_DISABLED);
    rtc_write_reg(RTC_REG_B | WITH_NMI_DISABLED, regB & ~RTC_TIMER_ON);
    
    // 讀取寄存器 B 來確定格式
    uint8_t format = regB;
    
    // 準備數值
    uint8_t seconds = datetime->second;
    uint8_t minutes = datetime->minute;
    uint8_t hours = datetime->hour;
    uint8_t day = datetime->day;
    uint8_t month = datetime->month;
    uint8_t year = datetime->year % 100;  // 僅取後兩位數字
    uint8_t weekday = datetime->weekday;
    
    // 若需要轉換為 BCD 格式
    if (!(format & 0x04)) {
        seconds = dec2bcd(seconds);
        minutes = dec2bcd(minutes);
        hours = dec2bcd(hours);
        day = dec2bcd(day);
        month = dec2bcd(month);
        year = dec2bcd(year);
    }
    
    // 寫入時間日期寄存器
    rtc_write_reg(RTC_REG_SEC | WITH_NMI_DISABLED, seconds);
    rtc_write_reg(RTC_REG_MIN | WITH_NMI_DISABLED, minutes);
    rtc_write_reg(RTC_REG_HRS | WITH_NMI_DISABLED, hours);
    rtc_write_reg(RTC_REG_DAY | WITH_NMI_DISABLED, day);
    rtc_write_reg(RTC_REG_MTH | WITH_NMI_DISABLED, month);
    rtc_write_reg(RTC_REG_YRS | WITH_NMI_DISABLED, year);
    rtc_write_reg(RTC_REG_WDY | WITH_NMI_DISABLED, weekday);
    
    // 恢復原始設定
    rtc_write_reg(RTC_REG_B | WITH_NMI_DISABLED, regB);
    
    // 顯示日期時間
    char time_str[32];
    sprintf(time_str, "%u:%u:%u %u/%u/%u", 
            datetime->hour, datetime->minute, datetime->second,
            datetime->day, datetime->month, datetime->year);
    printf("[RTC] Date/time set to %s\n", time_str);
}

// RTC 中斷處理程序
static void rtc_irq_handler(isr_param* param) {
    static uint32_t tick_count = 0;
    
    // 讀取寄存器 C，這會清除中斷標誌
    // 必須執行此操作，否則不會收到更多中斷
    rtc_read_reg(RTC_REG_C);
    
    // 每秒更新一次時間 (假設 1024Hz 的中斷頻率)
    if (++tick_count >= 1024) {
        tick_count = 0;
        
        // 更新時間
        rtc_get_datetime(&last_read_time);
    }
    
    // 若使用 APIC，發送 EOI
    if (acpi_is_supported()) {
        apic_send_eoi();
    }
}

// 每月天數（非閏年）
static const uint8_t days_in_month[12] = {
    31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31
};

// 判斷閏年
static int is_leap_year(uint16_t year) {
    return (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
}

uint32_t rtc_get_timestamp() {
    rtc_datetime_t dt;
    rtc_get_datetime(&dt);
    
    // 計算從 1970 年到當前年份的秒數
    uint32_t timestamp = 0;
    uint16_t year = 1970;
    
    // 計算整年的秒數
    while (year < dt.year) {
        timestamp += is_leap_year(year) ? 31622400 : 31536000;
        year++;
    }
    
    // 計算今年已過月份的秒數
    for (uint8_t m = 1; m < dt.month; m++) {
        uint8_t days = days_in_month[m-1];
        if (m == 2 && is_leap_year(dt.year)) days++; // 閏年二月
        timestamp += days * 86400;
    }
    
    // 計算本月已過天數的秒數
    timestamp += (dt.day - 1) * 86400;
    
    // 計算今天已過小時的秒數
    timestamp += dt.hour * 3600;
    
    // 計算當前小時已過分鐘的秒數
    timestamp += dt.minute * 60;
    
    // 加上當前秒數
    timestamp += dt.second;
    
    return timestamp;
}
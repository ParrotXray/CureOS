#ifndef _SYSTEM_CPU_H
#define _SYSTEM_CPU_H

#include <stdint.h>

typedef unsigned long reg32;
typedef unsigned short reg16;

typedef struct {
    reg32 edi;
    reg32 esi;
    reg32 ebp;
    reg32 esp;
    reg32 ebx;
    reg32 edx;
    reg32 ecx;
    reg32 eax;
} __attribute__((packed)) registers;

reg32 cpu_r_cr0();

reg32 cpu_r_cr2();

reg32 cpu_r_cr3();

void cpu_w_cr0(reg32 v);

void cpu_w_cr2(reg32 v);

void cpu_w_cr3(reg32 v);

void cpu_get_model(char* model_out);

int cpu_brand_string_supported();

void cpu_get_brand(char* brand_out);

/**
 * @brief 讀取CPU的時間戳計數器(TSC)
 * 
 * @return uint64_t 時間戳計數值
 */
static inline uint64_t cpu_rdtsc() {
    uint32_t low, high;
    __asm__ volatile("rdtsc" : "=a" (low), "=d" (high));
    return ((uint64_t)high << 32) | low;
}

/**
 * @brief 執行CPU暫停指令
 * 
 * 在等待時減少CPU功耗
 */
static inline void cpu_pause() {
    __asm__ volatile("pause");
}

/**
 * @brief 停止CPU執行，直到下一個中斷發生
 */
static inline void cpu_halt() {
    __asm__ volatile("hlt");
}

/**
 * @brief 停止CPU並進入低功耗模式
 */
static inline void cpu_idle() {
    __asm__ volatile("hlt");
}

/**
 * @brief 啟用中斷
 */
static inline void cpu_enable_interrupts() {
    __asm__ volatile("sti");
}

/**
 * @brief 禁用中斷
 */
static inline void cpu_disable_interrupts() {
    __asm__ volatile("cli");
}

#endif /* _SYSTEM_CPU_H */
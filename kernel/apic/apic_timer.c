#include <system/apic/apic.h>
#include <system/acpi/acpi.h>
#include <system/apic/apic_timer.h>
#include <arch/x86/interrupts.h>
#include <libc/stdio.h>
#include <hal/cpu.h>

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

// 計時器變數
static volatile uint32_t timer_ticks = 0;
static uint32_t timer_frequency = 0;      // Hz
static uint32_t ticks_per_ms = 0;

// 計時器中斷處理程序
static void apic_timer_handler(isr_param* param) {
    timer_ticks++;
    
    // 每秒顯示一次計時器狀態
    if (timer_ticks % timer_frequency == 0) {
        uint32_t seconds = timer_ticks / timer_frequency;
        printf("[APIC Timer] Uptime: %u seconds\n", seconds);
    }
    
    // 發送EOI
    apic_send_eoi();
}

// 使用PIT校準APIC計時器
static void apic_timer_calibrate() {
    printf("[APIC Timer] Calibrating APIC timer using PIT...\n");
    
    // 初始化PIT通道0為模式2 (週期模式)
    io_port_wb(PIT_COMMAND, 0x34);  // 通道0, 存取低/高位元組, 模式2, 二進制計數
    io_port_wb(PIT_CHANNEL0, PIT_DIVISOR & 0xFF);         // 低位元組
    io_port_wb(PIT_CHANNEL0, (PIT_DIVISOR >> 8) & 0xFF);  // 高位元組
    
    // 設置APIC計時器為計數模式
    apic_write(APIC_TIMER_DIVIDE_CONFIG, 0xB);  // 除數為1
    apic_write(APIC_LVT_TIMER, APIC_LVT_MASKED);  // 掩蔽計時器
    apic_write(APIC_TIMER_INITIAL_COUNT, 0xFFFFFFFF);  // 初始計數最大值
    
    // 等待一定的PIT計數週期
    uint64_t start_time = cpu_rdtsc();
    
    // 等待10ms
    for (volatile int i = 0; i < CALIBRATION_LOOPS; i++) {
        if ((i % (CALIBRATION_LOOPS / 10)) == 0) {
            printf(".");
        }
    }
    
    // 停止APIC計時器
    apic_write(APIC_LVT_TIMER, APIC_LVT_MASKED);
    
    // 讀取剩餘計數
    uint32_t apic_ticks = 0xFFFFFFFF - apic_read(APIC_TIMER_CURRENT_COUNT);
    
    uint64_t end_time = cpu_rdtsc();
    uint64_t tsc_ticks = end_time - start_time;
    
    printf("\n[APIC Timer] Calibration complete:\n");
    printf("[APIC Timer] APIC ticks: %u\n", apic_ticks);
    printf("[APIC Timer] TSC ticks: %llu\n", tsc_ticks);
    
    // 計算APIC計時器頻率，假設校準期間是10ms
    timer_frequency = apic_ticks * 100;  // 轉換為每秒頻率
    ticks_per_ms = apic_ticks / 10;      // 每毫秒的計數值
    
    printf("[APIC Timer] Frequency: %u Hz\n", timer_frequency);
    printf("[APIC Timer] Ticks per ms: %u\n", ticks_per_ms);
}

// 初始化並校準APIC計時器
void apic_timer_init() {
    printf("[APIC Timer] Initializing APIC timer...\n");
    
    // 確保APIC已初始化
    if (!acpi_is_supported()) {
        printf("[APIC Timer] ACPI not supported, cannot initialize APIC timer\n");
        return;
    }
    
    // 註冊計時器中斷處理程序
    register_irq_handler(APIC_TIMER_IRQ, apic_timer_handler);
    
    // 校準APIC計時器
    apic_timer_calibrate();
    
    // 設置計時器使用週期模式
    printf("[APIC Timer] Setting up APIC timer in periodic mode\n");
    apic_configure_timer(APIC_TIMER_VECTOR, APIC_TIMER_PERIODIC, ticks_per_ms * 10);  // 10ms中斷
    
    printf("[APIC Timer] APIC timer initialized at %u.%u MHz\n", 
           timer_frequency / 1000000, (timer_frequency % 1000000) / 1000);
}

// 獲取系統啟動後的毫秒數
uint32_t apic_timer_get_ms() {
    if (ticks_per_ms == 0) return 0;  // 計時器未初始化
    return timer_ticks * 10;  // 10ms每次中斷
}

// 暫停指定的毫秒數
void apic_timer_sleep(uint32_t ms) {
    uint32_t target_ticks = timer_ticks + (ms / 10);  // 10ms每次中斷
    while (timer_ticks < target_ticks) {
        __asm__ volatile("hlt");  // 處理器閒置，等待下一個中斷
    }
}

// 獲取當前計時器計數值
uint32_t apic_timer_get_current_count() {
    return apic_read(APIC_TIMER_CURRENT_COUNT);
}

// 設置計時器的中斷頻率
void apic_timer_set_frequency(uint32_t hz) {
    if (hz == 0 || ticks_per_ms == 0) return;
    
    uint32_t ms_per_interrupt = 1000 / hz;
    uint32_t ticks = ticks_per_ms * ms_per_interrupt;
    
    apic_configure_timer(APIC_TIMER_VECTOR, APIC_TIMER_PERIODIC, ticks);
    
    printf("[APIC Timer] Timer frequency set to %u Hz (%u ms per interrupt)\n", 
           hz, ms_per_interrupt);
}
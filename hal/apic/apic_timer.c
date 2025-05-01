#include <hal/apic/apic.h>
#include <hal/acpi/acpi.h>
#include <hal/apic/apic_timer.h>
#include <arch/x86/interrupts.h>
#include <libc/stdio.h>
#include <hal/cpu.h>

// 計時器變數
volatile uint32_t timer_ticks = 0;
uint32_t timer_frequency = 0;      // Hz
uint32_t ticks_per_ms = 0;

// 使用PIT校準APIC計時器
void apic_timer_calibrate() {
    printf("[APIC Timer] Calibrating APIC timer using PIT...\n");
    
    // 初始化 PIT 通道 0 為模式 2 (週期模式)，頻率約為 100Hz (10ms 一次)
    io_port_wb(PIT_COMMAND, 0x34);
    io_port_wb(PIT_CHANNEL0, 0x9C);  // 低位元組 (11932 & 0xFF)
    io_port_wb(PIT_CHANNEL0, 0x2E);  // 高位元組 (11932 >> 8)
    
    // 設置 APIC 計時器為單次計數模式
    apic_write(APIC_TIMER_DIVIDE_CONFIG, 0xB);  // 除數為 1
    apic_write(APIC_LVT_TIMER, APIC_LVT_MASKED);  // 掩蔽計時器
    apic_write(APIC_TIMER_INITIAL_COUNT, 0xFFFFFFFF);  // 初始計數最大值
    
    // 使用 PIT 計時，等待 100ms (10 次 PIT 中斷)
    // 讀取 PIT 通道 0 的計數值 (注意：PIT 是遞減計數器)
    uint8_t pit_status;
    int pit_cycles = 0;
    const int CALIBRATION_CYCLES = 10;  // 要等待的 PIT 週期數
    
    printf("[APIC Timer] Waiting for %d PIT cycles (about 100ms)...\n", CALIBRATION_CYCLES);
    
    // 等待 PIT 計時器完成一個週期的方法：讀取狀態端口，檢查 OUT 位 (bit 7)
    io_port_wb(PIT_COMMAND, 0xE2);  // 讀取通道 0 狀態命令
    pit_status = io_port_rb(PIT_CHANNEL0);  // 讀取狀態
    
    // 等待幾個 PIT 週期
    while (pit_cycles < CALIBRATION_CYCLES) {
        uint8_t new_status;
        io_port_wb(PIT_COMMAND, 0xE2);
        new_status = io_port_rb(PIT_CHANNEL0);
        
        // 檢測 OUT 位的變化 (從 0 變為 1 表示一個週期完成)
        if ((new_status & 0x80) && !(pit_status & 0x80)) {
            pit_cycles++;
            if (pit_cycles % (CALIBRATION_CYCLES/5) == 0) {
                printf(".");
            }
        }
        pit_status = new_status;
    }
    printf("\n");
    
    // 停止 APIC 計時器
    apic_write(APIC_LVT_TIMER, APIC_LVT_MASKED);
    
    // 讀取剩餘計數
    uint32_t apic_ticks = 0xFFFFFFFF - apic_read(APIC_TIMER_CURRENT_COUNT);
    
    // 計算 APIC 計時器頻率 (100ms = CALIBRATION_CYCLES * 10ms)
    ticks_per_ms = apic_ticks / (CALIBRATION_CYCLES * 10);
    timer_frequency = ticks_per_ms * 1000;
    
    printf("[APIC Timer] APIC ticks during %dms: %u\n", CALIBRATION_CYCLES * 10, apic_ticks);
    printf("[APIC Timer] Ticks per ms: %u\n", ticks_per_ms);
    printf("[APIC Timer] Estimated frequency: %u Hz\n", timer_frequency);
    
    // 校準失敗的回退機制
    if (ticks_per_ms < 10) {  // 如果計算值不合理 (太小)
        printf("[APIC Timer] Calibration seems off, using default values\n");
        ticks_per_ms = 1000;   // 預設假設 1MHz
        timer_frequency = 1000000;
    }
}

// 修改APIC計時器初始化函數
void apic_timer_init() {
    printf("[APIC Timer] Initializing APIC timer...\n");
    
    // 確保APIC已初始化
    if (!acpi_is_supported()) {
        printf("[APIC Timer] ACPI not supported, cannot initialize APIC timer\n");
        return;
    }
    
    // 校準APIC計時器
    apic_timer_calibrate();
    
    // 設置計時器使用週期模式，每毫秒觸發一次中斷
    printf("[APIC Timer] Setting up APIC timer in periodic mode\n");
    apic_configure_timer(APIC_TIMER_VECTOR, APIC_TIMER_PERIODIC, ticks_per_ms);
    
    printf("[APIC Timer] APIC timer initialized at %u.%u MHz\n", 
           timer_frequency / 1000000, (timer_frequency % 1000000) / 1000);
}

// 獲取系統啟動後的毫秒數
uint32_t apic_timer_get_ms() {
    if (ticks_per_ms == 0) return 0;  // 計時器未初始化
    return timer_ticks;  // 10ms每次中斷
}

// 暫停指定的毫秒數
void apic_timer_sleep(uint32_t ms) {
    if (ticks_per_ms == 0) return;  // 計時器未初始化
    
    uint32_t start_ticks = timer_ticks;
    uint32_t target_ticks = timer_ticks + ms;
    
    printf("[APIC Timer] Sleeping for %u ms (from tick %u to %u)...\n", 
           ms, start_ticks, target_ticks);
    
    while (timer_ticks < target_ticks) {
        // printf("%d, %d\n", timer_ticks, target_ticks);
        __asm__ volatile("hlt");  // 處理器閒置，等待下一個中斷
    }
    
    printf("[APIC Timer] Sleep complete, actual ticks passed: %u\n", 
           timer_ticks - start_ticks);
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

// 用于在中断处理程序中递增计数器
void apic_timer_tick() {
    timer_ticks++;
}
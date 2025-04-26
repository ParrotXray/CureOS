#include <system/acpi/acpi.h>
#include <system/apic/apic.h>
#include <system/mm/vmm.h>
#include <system/mm/page.h>
#include <libc/stdio.h>
#include <libc/string.h>
#include <stdint.h>

// 定義AP(應用處理器)啟動代碼的地址
#define AP_BOOT_ADDR 0x8000

// 保存所有CPU狀態
#define MAX_CPU_COUNT 16

typedef struct {
    uint8_t present;           // CPU是否存在
    uint8_t is_bsp;            // 是否是BSP(啟動處理器)
    uint8_t apic_id;           // 本地APIC ID
    uint8_t acpi_id;           // ACPI處理器ID
    uint32_t lapic_version;    // 本地APIC版本
    // 可以添加更多狀態...
} cpu_info_t;

// CPU核心資訊
static cpu_info_t cpu_infos[MAX_CPU_COUNT];
static int cpu_count = 0;
static int bsp_apic_id = 0;

// AP啟動標誌，用於同步
static volatile int ap_ready_count = 0;

// 引導處理器初始化各應用處理器的入口點
extern void ap_entry();

// 啟動代碼模板
static unsigned char ap_boot_code[] = {
    0xFA,               // cli
    0x8C, 0xC8,         // mov ax, cs
    0x8E, 0xD8,         // mov ds, ax
    0x8E, 0xC0,         // mov es, ax
    0x8E, 0xD0,         // mov ss, ax
    
    // 設置棧指針
    0xBC, 0x00, 0x80, 0x00, 0x00,  // mov esp, 0x8000
    
    // 開啟分頁
    0x0F, 0x20, 0xC0,   // mov eax, cr0
    0x0D, 0x00, 0x00, 0x00, 0x80,  // or eax, 0x80000000
    0x0F, 0x22, 0xC0,   // mov cr0, eax
    
    // 跳轉到高半核
    0xEA,               // jmp far
    0x00, 0x00, 0x00, 0xC0,  // 目標地址 (將被替換)
    0x08, 0x00          // 代碼段選擇子 0x08
};

/**
 * @brief 初始化SMP子系統
 * 
 * @return int 成功返回1，失敗返回0
 */
int smp_init() {
    printf("[SMP] Initializing SMP subsystem\n");
    
    // 檢查ACPI支持
    if (!acpi_is_supported()) {
        printf("[SMP] ACPI not supported, cannot initialize SMP\n");
        return 0;
    }
    
    // 獲取CPU數量
    cpu_count = acpi_get_cpu_count();
    if (cpu_count <= 0) {
        printf("[SMP] No CPU detected\n");
        return 0;
    }
    
    printf("[SMP] Detected %d CPUs\n", cpu_count);
    
    // 初始化CPU信息
    memset(cpu_infos, 0, sizeof(cpu_infos));
    
    // 獲取當前CPU的APIC ID (BSP)
    bsp_apic_id = apic_get_id() >> 24;
    
    // 設置BSP信息
    for (int i = 0; i < cpu_count; i++) {
        int apic_id = acpi_get_cpu_apic_id(i);
        if (apic_id < 0) continue;
        
        cpu_infos[i].present = 1;
        cpu_infos[i].apic_id = apic_id;
        cpu_infos[i].acpi_id = i;
        
        // 標記BSP
        if (apic_id == bsp_apic_id) {
            cpu_infos[i].is_bsp = 1;
            printf("[SMP] CPU %d is BSP (APIC ID: %d)\n", i, apic_id);
        } else {
            cpu_infos[i].is_bsp = 0;
        }
    }
    
    return 1;
}

/**
 * @brief 啟動所有應用處理器
 * 
 * @return int 成功返回1，失敗返回0
 */
int smp_start_aps() {
    printf("[SMP] Starting application processors\n");
    
    // 檢查是否已初始化
    if (cpu_count <= 1) {
        printf("[SMP] No APs to start\n");
        return 0;
    }
    
    // 準備AP啟動代碼
    // 1. 複製啟動代碼到低地址
    memcpy((void*)P2V(AP_BOOT_ADDR), ap_boot_code, sizeof(ap_boot_code));
    
    // 2. 填寫跳轉地址 (ap_entry的虛擬地址轉換為物理地址)
    uintptr_t ap_entry_phys = V2P(ap_entry);
    *((uint32_t*)(P2V(AP_BOOT_ADDR) + 19)) = ap_entry_phys;
    
    // 重置計數器
    ap_ready_count = 0;
    
    // 啟動每個AP
    for (int i = 0; i < cpu_count; i++) {
        if (!cpu_infos[i].present || cpu_infos[i].is_bsp) continue;
        
        printf("[SMP] Starting CPU %d (APIC ID: %d)...\n", i, cpu_infos[i].apic_id);
        
        // 發送INIT IPI
        apic_send_ipi(cpu_infos[i].apic_id, APIC_ICR_DELIVERY_INIT, 0);
        
        // 延遲
        for (volatile int j = 0; j < 10000000; j++);
        
        // 發送STARTUP IPI (兩次，根據Intel文檔建議)
        apic_send_ipi(cpu_infos[i].apic_id, APIC_ICR_DELIVERY_STARTUP, AP_BOOT_ADDR >> 12);
        
        // 延遲
        for (volatile int j = 0; j < 1000000; j++);
        
        // 第二次STARTUP IPI
        apic_send_ipi(cpu_infos[i].apic_id, APIC_ICR_DELIVERY_STARTUP, AP_BOOT_ADDR >> 12);
        
        // 等待AP準備好
        volatile int timeout = 100000000;
        while (ap_ready_count <= i - 1 && timeout-- > 0) {
            __asm__ volatile("pause");
        }
        
        if (timeout <= 0) {
            printf("[SMP] Timeout waiting for CPU %d\n", i);
        } else {
            printf("[SMP] CPU %d started successfully\n", i);
        }
    }
    
    printf("[SMP] %d APs started successfully\n", ap_ready_count);
    return (ap_ready_count == cpu_count - 1);
}

/**
 * @brief 獲取當前CPU的APIC ID
 * 
 * @return uint8_t APIC ID
 */
uint8_t smp_get_current_apic_id() {
    return apic_get_id() >> 24;
}

/**
 * @brief 獲取CPU數量
 * 
 * @return int CPU數量
 */
int smp_get_cpu_count() {
    return cpu_count;
}

/**
 * @brief 獲取指定索引的CPU信息
 * 
 * @param index CPU索引
 * @return cpu_info_t* CPU信息，失敗則NULL
 */
cpu_info_t* smp_get_cpu_info(int index) {
    if (index < 0 || index >= cpu_count) {
        return NULL;
    }
    
    return &cpu_infos[index];
}

/**
 * @brief 發送處理器間中斷到指定CPU
 * 
 * @param cpu_index 目標CPU索引
 * @param vector 中斷向量
 * @return int 成功返回1，失敗返回0
 */
int smp_send_ipi(int cpu_index, uint8_t vector) {
    if (cpu_index < 0 || cpu_index >= cpu_count || !cpu_infos[cpu_index].present) {
        return 0;
    }
    
    apic_send_ipi(cpu_infos[cpu_index].apic_id, APIC_ICR_DELIVERY_FIXED, vector);
    return 1;
}

/**
 * @brief 廣播處理器間中斷到所有CPU
 * 
 * @param vector 中斷向量
 * @param exclude_self 是否排除自己
 */
void smp_broadcast_ipi(uint8_t vector, int exclude_self) {
    apic_broadcast_ipi(APIC_ICR_DELIVERY_FIXED, vector, exclude_self);
}

/**
 * @brief AP處理器準備就緒時調用
 * 在AP的啟動代碼中調用此函數通知BSP
 */
void smp_ap_ready() {
    // 增加準備好的AP計數
    __sync_fetch_and_add(&ap_ready_count, 1);
    
    // 執行AP特定的初始化
    // ...
    
    // AP進入空閒循環，等待進一步指令
    while (1) {
        __asm__ volatile("hlt");
    }
}
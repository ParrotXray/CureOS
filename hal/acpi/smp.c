#include <hal/acpi/acpi.h>
#include <hal/apic/apic.h>
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

// 啟動代碼模板 (更加簡化明確的版本)
static uint8_t ap_boot_code[] = {
    0xFA,               // cli - 禁用中斷
    
    // 設置段寄存器為相同的值
    0xB8, 0x00, 0x00,   // mov ax, 0  
    0x8E, 0xD8,         // mov ds, ax
    0x8E, 0xC0,         // mov es, ax
    0x8E, 0xD0,         // mov ss, ax
    
    // 設置堆疊指針
    0xBC, 0x00, 0x80, 0x00, 0x00,  // mov esp, 0x8000
    
    // 直接跳到AP入口點
    0xB8, 0x00, 0x00, 0x00, 0x00,  // mov eax, 0x00000000 (將被替換為ap_entry的物理地址)
    0xFF, 0xE0,                    // jmp eax
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
    
    // 確保AP啟動代碼的物理地址被正確映射
    // 首先確保0x8000位址可訪問
    for (uintptr_t addr = 0x8000; addr < 0x9000; addr += 0x1000) {
        vmm_map_page((void*)addr, (void*)addr, PG_PREM_RW, PG_PREM_RW);
    }
    
    // 準備AP啟動代碼
    printf("[SMP] Copying boot code to 0x8000\n");
    memcpy((void*)AP_BOOT_ADDR, ap_boot_code, sizeof(ap_boot_code));
    
    // 填寫跳轉地址 (ap_entry的物理地址)
    uintptr_t ap_entry_phys = V2P(ap_entry);
    printf("[SMP] AP entry point: virt=0x%x, phys=0x%x\n", ap_entry, ap_entry_phys);
    *((uint32_t*)(AP_BOOT_ADDR + 19)) = ap_entry_phys;
    
    // 重置計數器
    ap_ready_count = 0;
    
    // 啟動每個AP
    for (int i = 0; i < cpu_count; i++) {
        if (!cpu_infos[i].present || cpu_infos[i].is_bsp) continue;
        
        printf("[SMP] Starting CPU %d (APIC ID: %d)...\n", i, cpu_infos[i].apic_id);
        
        // 發送INIT IPI
        apic_send_ipi(cpu_infos[i].apic_id, APIC_ICR_DELIVERY_INIT, 0);
        
        // 必須延遲至少10毫秒 (根據Intel手冊)
        for (volatile int j = 0; j < 20000000; j++);
        
        // 發送STARTUP IPI (兩次，根據Intel手冊建議)
        apic_send_ipi(cpu_infos[i].apic_id, APIC_ICR_DELIVERY_STARTUP, AP_BOOT_ADDR >> 12);
        
        // 延遲200微秒
        for (volatile int j = 0; j < 2000000; j++);
        
        // 第二次STARTUP IPI
        apic_send_ipi(cpu_infos[i].apic_id, APIC_ICR_DELIVERY_STARTUP, AP_BOOT_ADDR >> 12);
        
        // 等待AP準備好
        printf("[SMP] Waiting for CPU %d to respond...\n", i);
        volatile int timeout = 10000000;  // 更合理的超時值
        int dots_printed = 0;
        
        while (ap_ready_count <= i - 1 && timeout-- > 0) {
            __asm__ volatile("pause");
            
            // 每隔一段時間顯示進度點，而不是打印大量日誌
            if (timeout % 1000000 == 0) {
                printf(".");
                dots_printed++;
                if (dots_printed >= 50) {
                    printf("\n");
                    dots_printed = 0;
                }
            }
        }
        
        if (dots_printed > 0) {
            printf("\n");
        }
        
        if (timeout <= 0) {
            printf("[SMP] Timeout waiting for CPU %d\n", i);
            
            // 在超時情況下，繼續執行而不是無限等待
            // 這樣在單CPU系統上或有問題的系統上仍能繼續啟動
            continue;
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
    // 獲取當前CPU的APIC ID
    uint8_t apic_id = apic_get_id() >> 24;
    
    printf("[SMP] CPU with APIC ID %d is ready\n", apic_id);
    
    // 增加準備好的AP計數
    __sync_fetch_and_add(&ap_ready_count, 1);
    
    // AP進入空閒循環，等待進一步指令
    while (1) {
        __asm__ volatile("hlt");
    }
}
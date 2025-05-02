#ifndef _SYSTEM_ACPI_H
#define _SYSTEM_ACPI_H

#include <stdint.h>
#include <stddef.h>

// ACPI 標準規定的簽名
#define ACPI_RSDP_SIGNATURE "RSD PTR "
#define ACPI_RSDT_SIGNATURE "RSDT"
#define ACPI_XSDT_SIGNATURE "XSDT"
#define ACPI_FADT_SIGNATURE "FACP"
#define ACPI_MADT_SIGNATURE "APIC"
#define ACPI_HPET_SIGNATURE "HPET"

// ACPI 表頭結構
typedef struct {
    char signature[4];        // 4字節簽名
    uint32_t length;          // 表長度
    uint8_t revision;         // 修訂版本
    uint8_t checksum;         // 校驗和
    char oem_id[6];           // OEM ID
    char oem_table_id[8];     // OEM表ID
    uint32_t oem_revision;    // OEM修訂版本
    uint32_t creator_id;      // 創建者ID
    uint32_t creator_revision; // 創建者修訂版本
} __attribute__((packed)) acpi_header_t;

// RSDP (Root System Description Pointer)結構
typedef struct {
    char signature[8];        // "RSD PTR "
    uint8_t checksum;         // 校驗和
    char oem_id[6];           // OEM ID
    uint8_t revision;         // 版本 (0 for ACPI 1.0, 2 for ACPI 2.0+)
    uint32_t rsdt_address;    // RSDT物理地址
    
    // 以下是ACPI 2.0新增字段
    uint32_t length;          // RSDP長度
    uint64_t xsdt_address;    // XSDT物理地址
    uint8_t extended_checksum; // 擴展校驗和
    uint8_t reserved[3];      // 保留
} __attribute__((packed)) acpi_rsdp_t;

// RSDT (Root System Description Table)結構
typedef struct {
    acpi_header_t header;     // 標準ACPI表頭
    uint32_t entries[];       // 其他ACPI表的物理地址數組
} __attribute__((packed)) acpi_rsdt_t;

// XSDT (eXtended System Description Table)結構
typedef struct {
    acpi_header_t header;     // 標準ACPI表頭
    uint64_t entries[];       // 其他ACPI表的物理地址數組(64位)
} __attribute__((packed)) acpi_xsdt_t;

// FADT (Fixed ACPI Description Table)結構
typedef struct {
    // ACPI 1.0 欄位
    acpi_header_t header;     // 標準ACPI表頭
    uint32_t firmware_ctrl;   // FACS物理地址
    uint32_t dsdt;            // DSDT物理地址
    uint8_t reserved;         // 保留
    uint8_t preferred_pm_profile; // 首選電源管理配置文件
    uint16_t sci_int;         // SCI中斷號
    uint32_t smi_cmd;         // SMI命令端口
    uint8_t acpi_enable;      // ACPI啟用值
    uint8_t acpi_disable;     // ACPI禁用值
    uint8_t s4bios_req;       // S4BIOS請求
    uint8_t pstate_cnt;       // P狀態控制
    uint32_t pm1a_evt_blk;    // PM1a事件塊地址
    uint32_t pm1b_evt_blk;    // PM1b事件塊地址
    uint32_t pm1a_cnt_blk;    // PM1a控制塊地址
    uint32_t pm1b_cnt_blk;    // PM1b控制塊地址
    uint32_t pm2_cnt_blk;     // PM2控制塊地址
    uint32_t pm_tmr_blk;      // PM計時器塊地址
    uint32_t gpe0_blk;        // GPE0塊地址
    uint32_t gpe1_blk;        // GPE1塊地址
    uint8_t pm1_evt_len;      // PM1事件塊長度
    uint8_t pm1_cnt_len;      // PM1控制塊長度
    uint8_t pm2_cnt_len;      // PM2控制塊長度
    uint8_t pm_tmr_len;       // PM計時器塊長度
    uint8_t gpe0_blk_len;     // GPE0塊長度
    uint8_t gpe1_blk_len;     // GPE1塊長度
    uint8_t gpe1_base;        // GPE1基數
    uint8_t cst_cnt;          // _CST支持
    uint16_t p_lvl2_lat;      // P狀態2延遲
    uint16_t p_lvl3_lat;      // P狀態3延遲
    uint16_t flush_size;      // 高速緩存刷新大小
    uint16_t flush_stride;    // 高速緩存刷新步長
    uint8_t duty_offset;      // 占空比偏移
    uint8_t duty_width;       // 占空比寬度
    uint8_t day_alrm;         // RTC日期警報
    uint8_t mon_alrm;         // RTC月份警報
    uint8_t century;          // RTC世紀記錄
    uint16_t iapc_boot_arch;  // IA-PC啟動架構標誌
    uint8_t reserved2;        // 保留
    uint32_t flags;           // 功能標誌

    // ACPI 2.0+ 新增欄位
    struct {
        uint8_t space_id;      // 地址空間ID
        uint8_t bit_width;     // 寄存器位寬
        uint8_t bit_offset;    // 寄存器位偏移
        uint8_t access_size;   // 訪問大小
        uint64_t address;      // 64位地址
    } __attribute__((packed)) reset_reg; // 重置寄存器

    uint8_t reset_value;       // 重置值
    uint8_t reserved3[3];      // 保留
    
    // ACPI 2.0+ 新增 - 64位地址
    uint64_t x_firmware_ctrl;  // 64位FACS地址
    uint64_t x_dsdt;           // 64位DSDT地址
    
    // ACPI 2.0+ 新增 - 擴展地址
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_pm1a_evt_blk; // PM1a事件塊擴展地址
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_pm1b_evt_blk; // PM1b事件塊擴展地址
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_pm1a_cnt_blk; // PM1a控制塊擴展地址
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_pm1b_cnt_blk; // PM1b控制塊擴展地址
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_pm2_cnt_blk; // PM2控制塊擴展地址
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_pm_tmr_blk; // PM計時器塊擴展地址
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_gpe0_blk; // GPE0塊擴展地址
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) x_gpe1_blk; // GPE1塊擴展地址
    
    // ACPI 4.0+ 新增欄位
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) sleep_control_reg; // 睡眠控制寄存器
    
    struct {
        uint8_t space_id;
        uint8_t bit_width;
        uint8_t bit_offset;
        uint8_t access_size;
        uint64_t address;
    } __attribute__((packed)) sleep_status_reg; // 睡眠狀態寄存器
    
    // ACPI 5.0+ 新增欄位
    uint64_t hypervisor_id;    // 管理程序供應商ID
} __attribute__((packed)) acpi_fadt_t;

// MADT (Multiple APIC Description Table)結構
typedef struct {
    acpi_header_t header;     // 標準ACPI表頭
    uint32_t local_apic_addr; // 本地APIC基地址
    uint32_t flags;           // 標誌位 (bit 0: PC-AT兼容標記)
    // 後面跟隨多個APIC記錄 (動態數組)
} __attribute__((packed)) acpi_madt_t;

// HPET (High Precision Event Timer)表結構
typedef struct {
    acpi_header_t header;      // 標準ACPI表頭
    uint8_t hardware_rev_id;   // 硬件版本ID
    uint8_t comparator_count:5;// 比較器數量
    uint8_t counter_size:1;    // 計數器大小(1=64位)
    uint8_t reserved:1;        // 保留
    uint8_t legacy_replacement:1; // 傳統替換IRQ映射支持
    uint16_t pci_vendor_id;    // PCI供應商ID
    uint8_t address_space_id;  // 地址空間ID
    uint8_t register_bit_width;// 寄存器位寬
    uint8_t register_bit_offset;// 寄存器位偏移
    uint8_t reserved2;         // 保留
    uint64_t address;          // HPET基地址
    uint8_t hpet_number;       // HPET號碼
    uint16_t minimum_tick;     // 最小時鐘周期
    uint8_t page_protection;   // 頁保護
} __attribute__((packed)) acpi_hpet_t;

// SRAT (System Resource Affinity Table)表結構
typedef struct {
    acpi_header_t header;      // 標準ACPI表頭
    uint32_t table_revision;   // 表格版本
    uint64_t reserved;         // 保留
    // 後面跟隨多個關聯性結構
} __attribute__((packed)) acpi_srat_t;

// SLIT (System Locality Information Table)表結構
typedef struct {
    acpi_header_t header;      // 標準ACPI表頭
    uint64_t localities;       // 指示系統中局部性數量
    // 後面跟隨多個局部性條目
} __attribute__((packed)) acpi_slit_t;

// MCFG (Memory Mapped Configuration Space)表結構
typedef struct {
    acpi_header_t header;      // 標準ACPI表頭
    uint64_t reserved;         // 保留
    // 後面跟隨多個配置空間資源描述符
} __attribute__((packed)) acpi_mcfg_t;

// MADT記錄類型
#define ACPI_MADT_TYPE_LOCAL_APIC       0
#define ACPI_MADT_TYPE_IO_APIC          1
#define ACPI_MADT_TYPE_INT_SRC_OVERRIDE 2
#define ACPI_MADT_TYPE_NMI_SRC          3
#define ACPI_MADT_TYPE_LOCAL_APIC_NMI   4
#define ACPI_MADT_TYPE_LOCAL_X2APIC     9

// MADT記錄頭
typedef struct {
    uint8_t type;             // 記錄類型
    uint8_t length;           // 記錄長度
} __attribute__((packed)) acpi_madt_record_header_t;

// 本地APIC記錄
typedef struct {
    acpi_madt_record_header_t header;
    uint8_t acpi_processor_id; // ACPI處理器ID
    uint8_t apic_id;          // APIC ID
    uint32_t flags;           // 標誌位(bit 0: 處理器啟用, bit 1: 在線能力)
} __attribute__((packed)) acpi_madt_local_apic_t;

// IO APIC記錄
typedef struct {
    acpi_madt_record_header_t header;
    uint8_t io_apic_id;       // IO APIC ID
    uint8_t reserved;         // 保留
    uint32_t io_apic_addr;    // IO APIC地址
    uint32_t global_system_interrupt_base; // 全局系統中斷基址
} __attribute__((packed)) acpi_madt_io_apic_t;

// 中斷源覆蓋記錄
typedef struct {
    acpi_madt_record_header_t header;
    uint8_t bus;              // 匯流排(通常是ISA=0)
    uint8_t source;           // IRQ來源(通常是ISA IRQ)
    uint32_t global_system_interrupt; // 全局系統中斷
    uint16_t flags;           // 標誌位
} __attribute__((packed)) acpi_madt_int_src_override_t;

/**
 * @brief 初始化ACPI子系統
 * 
 * @return int 成功返回1，失敗返回0
 */
int acpi_init();

/**
 * @brief 查找指定簽名的ACPI表
 * 
 * @param signature 表簽名(4字節)
 * @return void* 表的虛擬地址，失敗則NULL
 */
void* acpi_find_table(const char* signature);

/**
 * @brief 獲取本地APIC地址
 * 
 * @return uintptr_t 本地APIC基地址
 */
uintptr_t acpi_get_local_apic_addr();

/**
 * @brief 獲取IO APIC信息
 * 
 * @param io_apic_id 指向存儲IO APIC ID的變量
 * @param io_apic_addr 指向存儲IO APIC地址的變量
 * @return int 成功返回1，失敗返回0
 */
int acpi_get_io_apic_info(uint8_t* io_apic_id, uintptr_t* io_apic_addr);

/**
 * @brief 檢查系統是否支持ACPI
 * 
 * @return int 支持返回1，不支持返回0
 */
int acpi_is_supported();

/**
 * @brief 獲取CPU核心數量
 * 
 * @return int CPU核心數量
 */
int acpi_get_cpu_count();

/**
 * @brief 獲取指定索引的CPU的APIC ID
 * 
 * @param index CPU索引
 * @return int APIC ID，錯誤則-1
 */
int acpi_get_cpu_apic_id(int index);

#endif /* _SYSTEM_ACPI_H */
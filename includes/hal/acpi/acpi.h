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

// FADT (Fixed ACPI Description Table)結構(簡化版)
typedef struct {
    acpi_header_t header;     // 標準ACPI表頭
    uint32_t firmware_ctrl;   // FACS物理地址
    uint32_t dsdt;            // DSDT物理地址
    // ...其他FADT字段(省略)
} __attribute__((packed)) acpi_fadt_t;

// MADT (Multiple APIC Description Table)結構
typedef struct {
    acpi_header_t header;     // 標準ACPI表頭
    uint32_t local_apic_addr; // 本地APIC基地址
    uint32_t flags;           // 標誌位
    // 後面跟隨多個APIC記錄
} __attribute__((packed)) acpi_madt_t;

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
#include <hal/acpi/acpi.h>
#include <system/mm/vmm.h>
#include <system/mm/page.h>
#include <libc/string.h>
#include <libc/stdio.h>

// 全局變量存儲ACPI相關信息
static acpi_rsdp_t* rsdp = NULL;
static acpi_rsdt_t* rsdt = NULL;
static acpi_xsdt_t* xsdt = NULL;
static acpi_madt_t* madt = NULL;
static uintptr_t local_apic_addr = 0;
static uint8_t io_apic_id = 0;
static uintptr_t io_apic_addr = 0;

// CPU信息存儲
#define MAX_CPU_COUNT 16
static int cpu_count = 0;
static uint8_t cpu_apic_ids[MAX_CPU_COUNT];

// 計算校驗和
static uint8_t acpi_checksum(void* ptr, size_t size) {
    uint8_t sum = 0;
    uint8_t* p = (uint8_t*)ptr;
    
    for (size_t i = 0; i < size; i++) {
        sum += p[i];
    }
    
    return sum;
}

// 安全比較兩個字符串
static int safe_memcmp(const void* s1, const void* s2, size_t n) {
    const unsigned char* p1 = s1;
    const unsigned char* p2 = s2;
    
    for (size_t i = 0; i < n; i++) {
        if (p1[i] != p2[i])
            return p1[i] - p2[i];
    }
    
    return 0;
}

static acpi_rsdp_t* acpi_find_rsdp() {
    printf("[ACPI] Starting BIOS area search for RSDP\n");

    const char signature[9] = "RSD PTR ";

    uintptr_t addr = 0xE0000;
    for (; addr < 0x100000; addr += 16) {
        uint8_t* ptr = (uint8_t*)addr; // <<< 不轉P2V

        if (ptr[0] != 'R') {
            continue;
        }

        if (memcmp(ptr, signature, 8) == 0) {
            acpi_rsdp_t* potential_rsdp = (acpi_rsdp_t*)ptr;

            uint8_t sum = 0;
            for (int i = 0; i < 20; i++) {
                sum += ((uint8_t*)potential_rsdp)[i];
            }

            if (sum == 0) {
                printf("[ACPI] Valid RSDP found at physical address 0x%x\n", addr);
                return potential_rsdp;
            } else {
                printf("[ACPI] Invalid checksum for RSDP at 0x%x\n", addr);
            }
        }
    }

    printf("[ACPI] No valid RSDP found in BIOS area\n");
    return NULL;
}


// 解析MADT表，獲取APIC信息
static void acpi_parse_madt() {
    if (!madt) {
        printf("[ACPI] MADT not available for parsing\n");
        return;
    }

    printf("[ACPI] Parsing MADT table\n");

    // 取得本地 APIC 地址
    local_apic_addr = madt->local_apic_addr;
    printf("[ACPI] Local APIC address: 0x%x\n", local_apic_addr);

    uintptr_t current = (uintptr_t)madt + sizeof(acpi_madt_t);
    uintptr_t end = (uintptr_t)madt + madt->header.length;

    cpu_count = 0; // 重置CPU數量計數

    printf("[ACPI] --- MADT Records Dump Start ---\n");

    while (current < end && current >= (uintptr_t)madt) {
        acpi_madt_record_header_t* record = (acpi_madt_record_header_t*)current;

        if (!record || record->length < 2 || (current + record->length > end)) {
            printf("[ACPI] Invalid MADT record at 0x%x, stopping parsing\n", current);
            break;
        }

        printf("[ACPI] Record Type=%d, Length=%d at 0x%x\n",
               record->type, record->length, current);

        switch (record->type) {
            case ACPI_MADT_TYPE_LOCAL_APIC: {
                if (record->length >= sizeof(acpi_madt_local_apic_t)) {
                    acpi_madt_local_apic_t* local_apic = (acpi_madt_local_apic_t*)record;
                    printf("[ACPI]   Local APIC: Processor ID=%d, APIC ID=%d, Flags=0x%x\n",
                           local_apic->acpi_processor_id,
                           local_apic->apic_id,
                           local_apic->flags);

                    if (local_apic->flags & 1) { // Processor is enabled
                        if (cpu_count < MAX_CPU_COUNT) {
                            cpu_apic_ids[cpu_count++] = local_apic->apic_id;
                            printf("[ACPI]   -> Registered CPU #%d with APIC ID=%d\n",
                                   cpu_count - 1, local_apic->apic_id);
                        }
                    }
                }
                break;
            }
            case ACPI_MADT_TYPE_IO_APIC: {
                if (record->length >= sizeof(acpi_madt_io_apic_t)) {
                    acpi_madt_io_apic_t* io_apic = (acpi_madt_io_apic_t*)record;
                    printf("[ACPI]   IO APIC: ID=%d, Address=0x%x, GSIB=%d\n",
                           io_apic->io_apic_id,
                           io_apic->io_apic_addr,
                           io_apic->global_system_interrupt_base);

                    io_apic_id = io_apic->io_apic_id;
                    io_apic_addr = io_apic->io_apic_addr;
                }
                break;
            }
            case ACPI_MADT_TYPE_INT_SRC_OVERRIDE: {
                if (record->length >= sizeof(acpi_madt_int_src_override_t)) {
                    acpi_madt_int_src_override_t* override = (acpi_madt_int_src_override_t*)record;
                    printf("[ACPI]   Interrupt Override: Bus=%d, Source=%d, GSI=%d, Flags=0x%x\n",
                           override->bus,
                           override->source,
                           override->global_system_interrupt,
                           override->flags);
                }
                break;
            }
            case ACPI_MADT_TYPE_NMI_SRC: {
                printf("[ACPI]   NMI Source: (Details skipped)\n");
                break;
            }
            case ACPI_MADT_TYPE_LOCAL_APIC_NMI: {
                printf("[ACPI]   Local APIC NMI: (Details skipped)\n");
                break;
            }
            default: {
                printf("[ACPI]   Unknown MADT record type %d (Details skipped)\n",
                       record->type);
                break;
            }
        }

        current += record->length;
    }

    printf("[ACPI] --- MADT Records Dump End ---\n");

    // 最後確認是否找到IOAPIC
    if (io_apic_addr == 0) {
        printf("[ACPI] No IO APIC found in MADT, will fallback to default IO APIC address\n");
        io_apic_addr = 0xFEC00000;
        io_apic_id = 0;
    }
}

// 查找特定簽名的表
static void* acpi_find_table_in_rsdt(const char* signature) {
    if (!rsdt) {
        printf("[ACPI] RSDT not available\n");
        return NULL;
    }
    
    // 安全檢查RSDT頭部長度
    if (rsdt->header.length < sizeof(acpi_header_t)) {
        printf("[ACPI] Invalid RSDT header length\n");
        return NULL;
    }
    
    // 計算條目數量
    uint32_t entries = (rsdt->header.length - sizeof(acpi_header_t)) / sizeof(uint32_t);
    printf("[ACPI] RSDT contains %d entries\n", entries);
    
    // 確保條目數量合理
    if (entries > 100) { // 設置一個上限以防止無效數據
        printf("[ACPI] Too many RSDT entries, limiting search\n");
        entries = 100;
    }
    
    for (uint32_t i = 0; i < entries; i++) {
        if (!rsdt->entries[i]) {
            continue;
        }
        
        // 嘗試訪問表頭
        acpi_header_t* header = (acpi_header_t*)P2V(rsdt->entries[i]);
        if (!header) {
            printf("[ACPI] Could not map RSDT entry %d\n", i);
            continue;
        }
        
        // 檢查簽名
        if (safe_memcmp(header->signature, signature, 4) == 0) {
            // 驗證表校驗和
            if (acpi_checksum(header, header->length) == 0) {
                printf("[ACPI] Found valid table '%c%c%c%c' at 0x%x\n", 
                       signature[0], signature[1], signature[2], signature[3], 
                       rsdt->entries[i]);
                return header;
            } else {
                printf("[ACPI] Found table '%c%c%c%c' but checksum invalid\n", 
                       signature[0], signature[1], signature[2], signature[3]);
            }
        }
    }
    
    printf("[ACPI] Table '%c%c%c%c' not found in RSDT\n", 
           signature[0], signature[1], signature[2], signature[3]);
    return NULL;
}

static void* acpi_find_table_in_xsdt(const char* signature) {
    if (!xsdt) {
        printf("[ACPI] XSDT not available\n");
        return NULL;
    }
    
    // 安全檢查XSDT頭部長度
    if (xsdt->header.length < sizeof(acpi_header_t)) {
        printf("[ACPI] Invalid XSDT header length\n");
        return NULL;
    }
    
    // 計算條目數量
    uint32_t entries = (xsdt->header.length - sizeof(acpi_header_t)) / sizeof(uint64_t);
    printf("[ACPI] XSDT contains %d entries\n", entries);
    
    // 確保條目數量合理
    if (entries > 100) { // 設置一個上限以防止無效數據
        printf("[ACPI] Too many XSDT entries, limiting search\n");
        entries = 100;
    }
    
    for (uint32_t i = 0; i < entries; i++) {
        if (!xsdt->entries[i]) {
            continue;
        }
        
        // 嘗試訪問表頭
        acpi_header_t* header = (acpi_header_t*)P2V(xsdt->entries[i]);
        if (!header) {
            printf("[ACPI] Could not map XSDT entry %d\n", i);
            continue;
        }
        
        // 檢查簽名
        if (safe_memcmp(header->signature, signature, 4) == 0) {
            // 驗證表校驗和
            if (acpi_checksum(header, header->length) == 0) {
                printf("[ACPI] Found valid table '%c%c%c%c' at 0x%llx\n", 
                       signature[0], signature[1], signature[2], signature[3], 
                       xsdt->entries[i]);
                return header;
            } else {
                printf("[ACPI] Found table '%c%c%c%c' but checksum invalid\n", 
                       signature[0], signature[1], signature[2], signature[3]);
            }
        }
    }
    
    printf("[ACPI] Table '%c%c%c%c' not found in XSDT\n", 
           signature[0], signature[1], signature[2], signature[3]);
    return NULL;
}

int acpi_init() {
    printf("[ACPI] Initializing ACPI subsystem\n");
    
    // 查找RSDP (重要的第一步，這裡是關鍵)
    printf("[ACPI] Searching for RSDP...\n");
    
    // 使用简化的RSDP查找函数，同时设置默认值
    local_apic_addr = 0xFEE00000;  // 默认值
    io_apic_addr = 0xFEC00000;     // 默认值
    io_apic_id = 0;                // 默认值
    cpu_count = 1;                 // 默认至少有一个处理器
    cpu_apic_ids[0] = 0;           // 默认BSP的APIC ID为0
    
    // 尝试查找RSDP，如果失败仍然能使用默认值继续
    rsdp = acpi_find_rsdp();
    
    if (!rsdp) {
        printf("[ACPI] RSDP not found, using default APIC values\n");
        printf("[ACPI] Using default Local APIC address: 0x%x\n", local_apic_addr);
        printf("[ACPI] Using default IO APIC address: 0x%x\n", io_apic_addr);
        printf("[ACPI] ACPI initialization complete with default values\n");
        return 1;  // 返回成功，使用默认值
    }
    
    printf("[ACPI] Found RSDP at %p, revision %d\n", rsdp, rsdp->revision);
    
    // 根據ACPI版本選擇RSDT或XSDT
    if (rsdp->revision >= 2 && rsdp->xsdt_address) {
        printf("[ACPI] Using XSDT at physical address 0x%llx\n", rsdp->xsdt_address);
        printf("[ACPI] Mapping XSDT memory...\n");
        // xsdt = (acpi_xsdt_t*)P2V(rsdp->xsdt_address);

        uintptr_t xsdt_phys = (uintptr_t)rsdp->xsdt_address;

        // 映射 XSDT 表
        for (uintptr_t offset = 0; offset < 0x1000; offset += 0x1000) {
            vmm_map_page((void*)(P2V(xsdt_phys + offset)), 
                         xsdt_phys + offset, 
                         PG_PREM_RW, PG_PREM_RW);
        }
        
        // 轉虛擬地址
        xsdt = (acpi_xsdt_t*)P2V(xsdt_phys);
        
        if (acpi_checksum(xsdt, xsdt->header.length) != 0) {
            printf("[ACPI] XSDT checksum invalid, trying RSDT\n");
            xsdt = NULL;
        }
        
        if (!xsdt) {
            printf("[ACPI] Failed to map XSDT, trying RSDT\n");
        } else if (acpi_checksum(xsdt, xsdt->header.length) != 0) {
            printf("[ACPI] XSDT checksum invalid, trying RSDT\n");
            xsdt = NULL;
        }
    }
    
    if (!xsdt && rsdp->rsdt_address) {
        printf("[ACPI] Using RSDT at physical address 0x%x\n", rsdp->rsdt_address);
        printf("[ACPI] Mapping RSDT memory...\n");

        // 預先映射 RSDT 完整表格
        uintptr_t rsdt_phys = rsdp->rsdt_address;
        acpi_rsdt_t* rsdt_tmp = (acpi_rsdt_t*)(uintptr_t)rsdt_phys;
        
        // 注意：rsdt_tmp 這時候直接是物理地址指標，不能直接解dereference
        // 所以我們假設RSDT表長度不會超過一兩個頁面，保守做法：
        
        for (uintptr_t offset = 0; offset < 0x1000; offset += 0x1000) {
            vmm_map_page((void*)(P2V(rsdt_phys + offset)),
                         rsdt_phys + offset,
                         PG_PREM_RW, PG_PREM_RW);
        }
        
        // 再去用P2V轉虛擬位址來讀
        rsdt = (acpi_rsdt_t*)P2V(rsdt_phys);
        
        if (acpi_checksum(rsdt, rsdt->header.length) != 0) {
            printf("[ACPI] RSDT checksum invalid, using default values\n");
            rsdt = NULL;
            return 1;
        }
        
        if (!rsdt) {
            printf("[ACPI] Failed to map RSDT, using default values\n");
            return 1;  // 使用默认值
        }
        
        if (acpi_checksum(rsdt, rsdt->header.length) != 0) {
            printf("[ACPI] RSDT checksum invalid, using default values\n");
            rsdt = NULL;
            return 1;  // 使用默认值
        }
    }
    
    if (!xsdt && !rsdt) {
        printf("[ACPI] No valid system description table found, using default values\n");
        return 1;  // 使用默认值
    }
    
    // 查找MADT
    madt = (acpi_madt_t*)acpi_find_table(ACPI_MADT_SIGNATURE);
    
    if (madt) {
        printf("[ACPI] Found MADT, parsing APIC information\n");
        acpi_parse_madt();
    } else {
        printf("[ACPI] MADT not found, using default APIC values\n");
        // 已经设置了默认值，不需要重复设置
    }
    
    printf("[ACPI] ACPI initialization complete\n");
    return 1;
}

void* acpi_find_table(const char* signature) {
    printf("[ACPI] Looking for table '%c%c%c%c'\n", 
           signature[0], signature[1], signature[2], signature[3]);
    
    // 優先使用XSDT，如果有的話
    if (xsdt) {
        void* table = acpi_find_table_in_xsdt(signature);
        if (table) return table;
    }
    
    // 否則使用RSDT
    if (rsdt) {
        return acpi_find_table_in_rsdt(signature);
    }
    
    return NULL;
}

uintptr_t acpi_get_local_apic_addr() {
    return local_apic_addr;
}

int acpi_get_io_apic_info(uint8_t* out_io_apic_id, uintptr_t* out_io_apic_addr) {
    if (out_io_apic_id) *out_io_apic_id = io_apic_id;
    if (out_io_apic_addr) *out_io_apic_addr = io_apic_addr;
    
    return 1;  // 始终返回成功，因为我们有默认值
}

int acpi_is_supported() {
    return 1;  // 始终返回支持，因为我们有默认值
}

int acpi_get_cpu_count() {
    return cpu_count > 0 ? cpu_count : 1; // 至少返回1，表示BSP
}

int acpi_get_cpu_apic_id(int index) {
    if (index < 0 || index >= cpu_count || cpu_count == 0) {
        return -1;
    }
    
    return cpu_apic_ids[index];
}
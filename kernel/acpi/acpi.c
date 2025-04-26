#include <system/acpi/acpi.h>
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

// 查找RSDP - 完全重寫版本，更加謹慎和安全
static acpi_rsdp_t* acpi_find_rsdp() {
    printf("[ACPI] Starting search for RSDP\n");
    
    // 首先使用靜態定義的簽名進行比較
    const char signature[9] = "RSD PTR ";
    
    // 首先嘗試從BIOS區域 0xE0000 - 0xFFFFF 尋找，這是更可靠的
    printf("[ACPI] Searching in BIOS area\n");
    
    // 確保這塊記憶體已經被映射，如果沒有則進行映射
    // 注意：這些地址應該已經被映射在內核的初始化階段
    
    for (uintptr_t addr = 0xE0000; addr < 0x100000; addr += 16) {
        // 映射地址到虛擬內存
        void* vaddr = (void*)P2V(addr);
        
        // 嘗試比較簽名
        if (safe_memcmp(vaddr, signature, 8) == 0) {
            printf("[ACPI] Potential RSDP found at physical address 0x%x\n", addr);
            
            // 額外驗證校驗和
            acpi_rsdp_t* potential_rsdp = (acpi_rsdp_t*)vaddr;
            
            // 計算並檢查校驗和 (ACPI 1.0 部分)
            uint8_t sum = 0;
            for (int i = 0; i < 20; i++) {
                sum += ((uint8_t*)potential_rsdp)[i];
            }
            
            if (sum == 0) {
                printf("[ACPI] RSDP checksum valid\n");
                return potential_rsdp;
            } else {
                printf("[ACPI] RSDP found but checksum invalid (got %d)\n", sum);
            }
        }
    }
    
    // 如果在BIOS區域沒找到，再嘗試EBDA
    // 注意：訪問EBDA可能不那麼可靠，要更加小心
    printf("[ACPI] RSDP not found in BIOS area, checking EBDA\n");
    
    // 獲取EBDA地址，通常位於物理地址0x40E
    // 但要小心：有些系統可能沒有EBDA或地址無效
    uint16_t* ebda_ptr_addr = (uint16_t*)P2V(0x40E);
    
    // 確保地址可訪問
    if (ebda_ptr_addr) {
        uint16_t ebda_segment = *ebda_ptr_addr;
        uintptr_t ebda_addr = ebda_segment << 4; // 轉換段地址為物理地址
        
        printf("[ACPI] EBDA address: 0x%x\n", ebda_addr);
        
        // 驗證EBDA地址是否在合理範圍內
        if (ebda_addr && ebda_addr < 0x100000 && ebda_addr >= 0x80000) {
            // 在EBDA的前1KB搜索
            for (uintptr_t addr = ebda_addr; addr < ebda_addr + 1024; addr += 16) {
                // 映射地址到虛擬內存
                void* vaddr = (void*)P2V(addr);
                
                // 嘗試比較簽名
                if (safe_memcmp(vaddr, signature, 8) == 0) {
                    printf("[ACPI] Potential RSDP found in EBDA at physical address 0x%x\n", addr);
                    
                    // 額外驗證校驗和
                    acpi_rsdp_t* potential_rsdp = (acpi_rsdp_t*)vaddr;
                    
                    // 計算並檢查校驗和 (ACPI 1.0 部分)
                    uint8_t sum = 0;
                    for (int i = 0; i < 20; i++) {
                        sum += ((uint8_t*)potential_rsdp)[i];
                    }
                    
                    if (sum == 0) {
                        printf("[ACPI] RSDP in EBDA checksum valid\n");
                        return potential_rsdp;
                    } else {
                        printf("[ACPI] RSDP found in EBDA but checksum invalid (got %d)\n", sum);
                    }
                }
            }
        }
    }
    
    printf("[ACPI] RSDP not found anywhere\n");
    return NULL;
}

// 解析MADT表，獲取APIC信息
static void acpi_parse_madt() {
    if (!madt) {
        printf("[ACPI] MADT not available for parsing\n");
        return;
    }
    
    printf("[ACPI] Parsing MADT table\n");
    
    // 獲取本地APIC地址
    local_apic_addr = madt->local_apic_addr;
    printf("[ACPI] Local APIC address: 0x%x\n", local_apic_addr);
    
    // 解析MADT記錄
    uintptr_t end = (uintptr_t)madt + madt->header.length;
    uintptr_t current = (uintptr_t)madt + sizeof(acpi_madt_t);
    
    cpu_count = 0; // 重置CPU計數
    
    while (current < end && current >= (uintptr_t)madt) { // 添加下界檢查
        acpi_madt_record_header_t* record = (acpi_madt_record_header_t*)current;
        
        // 檢查記錄長度的有效性
        if (!record || record->length < 2 || current + record->length > end) {
            printf("[ACPI] Invalid MADT record, stopping parsing\n");
            break;
        }
        
        switch (record->type) {
            case ACPI_MADT_TYPE_LOCAL_APIC: {
                if (record->length >= sizeof(acpi_madt_local_apic_t)) {
                    acpi_madt_local_apic_t* local_apic = (acpi_madt_local_apic_t*)record;
                    // 檢查處理器標誌位，確認處理器啟用
                    if (local_apic->flags & 1) {
                        if (cpu_count < MAX_CPU_COUNT) {
                            cpu_apic_ids[cpu_count++] = local_apic->apic_id;
                            printf("[ACPI] Found processor: ID=%d, APIC ID=%d\n", 
                                local_apic->acpi_processor_id, local_apic->apic_id);
                        }
                    }
                }
                break;
            }
            case ACPI_MADT_TYPE_IO_APIC: {
                if (record->length >= sizeof(acpi_madt_io_apic_t)) {
                    acpi_madt_io_apic_t* io_apic = (acpi_madt_io_apic_t*)record;
                    io_apic_id = io_apic->io_apic_id;
                    io_apic_addr = io_apic->io_apic_addr;
                    printf("[ACPI] Found IO APIC: ID=%d, Address=0x%x\n", 
                        io_apic_id, io_apic_addr);
                }
                break;
            }
            case ACPI_MADT_TYPE_INT_SRC_OVERRIDE: {
                if (record->length >= sizeof(acpi_madt_int_src_override_t)) {
                    acpi_madt_int_src_override_t* override = (acpi_madt_int_src_override_t*)record;
                    printf("[ACPI] Found Interrupt Override: Bus=%d, Source=%d, GSI=%d, Flags=0x%x\n", 
                        override->bus, override->source, override->global_system_interrupt, override->flags);
                }
                break;
            }
            // 其他記錄類型省略
        }
        
        current += record->length;
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
    rsdp = acpi_find_rsdp();
    
    if (!rsdp) {
        printf("[ACPI] RSDP not found, ACPI not supported\n");
        // 設置默認值，以便系統仍然能夠運行
        local_apic_addr = 0xFEE00000;
        io_apic_addr = 0xFEC00000;
        return 0;
    }
    
    printf("[ACPI] Found RSDP at %p, revision %d\n", rsdp, rsdp->revision);
    
    // 根據ACPI版本選擇RSDT或XSDT
    if (rsdp->revision >= 2 && rsdp->xsdt_address) {
        printf("[ACPI] Using XSDT at physical address 0x%llx\n", rsdp->xsdt_address);
        xsdt = (acpi_xsdt_t*)P2V(rsdp->xsdt_address);
        
        if (!xsdt) {
            printf("[ACPI] Failed to map XSDT, trying RSDT\n");
        } else if (acpi_checksum(xsdt, xsdt->header.length) != 0) {
            printf("[ACPI] XSDT checksum invalid, trying RSDT\n");
            xsdt = NULL;
        }
    }
    
    if (!xsdt && rsdp->rsdt_address) {
        printf("[ACPI] Using RSDT at physical address 0x%x\n", rsdp->rsdt_address);
        rsdt = (acpi_rsdt_t*)P2V(rsdp->rsdt_address);
        
        if (!rsdt) {
            printf("[ACPI] Failed to map RSDT\n");
            return 0;
        }
        
        if (acpi_checksum(rsdt, rsdt->header.length) != 0) {
            printf("[ACPI] RSDT checksum invalid\n");
            rsdt = NULL;
            return 0;
        }
    }
    
    if (!xsdt && !rsdt) {
        printf("[ACPI] No valid system description table found\n");
        return 0;
    }
    
    // 查找MADT
    madt = (acpi_madt_t*)acpi_find_table(ACPI_MADT_SIGNATURE);
    
    if (madt) {
        printf("[ACPI] Found MADT, parsing APIC information\n");
        acpi_parse_madt();
    } else {
        printf("[ACPI] MADT not found, using default APIC values\n");
        local_apic_addr = 0xFEE00000;
        io_apic_addr = 0xFEC00000;
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
    if (!io_apic_addr) {
        // 使用默認值
        if (out_io_apic_id) *out_io_apic_id = 0;
        if (out_io_apic_addr) *out_io_apic_addr = 0xFEC00000;
        return 0;
    }
    
    if (out_io_apic_id) *out_io_apic_id = io_apic_id;
    if (out_io_apic_addr) *out_io_apic_addr = io_apic_addr;
    
    return 1;
}

int acpi_is_supported() {
    return rsdp != NULL;
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
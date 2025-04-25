#include <system/mm/page.h>
#include <system/mm/vmm.h>
#include <system/mm/pmm.h>
#include <libc/string.h>

// TODO: Move these nasty inline asm stuff into hal
//      These should be arch dependent
ptd_t* get_pd() {
    ptd_t* pd;
    #ifdef __ARCH_IA32
    __asm__(
        "movl %%cr3, %0\n"
        "andl $0xfffff000, %0"
        : "=r"(pd)
    );
    #endif
    return P2V(pd);
}

void set_pd(ptd_t* pd) {
    #ifdef __ARCH_IA32
    __asm__(
        "movl %0, %%eax\n"
        "andl $0xfffff000, %%eax\n"
        "movl %%eax, %%cr3\n"
        :
        : "r" (pd)
    );
    #endif
}

void vmm_init() {
    // TODO: something here?
}

ptd_t* vmm_init_pd() {
    ptd_t* dir = pmm_alloc_page();
    for (size_t i = 0; i < 1024; i++)
    {
        dir[i] = 0;
    }
    
    // 自己映射自己，方便我们在软件层面进行查表地址转换
    dir[1023] = PDE(T_SELF_REF_PERM, dir);

    return dir;
}

void* vmm_map_page(void* va, void* pa, pt_attr dattr, pt_attr tattr) {
    // 显然，对空指针进行映射没有意义。
    if (!pa || !va) {
        return NULL;
    }

    uintptr_t pd_offset = PD_INDEX(va);
    uintptr_t pt_offset = PT_INDEX(va);
    ptd_t* ptd = (ptd_t*)PTD_BASE_VADDR;

    // 在页表与页目录中找到一个可用的空位进行映射（位于va或其附近）
    ptd_t* pde = ptd[pd_offset];
    pt_t* pt = (uintptr_t)PT_VADDR(pd_offset);
    while (pde && pd_offset < 1024) {
        if (pt_offset == 1024) {
            pd_offset++;
            pt_offset = 0;
            pde = ptd[pd_offset];
            pt = (pt_t*)PT_VADDR(pd_offset);
        }
        // 页表有空位，只需要开辟一个新的 PTE
        if (pt && !pt[pt_offset]) {
            pt[pt_offset] = PTE(tattr, pa);
            return V_ADDR(pd_offset, pt_offset, PG_OFFSET(va));
        }
        pt_offset++;
    }
    
    // 页目录与所有页表已满！
    if (pd_offset > 1024) {
        return NULL;
    }

    // 页目录有空位，需要开辟一个新的 PDE
    uint8_t* new_pt_pa = pmm_alloc_page();
    
    // 物理内存已满！
    if (!new_pt_pa) {
        return NULL;
    }
    
    ptd[pd_offset] = PDE(dattr, new_pt_pa);
    
    memset((void*)PT_VADDR(pd_offset), 0, PM_PAGE_SIZE);
    pt[pt_offset] = PTE(tattr, pa);

    return V_ADDR(pd_offset, pt_offset, PG_OFFSET(va));
}

void* vmm_alloc_page(void* vpn, pt_attr dattr, pt_attr tattr) {
    void* pp = pmm_alloc_page();
    void* result = vmm_map_page(vpn, pp, dattr, tattr);
    if (!result) pmm_free_page(pp);
    return result;
}

void vmm_unmap_page(void* vpn) {
    // 1. 防止解除映射核心關鍵區域 (0xC0000000以上的空間)
    if ((uintptr_t)vpn >= 0xC0000000) {
        printf("[VMM-UNMAP-DEBUG] Skipping kernel space page at %p\n", vpn);
        return;
    }
    
    // 2. 添加更多的保護區域檢查
    // 防止解除映射可能正在使用的I/O映射或其他特殊頁面
    // 例如VGA緩衝區，BIOS數據區等
    if ((uintptr_t)vpn < 0x1000) {  // 第一頁通常包含重要的IDT等
        printf("[VMM-UNMAP-DEBUG] Skipping critical first page at %p\n", vpn);
        return;
    }
    
    if ((uintptr_t)vpn >= 0xA0000 && (uintptr_t)vpn < 0x100000) {  // VGA和BIOS區域
        printf("[VMM-UNMAP-DEBUG] Skipping VGA/BIOS area at %p\n", vpn);
        return;
    }
    
    // 3. 獲取頁目錄和頁表項
    uintptr_t pd_offset = PD_INDEX(vpn);
    uintptr_t pt_offset = PT_INDEX(vpn);
    ptd_t* self_pde = PTD_BASE_VADDR;
    
    ptd_t pde = self_pde[pd_offset];
    
    // 4. 進行完整性和有效性檢查
    if (!pde) {
        printf("[VMM-UNMAP-DEBUG] Page directory entry not present for %p\n", vpn);
        return;  // 頁目錄項不存在，無需進一步操作
    }
    
    // 5. 獲取頁表並檢查頁表項
    pt_t* pt = (pt_t*)PT_VADDR(pd_offset);
    uint32_t pte = pt[pt_offset];
    
    if (!(pte & 1)) {  // 檢查頁面是否存在 (P位)
        printf("[VMM-UNMAP-DEBUG] Page not present at %p\n", vpn);
        return;  // 頁表項不存在或不是有效映射
    }
    
    // 6. 嘗試釋放物理頁面，但添加更多檢查
    uintptr_t physical_addr = pte & ~0xFFF;  // 獲取物理地址 (去掉標誌位)
    
    // 檢查此物理地址是否為安全可釋放區域
    // 此處可添加更多邏輯，如檢查是否為內核預留區域等
    
    // 7. 釋放頁面並刷新TLB
    if (IS_CACHED(pte)) {
        // 記錄即將釋放的頁面
        printf("[VMM-UNMAP-DEBUG] Freeing physical page %p mapped at virtual address %p\n", 
               (void*)physical_addr, vpn);
               
        if (pmm_free_page(pte)) {
            // 僅在物理頁面成功釋放後才刷新TLB
            #ifdef __ARCH_IA32
            // 注意：確保這不是包含當前執行代碼的頁面
            __asm__ volatile("invlpg (%0)" :: "r"((uintptr_t)vpn) : "memory");
            #endif
            
            // 8. 清除頁表項
            pt[pt_offset] = 0;
            printf("[VMM-UNMAP-DEBUG] Successfully unmapped page at %p\n", vpn);
        } else {
            printf("[VMM-UNMAP-ERROR] Failed to free physical page for %p\n", vpn);
        }
    } else {
        // 未緩存頁面，直接清除頁表項
        pt[pt_offset] = 0;
        printf("[VMM-UNMAP-DEBUG] Unmapped uncached page at %p\n", vpn);
    }
}

void* vmm_v2p(void* va) {
    uintptr_t pd_offset = PD_INDEX(va);
    uintptr_t pt_offset = PT_INDEX(va);
    uintptr_t po = PG_OFFSET(va);
    ptd_t* self_pde = PTD_BASE_VADDR;

    ptd_t pde = self_pde[pd_offset];
    if (pde) {
        pt_t pte = ((pt_t*)PT_VADDR(pd_offset))[pt_offset];
        if (pte) {
            uintptr_t ppn = pte >> 12;
            return (void*)P_ADDR(ppn, po);
        }
    }

    return NULL;
}
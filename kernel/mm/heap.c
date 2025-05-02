#include <system/mm/heap.h>
#include <system/mm/vmm.h>
#include <system/mm/pmm.h>
#include <system/mm/page.h>
#include <libc/string.h>
#include <libc/stdio.h>

// 全局內核堆
heap_t* kheap = NULL;

static heap_block_t* find_free_block(heap_t* heap, size_t size) {
    heap_block_t* current = heap->first_block;
    
    // 尋找符合大小的第一個空閒塊
    int block_count = 0;
    while (current) {
        if (current->is_free && current->size >= size) {
            return current;
        }
        
        current = current->next;
    }
    
    return NULL;
}

static heap_block_t* extend_heap(heap_t* heap, size_t size) {
    // 檢查是否允許擴展
    if (!(heap->flags & HEAP_AUTO_EXTEND)) {
        return NULL;
    }
    
    // 計算需要擴展的大小（對齊到頁大小）
    size_t block_size = size + sizeof(heap_block_t);
    size_t pages_needed = (block_size + 0xFFF) >> 12; // 向上取整到頁大小
    
    // 檢查是否超過最大地址
    if (heap->end_addr + (pages_needed << 12) > heap->max_addr) {
        return NULL;
    }
    
    // 分配新的頁面
    uintptr_t old_end = heap->end_addr;
    
    for (size_t i = 0; i < pages_needed; i++) {
        void* vaddr = (void*)(old_end + (i << 12));
        
        if (!vmm_alloc_page(vaddr, PG_PREM_RW, PG_PREM_RW)) {
            // 分配失敗，回滾
            for (size_t j = 0; j < i; j++) {
                vmm_unmap_page((void*)(old_end + (j << 12)));
            }
            return NULL;
        }
    }
    
    // 更新堆結束地址
    heap->end_addr += pages_needed << 12;
    
    // 創建新塊
    heap_block_t* new_block = (heap_block_t*)old_end;
    
    new_block->size = (pages_needed << 12) - sizeof(heap_block_t);
    new_block->is_free = HEAP_BLOCK_FREE;
    
    // 鏈接到堆
    if (!heap->first_block) {
        heap->first_block = new_block;
        new_block->next = NULL;
    } else {
        heap_block_t* current = heap->first_block;
        int blocks_traversed = 0;
        while (current->next) {
            current = current->next;
            blocks_traversed++;
        }
        
        current->next = new_block;
        new_block->next = NULL;
    }
    
    return new_block;
}

static void split_block(heap_block_t* block, size_t size) {
    // 確保 size 已對齊到 16 位元組邊界
    size = (size + 15) & ~15;
    
    // 只有當剩餘空間足夠容納一個新塊頭加上至少 16 位元組數據時才分裂
    if (block->size > size + sizeof(heap_block_t) + 16) {
        
        // 計算新塊的位置，確保對齊到 16 位元組邊界
        uintptr_t new_block_addr = ((uintptr_t)block + sizeof(heap_block_t) + size);
        // 確保地址對齊 16 位元組
        new_block_addr = (new_block_addr + 15) & ~15;
        
        heap_block_t* new_block = (heap_block_t*)new_block_addr;
        
        // 計算實際使用的大小（可能因對齊而增加）
        size_t actual_size = new_block_addr - (uintptr_t)block - sizeof(heap_block_t);
        
        // 設置新塊屬性
        new_block->size = block->size - actual_size - sizeof(heap_block_t);
        new_block->is_free = HEAP_BLOCK_FREE;
        new_block->next = block->next;
        
        // 更新當前塊
        block->size = actual_size;
        block->next = new_block;
    }
}

static void merge_adjacent_blocks(heap_t* heap) {
    
    heap_block_t* current = heap->first_block;
    
    int merges_performed = 0;
    int blocks_traversed = 0;
    
    while (current && current->next) {
        if (current->is_free && current->next->is_free) {       
            // 合併兩個相鄰的空閒塊
            current->size += sizeof(heap_block_t) + current->next->size;
            current->next = current->next->next;
            
            merges_performed++;
        } else {
            current = current->next;
            blocks_traversed++;
        }
    }
    
}

heap_t* heap_init(uintptr_t start_addr, uintptr_t end_addr, uintptr_t max_addr) {
    
    // 確保地址對齊
    start_addr = (start_addr + 0xF) & ~0xF; // 16字節對齊
    
    // 創建堆結構
    heap_t* heap = (heap_t*)start_addr;
    start_addr += sizeof(heap_t);
    
    // 初始化堆結構
    heap->start_addr = start_addr;
    heap->end_addr = end_addr;
    heap->max_addr = max_addr;
    heap->flags = HEAP_AUTO_EXTEND;
    heap->first_block = NULL;
    
    // 初始化第一個塊
    if (start_addr < end_addr) {
        heap_block_t* first_block = (heap_block_t*)start_addr;
        
        first_block->size = end_addr - start_addr - sizeof(heap_block_t);
        
        first_block->is_free = HEAP_BLOCK_FREE;
        first_block->next = NULL;
        heap->first_block = first_block;
    } else {
        // printf("[HEAP-DEBUG] start_addr >= end_addr, skipping first block creation\n");
    }
    
    return heap;
}

void* heap_alloc(heap_t* heap, size_t size) {
    // 對齊到16字節邊界
    size = (size + 15) & ~15;
    
    if (size == 0) {
        return NULL;
    }
    
    // 尋找空閒塊
    heap_block_t* block = find_free_block(heap, size);
    
    // 如果沒有找到合適的塊，嘗試擴展堆
    if (!block) {
        block = extend_heap(heap, size);
        if (!block) {
            return NULL; // 無法分配
        }
    }
    
    // 分裂塊（如果可能）
    split_block(block, size);
    
    // 標記為已使用
    block->is_free = HEAP_BLOCK_USED;
    
    // 返回數據區域指針
    void* result = (void*)((uintptr_t)block + sizeof(heap_block_t));
    return result;
}

int heap_free(heap_t* heap, void* addr) {
    
    if (!addr) {
        return 0;
    }
    
    // 獲取塊頭
    heap_block_t* block = (heap_block_t*)((uintptr_t)addr - sizeof(heap_block_t));
    
    // 檢查地址是否在堆範圍內
    if ((uintptr_t)block < heap->start_addr || (uintptr_t)block >= heap->end_addr) {
        return 0;
    }
    
    // 標記為空閒
    block->is_free = HEAP_BLOCK_FREE;
    
    // 合併相鄰的空閒塊
    merge_adjacent_blocks(heap);
    
    return 1;
}

void* heap_realloc(heap_t* heap, void* addr, size_t new_size) {
    
    if (!addr) {
        return heap_alloc(heap, new_size);
    }
    
    if (new_size == 0) {
        heap_free(heap, addr);
        return NULL;
    }
    
    // 對齊到8字節邊界
    new_size = (new_size + 7) & ~7;
    
    // 獲取塊頭
    heap_block_t* block = (heap_block_t*)((uintptr_t)addr - sizeof(heap_block_t));
    
    // 檢查地址是否在堆範圍內
    if ((uintptr_t)block < heap->start_addr || (uintptr_t)block >= heap->end_addr) {
        return NULL;
    }
    
    // 如果新大小比當前大小小，可以縮小塊
    if (new_size <= block->size) {
        split_block(block, new_size);
        return addr;
    }
    
    // 如果下一個塊是空閒的，檢查合併後是否夠大
    if (block->next && block->next->is_free && 
        (block->size + sizeof(heap_block_t) + block->next->size >= new_size)) {
        
        // 合併塊
        block->size += sizeof(heap_block_t) + block->next->size;
        block->next = block->next->next;
        
        // 如果合併後的空間過大，再分裂
        split_block(block, new_size);
        
        return addr;
    }
    
    // 需要分配新塊
    void* new_addr = heap_alloc(heap, new_size);
    if (!new_addr) {
        return NULL;
    }
    
    // 複製數據
    memcpy(new_addr, addr, block->size);
    
    // 釋放舊塊
    heap_free(heap, addr);
    
    return new_addr;
}

int heap_expand(heap_t* heap, size_t size) {
    
    // 檢查是否超過最大地址
    if (heap->end_addr + size > heap->max_addr) {
        return 0;
    }
    
    // 計算需要的頁數
    size_t pages_needed = (size + 0xFFF) >> 12; // 向上取整到頁大小
    uintptr_t old_end = heap->end_addr;
    
    // 分配新頁
    for (size_t i = 0; i < pages_needed; i++) {
        void* vaddr = (void*)(old_end + (i << 12));
        if (!vmm_alloc_page(vaddr, PG_PREM_RW, PG_PREM_RW)) {
            // 分配失敗，回滾
            for (size_t j = 0; j < i; j++) {
                vmm_unmap_page((void*)(old_end + (j << 12)));
            }
            return 0;
        }
    }
    
    // 更新堆結束地址
    heap->end_addr += pages_needed << 12;
    
    // 添加新的空閒塊或擴展最後一個塊
    heap_block_t* last_block = heap->first_block;
    if (!last_block) {
        // 創建第一個塊
        heap_block_t* new_block = (heap_block_t*)old_end;
        new_block->size = (pages_needed << 12) - sizeof(heap_block_t);
        new_block->is_free = HEAP_BLOCK_FREE;
        new_block->next = NULL;
        heap->first_block = new_block;
    } else {
        // 找到最後一個塊
        while (last_block->next) {
            last_block = last_block->next;
        }
        
        if (last_block->is_free && (uintptr_t)last_block + sizeof(heap_block_t) + last_block->size == old_end) {
            // 擴展最後一個塊
            last_block->size += pages_needed << 12;
        } else {
            // 創建新塊
            heap_block_t* new_block = (heap_block_t*)old_end;
            new_block->size = (pages_needed << 12) - sizeof(heap_block_t);
            new_block->is_free = HEAP_BLOCK_FREE;
            new_block->next = NULL;
            last_block->next = new_block;
        }
    }
    
    return 1;
}

size_t heap_get_free_size(heap_t* heap) {    
    size_t free_size = 0;
    heap_block_t* current = heap->first_block;
    int block_count = 0;
    
    while (current) {
        if (current->is_free) {
            free_size += current->size;
        }
        current = current->next;
    }
    
    return free_size;
}

// 內核內存分配函數
void* kmalloc(size_t size) {    
    if (!kheap) {
        return NULL;
    }
    
    void* ptr = heap_alloc(kheap, size);
    
    return ptr;
}

void kfree(void* addr) {    
    if (!kheap || !addr) {
        return;
    }
    
    heap_free(kheap, addr);
}

void* krealloc(void* addr, size_t size) {
    if (!kheap) {
        return NULL;
    }
    
    void* ptr = heap_realloc(kheap, addr, size);
    
    return ptr;
}
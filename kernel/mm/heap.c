#include <system/mm/heap.h>
#include <system/mm/vmm.h>
#include <system/mm/pmm.h>
#include <system/mm/page.h>
#include <libc/string.h>
#include <libc/stdio.h>

// 全局內核堆
heap_t* kheap = NULL;

static heap_block_t* find_free_block(heap_t* heap, size_t size) {
    printf("[FIND-BLOCK-DEBUG] Entering find_free_block\n");
    printf("[FIND-BLOCK-DEBUG] heap = %p, size = %d\n", heap, size);
    
    heap_block_t* current = heap->first_block;
    printf("[FIND-BLOCK-DEBUG] first_block = %p\n", current);
    
    // 尋找符合大小的第一個空閒塊
    int block_count = 0;
    while (current) {
        printf("[FIND-BLOCK-DEBUG] Checking block %d at %p, is_free = %d, size = %d\n", 
               block_count++, current, current->is_free, current->size);
        
        if (current->is_free && current->size >= size) {
            printf("[FIND-BLOCK-DEBUG] Found suitable block, returning\n");
            return current;
        }
        
        current = current->next;
    }
    
    printf("[FIND-BLOCK-DEBUG] No suitable block found, returning NULL\n");
    return NULL;
}

static heap_block_t* extend_heap(heap_t* heap, size_t size) {
    printf("[EXTEND-HEAP-DEBUG] Entering extend_heap\n");
    printf("[EXTEND-HEAP-DEBUG] heap = %p, size = %d\n", heap, size);
    
    // 檢查是否允許擴展
    if (!(heap->flags & HEAP_AUTO_EXTEND)) {
        printf("[EXTEND-HEAP-DEBUG] Auto-extend not enabled, returning NULL\n");
        return NULL;
    }
    
    // 計算需要擴展的大小（對齊到頁大小）
    size_t block_size = size + sizeof(heap_block_t);
    size_t pages_needed = (block_size + 0xFFF) >> 12; // 向上取整到頁大小
    printf("[EXTEND-HEAP-DEBUG] block_size = %d, pages_needed = %d\n", 
           block_size, pages_needed);
    
    // 檢查是否超過最大地址
    if (heap->end_addr + (pages_needed << 12) > heap->max_addr) {
        printf("[EXTEND-HEAP-DEBUG] Expansion would exceed max_addr, returning NULL\n");
        return NULL;
    }
    
    // 分配新的頁面
    uintptr_t old_end = heap->end_addr;
    printf("[EXTEND-HEAP-DEBUG] old_end = %p\n", (void*)old_end);
    
    printf("[EXTEND-HEAP-DEBUG] Allocating %d pages\n", pages_needed);
    for (size_t i = 0; i < pages_needed; i++) {
        void* vaddr = (void*)(old_end + (i << 12));
        printf("[EXTEND-HEAP-DEBUG] Calling vmm_alloc_page for address %p\n", vaddr);
        
        if (!vmm_alloc_page(vaddr, PG_PREM_RW, PG_PREM_RW)) {
            printf("[EXTEND-HEAP-DEBUG] vmm_alloc_page failed for page %d\n", i);
            // 分配失敗，回滾
            for (size_t j = 0; j < i; j++) {
                printf("[EXTEND-HEAP-DEBUG] Rolling back, unmapping page %d\n", j);
                vmm_unmap_page((void*)(old_end + (j << 12)));
            }
            return NULL;
        }
    }
    
    // 更新堆結束地址
    heap->end_addr += pages_needed << 12;
    printf("[EXTEND-HEAP-DEBUG] Updated end_addr = %p\n", (void*)heap->end_addr);
    
    // 創建新塊
    heap_block_t* new_block = (heap_block_t*)old_end;
    printf("[EXTEND-HEAP-DEBUG] Created new block at %p\n", (void*)new_block);
    
    new_block->size = (pages_needed << 12) - sizeof(heap_block_t);
    new_block->is_free = HEAP_BLOCK_FREE;
    printf("[EXTEND-HEAP-DEBUG] New block size = %d\n", new_block->size);
    
    // 鏈接到堆
    if (!heap->first_block) {
        printf("[EXTEND-HEAP-DEBUG] No first_block, setting new block as first\n");
        heap->first_block = new_block;
        new_block->next = NULL;
    } else {
        printf("[EXTEND-HEAP-DEBUG] Adding new block to the end of the list\n");
        heap_block_t* current = heap->first_block;
        int blocks_traversed = 0;
        while (current->next) {
            current = current->next;
            blocks_traversed++;
        }
        printf("[EXTEND-HEAP-DEBUG] Traversed %d blocks to reach the end\n", blocks_traversed);
        
        current->next = new_block;
        new_block->next = NULL;
    }
    
    printf("[EXTEND-HEAP-DEBUG] Exiting extend_heap, returning new_block = %p\n", (void*)new_block);
    return new_block;
}

static void split_block(heap_block_t* block, size_t size) {
    printf("[SPLIT-BLOCK-DEBUG] Entering split_block\n");
    printf("[SPLIT-BLOCK-DEBUG] block = %p, size = %d, block->size = %d\n", 
           block, size, block->size);
    
    // 只有當剩餘空間足夠容納一個新塊頭加上至少1字節數據時才分裂
    if (block->size > size + sizeof(heap_block_t) + 1) {
        printf("[SPLIT-BLOCK-DEBUG] Block is large enough to split\n");
        
        // 計算新塊的位置
        heap_block_t* new_block = (heap_block_t*)((uintptr_t)block + sizeof(heap_block_t) + size);
        printf("[SPLIT-BLOCK-DEBUG] Created new block at %p\n", new_block);
        
        // 設置新塊屬性
        new_block->size = block->size - size - sizeof(heap_block_t);
        new_block->is_free = HEAP_BLOCK_FREE;
        new_block->next = block->next;
        printf("[SPLIT-BLOCK-DEBUG] New block size = %d\n", new_block->size);
        
        // 更新當前塊
        block->size = size;
        block->next = new_block;
        printf("[SPLIT-BLOCK-DEBUG] Updated current block size = %d\n", block->size);
        
        printf("[SPLIT-BLOCK-DEBUG] Block split successfully\n");
    } else {
        printf("[SPLIT-BLOCK-DEBUG] Block not large enough to split, skipping\n");
    }
    
    printf("[SPLIT-BLOCK-DEBUG] Exiting split_block\n");
}

static void merge_adjacent_blocks(heap_t* heap) {
    printf("[MERGE-BLOCKS-DEBUG] Entering merge_adjacent_blocks\n");
    printf("[MERGE-BLOCKS-DEBUG] heap = %p\n", heap);
    
    heap_block_t* current = heap->first_block;
    printf("[MERGE-BLOCKS-DEBUG] first_block = %p\n", current);
    
    int merges_performed = 0;
    int blocks_traversed = 0;
    
    while (current && current->next) {
        printf("[MERGE-BLOCKS-DEBUG] Checking block %d at %p and next block at %p\n", 
               blocks_traversed, current, current->next);
        
        if (current->is_free && current->next->is_free) {
            printf("[MERGE-BLOCKS-DEBUG] Both blocks are free, merging\n");
            printf("[MERGE-BLOCKS-DEBUG] Current block size = %d, next block size = %d\n",
                   current->size, current->next->size);
            
            // 合併兩個相鄰的空閒塊
            current->size += sizeof(heap_block_t) + current->next->size;
            current->next = current->next->next;
            
            printf("[MERGE-BLOCKS-DEBUG] After merge, block size = %d\n", current->size);
            merges_performed++;
        } else {
            printf("[MERGE-BLOCKS-DEBUG] Blocks not both free, skipping merge\n");
            current = current->next;
            blocks_traversed++;
        }
    }
    
    printf("[MERGE-BLOCKS-DEBUG] Exiting merge_adjacent_blocks\n");
    printf("[MERGE-BLOCKS-DEBUG] Total blocks traversed = %d, merges performed = %d\n",
           blocks_traversed, merges_performed);
}

heap_t* heap_init(uintptr_t start_addr, uintptr_t end_addr, uintptr_t max_addr) {
    // printf("[HEAP-DEBUG] Entering heap_init\n");
    // printf("[HEAP-DEBUG] start_addr = %p\n", (void*)start_addr);
    // printf("[HEAP-DEBUG] end_addr = %p\n", (void*)end_addr);
    // printf("[HEAP-DEBUG] max_addr = %p\n", (void*)max_addr);
    
    // 確保地址對齊
    start_addr = (start_addr + 0xF) & ~0xF; // 16字節對齊
    // printf("[HEAP-DEBUG] Aligned start_addr = %p\n", (void*)start_addr);
    
    // 創建堆結構
    // printf("[HEAP-DEBUG] Creating heap structure\n");
    heap_t* heap = (heap_t*)start_addr;
    start_addr += sizeof(heap_t);
    // printf("[HEAP-DEBUG] Heap structure created at %p\n", (void*)heap);
    // printf("[HEAP-DEBUG] New start_addr after heap structure = %p\n", (void*)start_addr);
    // printf("[HEAP-DEBUG] Heap structure size = %d bytes\n", sizeof(heap_t));
    
    // 初始化堆結構
    // printf("[HEAP-DEBUG] Initializing heap structure fields\n");
    heap->start_addr = start_addr;
    heap->end_addr = end_addr;
    heap->max_addr = max_addr;
    heap->flags = HEAP_AUTO_EXTEND;
    heap->first_block = NULL;
    // printf("[HEAP-DEBUG] Heap structure fields initialized\n");
    
    // 初始化第一個塊
    if (start_addr < end_addr) {
        // printf("[HEAP-DEBUG] Initializing first block\n");
        heap_block_t* first_block = (heap_block_t*)start_addr;
        // printf("[HEAP-DEBUG] First block created at %p\n", (void*)first_block);
        // printf("[HEAP-DEBUG] Block header size = %d bytes\n", sizeof(heap_block_t));
        
        first_block->size = end_addr - start_addr - sizeof(heap_block_t);
        // printf("[HEAP-DEBUG] First block size = %d bytes\n", first_block->size);
        
        first_block->is_free = HEAP_BLOCK_FREE;
        first_block->next = NULL;
        heap->first_block = first_block;
        // printf("[HEAP-DEBUG] First block initialized\n");
    } else {
        printf("[HEAP-DEBUG] start_addr >= end_addr, skipping first block creation\n");
    }
    
    // printf("[HEAP-DEBUG] Heap initialization complete, returning heap = %p\n", (void*)heap);
    return heap;
}

void* heap_alloc(heap_t* heap, size_t size) {
    printf("[HEAP-ALLOC-DEBUG] Entering heap_alloc\n");
    printf("[HEAP-ALLOC-DEBUG] heap = %p, size = %d\n", heap, size);
    
    // 對齊到8字節邊界
    size = (size + 7) & ~7;
    printf("[HEAP-ALLOC-DEBUG] Aligned size = %d\n", size);
    
    if (size == 0) {
        printf("[HEAP-ALLOC-DEBUG] Size is 0, returning NULL\n");
        return NULL;
    }
    
    // 尋找空閒塊
    printf("[HEAP-ALLOC-DEBUG] Looking for free block\n");
    heap_block_t* block = find_free_block(heap, size);
    printf("[HEAP-ALLOC-DEBUG] find_free_block returned block = %p\n", block);
    
    // 如果沒有找到合適的塊，嘗試擴展堆
    if (!block) {
        printf("[HEAP-ALLOC-DEBUG] No suitable block found, trying to extend heap\n");
        block = extend_heap(heap, size);
        if (!block) {
            printf("[HEAP-ALLOC-DEBUG] Failed to extend heap, returning NULL\n");
            return NULL; // 無法分配
        }
        printf("[HEAP-ALLOC-DEBUG] Heap extended, got block at %p\n", block);
    }
    
    // 分裂塊（如果可能）
    printf("[HEAP-ALLOC-DEBUG] Splitting block if possible\n");
    split_block(block, size);
    
    // 標記為已使用
    printf("[HEAP-ALLOC-DEBUG] Marking block as used\n");
    block->is_free = HEAP_BLOCK_USED;
    
    // 返回數據區域指針
    void* result = (void*)((uintptr_t)block + sizeof(heap_block_t));
    printf("[HEAP-ALLOC-DEBUG] Returning data area at %p\n", result);
    return result;
}

int heap_free(heap_t* heap, void* addr) {
    printf("[HEAP-FREE-DEBUG] Entering heap_free\n");
    printf("[HEAP-FREE-DEBUG] heap = %p, addr = %p\n", heap, addr);
    
    if (!addr) {
        printf("[HEAP-FREE-DEBUG] addr is NULL, returning 0\n");
        return 0;
    }
    
    // 獲取塊頭
    heap_block_t* block = (heap_block_t*)((uintptr_t)addr - sizeof(heap_block_t));
    printf("[HEAP-FREE-DEBUG] Block header at %p\n", block);
    
    // 檢查地址是否在堆範圍內
    if ((uintptr_t)block < heap->start_addr || (uintptr_t)block >= heap->end_addr) {
        printf("[HEAP-FREE-DEBUG] Block not in heap range, returning 0\n");
        return 0;
    }
    
    // 標記為空閒
    printf("[HEAP-FREE-DEBUG] Marking block as free\n");
    block->is_free = HEAP_BLOCK_FREE;
    
    // 合併相鄰的空閒塊
    printf("[HEAP-FREE-DEBUG] Merging adjacent free blocks\n");
    merge_adjacent_blocks(heap);
    
    printf("[HEAP-FREE-DEBUG] heap_free successful, returning 1\n");
    return 1;
}

void* heap_realloc(heap_t* heap, void* addr, size_t new_size) {
    printf("[HEAP-REALLOC-DEBUG] Entering heap_realloc\n");
    printf("[HEAP-REALLOC-DEBUG] heap = %p, addr = %p, new_size = %d\n", heap, addr, new_size);
    
    if (!addr) {
        printf("[HEAP-REALLOC-DEBUG] addr is NULL, calling heap_alloc\n");
        return heap_alloc(heap, new_size);
    }
    
    if (new_size == 0) {
        printf("[HEAP-REALLOC-DEBUG] new_size is 0, freeing memory and returning NULL\n");
        heap_free(heap, addr);
        return NULL;
    }
    
    // 對齊到8字節邊界
    new_size = (new_size + 7) & ~7;
    printf("[HEAP-REALLOC-DEBUG] Aligned new_size = %d\n", new_size);
    
    // 獲取塊頭
    heap_block_t* block = (heap_block_t*)((uintptr_t)addr - sizeof(heap_block_t));
    printf("[HEAP-REALLOC-DEBUG] Block header at %p, current size = %d\n", block, block->size);
    
    // 檢查地址是否在堆範圍內
    if ((uintptr_t)block < heap->start_addr || (uintptr_t)block >= heap->end_addr) {
        printf("[HEAP-REALLOC-DEBUG] Block not in heap range, returning NULL\n");
        return NULL;
    }
    
    // 如果新大小比當前大小小，可以縮小塊
    if (new_size <= block->size) {
        printf("[HEAP-REALLOC-DEBUG] New size <= current size, splitting block\n");
        split_block(block, new_size);
        return addr;
    }
    
    // 如果下一個塊是空閒的，檢查合併後是否夠大
    if (block->next && block->next->is_free && 
        (block->size + sizeof(heap_block_t) + block->next->size >= new_size)) {
        printf("[HEAP-REALLOC-DEBUG] Next block is free and large enough, merging\n");
        
        // 合併塊
        block->size += sizeof(heap_block_t) + block->next->size;
        block->next = block->next->next;
        printf("[HEAP-REALLOC-DEBUG] After merge, block size = %d\n", block->size);
        
        // 如果合併後的空間過大，再分裂
        printf("[HEAP-REALLOC-DEBUG] Splitting merged block if needed\n");
        split_block(block, new_size);
        
        return addr;
    }
    
    // 需要分配新塊
    printf("[HEAP-REALLOC-DEBUG] Need to allocate new block\n");
    void* new_addr = heap_alloc(heap, new_size);
    if (!new_addr) {
        printf("[HEAP-REALLOC-DEBUG] Failed to allocate new block, returning NULL\n");
        return NULL;
    }
    
    // 複製數據
    printf("[HEAP-REALLOC-DEBUG] Copying data from old block to new\n");
    memcpy(new_addr, addr, block->size);
    
    // 釋放舊塊
    printf("[HEAP-REALLOC-DEBUG] Freeing old block\n");
    heap_free(heap, addr);
    
    printf("[HEAP-REALLOC-DEBUG] Reallocation successful, returning new addr = %p\n", new_addr);
    return new_addr;
}

int heap_expand(heap_t* heap, size_t size) {
    printf("[HEAP-EXPAND-DEBUG] Entering heap_expand\n");
    printf("[HEAP-EXPAND-DEBUG] heap = %p, size = %d\n", heap, size);
    
    // 檢查是否超過最大地址
    if (heap->end_addr + size > heap->max_addr) {
        printf("[HEAP-EXPAND-DEBUG] Expansion would exceed max_addr, returning 0\n");
        return 0;
    }
    
    // 計算需要的頁數
    size_t pages_needed = (size + 0xFFF) >> 12; // 向上取整到頁大小
    uintptr_t old_end = heap->end_addr;
    printf("[HEAP-EXPAND-DEBUG] pages_needed = %d, old_end = %p\n", pages_needed, (void*)old_end);
    
    // 分配新頁
    printf("[HEAP-EXPAND-DEBUG] Allocating pages\n");
    for (size_t i = 0; i < pages_needed; i++) {
        void* vaddr = (void*)(old_end + (i << 12));
        if (!vmm_alloc_page(vaddr, PG_PREM_RW, PG_PREM_RW)) {
            printf("[HEAP-EXPAND-DEBUG] Failed to allocate page %d, rolling back\n", i);
            // 分配失敗，回滾
            for (size_t j = 0; j < i; j++) {
                vmm_unmap_page((void*)(old_end + (j << 12)));
            }
            return 0;
        }
    }
    
    // 更新堆結束地址
    heap->end_addr += pages_needed << 12;
    printf("[HEAP-EXPAND-DEBUG] New end_addr = %p\n", (void*)heap->end_addr);
    
    // 添加新的空閒塊或擴展最後一個塊
    heap_block_t* last_block = heap->first_block;
    if (!last_block) {
        // 創建第一個塊
        printf("[HEAP-EXPAND-DEBUG] No first block, creating new first block\n");
        heap_block_t* new_block = (heap_block_t*)old_end;
        new_block->size = (pages_needed << 12) - sizeof(heap_block_t);
        new_block->is_free = HEAP_BLOCK_FREE;
        new_block->next = NULL;
        heap->first_block = new_block;
        printf("[HEAP-EXPAND-DEBUG] New first block created at %p, size = %d\n", 
               new_block, new_block->size);
    } else {
        // 找到最後一個塊
        printf("[HEAP-EXPAND-DEBUG] Finding last block\n");
        while (last_block->next) {
            last_block = last_block->next;
        }
        printf("[HEAP-EXPAND-DEBUG] Last block found at %p\n", last_block);
        
        if (last_block->is_free && (uintptr_t)last_block + sizeof(heap_block_t) + last_block->size == old_end) {
            // 擴展最後一個塊
            printf("[HEAP-EXPAND-DEBUG] Last block is free and adjacent, extending it\n");
            last_block->size += pages_needed << 12;
            printf("[HEAP-EXPAND-DEBUG] Extended last block size = %d\n", last_block->size);
        } else {
            // 創建新塊
            printf("[HEAP-EXPAND-DEBUG] Creating new block at end\n");
            heap_block_t* new_block = (heap_block_t*)old_end;
            new_block->size = (pages_needed << 12) - sizeof(heap_block_t);
            new_block->is_free = HEAP_BLOCK_FREE;
            new_block->next = NULL;
            last_block->next = new_block;
            printf("[HEAP-EXPAND-DEBUG] New block created at %p, size = %d\n", 
                   new_block, new_block->size);
        }
    }
    
    printf("[HEAP-EXPAND-DEBUG] heap_expand successful, returning 1\n");
    return 1;
}

size_t heap_get_free_size(heap_t* heap) {
    printf("[HEAP-FREE-SIZE-DEBUG] Entering heap_get_free_size\n");
    printf("[HEAP-FREE-SIZE-DEBUG] heap = %p\n", heap);
    
    size_t free_size = 0;
    heap_block_t* current = heap->first_block;
    int block_count = 0;
    
    while (current) {
        printf("[HEAP-FREE-SIZE-DEBUG] Checking block %d at %p\n", block_count++, current);
        if (current->is_free) {
            free_size += current->size;
            printf("[HEAP-FREE-SIZE-DEBUG] Block is free, size = %d, running total = %d\n", 
                   current->size, free_size);
        }
        current = current->next;
    }
    
    printf("[HEAP-FREE-SIZE-DEBUG] Exiting heap_get_free_size, total free size = %d\n", free_size);
    return free_size;
}

// 內核內存分配函數
void* kmalloc(size_t size) {
    printf("[KMALLOC-DEBUG] kmalloc called with size = %d\n", size);
    
    if (!kheap) {
        printf("[KMALLOC-DEBUG] kheap is NULL, returning NULL\n");
        return NULL;
    }
    
    printf("[KMALLOC-DEBUG] kheap = %p, calling heap_alloc\n", kheap);
    void* ptr = heap_alloc(kheap, size);
    printf("[KMALLOC-DEBUG] heap_alloc returned ptr = %p\n", ptr);
    
    return ptr;
}

void kfree(void* addr) {
    printf("[KFREE-DEBUG] Entering kfree\n");
    printf("[KFREE-DEBUG] addr = %p\n", addr);
    
    if (!kheap || !addr) {
        printf("[KFREE-DEBUG] kheap is NULL or addr is NULL, returning\n");
        return;
    }
    
    printf("[KFREE-DEBUG] Calling heap_free\n");
    heap_free(kheap, addr);
    printf("[KFREE-DEBUG] Exiting kfree\n");
}

void* krealloc(void* addr, size_t size) {
    printf("[KREALLOC-DEBUG] Entering krealloc\n");
    printf("[KREALLOC-DEBUG] addr = %p, size = %d\n", addr, size);
    
    if (!kheap) {
        printf("[KREALLOC-DEBUG] kheap is NULL, returning NULL\n");
        return NULL;
    }
    
    printf("[KREALLOC-DEBUG] Calling heap_realloc\n");
    void* ptr = heap_realloc(kheap, addr, size);
    printf("[KREALLOC-DEBUG] heap_realloc returned ptr = %p\n", ptr);
    
    return ptr;
}
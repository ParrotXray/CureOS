#include <system/mm/heap_debug.h>
#include <libc/stdio.h>
#include <libc/string.h>

// 調試狀態
static int debug_enabled = 0;
static heap_stats_t global_stats = {0};
static allocation_record_t* allocation_records = NULL;

void heap_debug_enable(int enable) {
    debug_enabled = enable;
    
    // 重置統計信息
    if (enable) {
        memset(&global_stats, 0, sizeof(heap_stats_t));
    }
    
    // 清理追蹤記錄
    if (!enable && allocation_records) {
        allocation_record_t* current = allocation_records;
        while (current) {
            allocation_record_t* next = current->next;
            kfree(current);
            current = next;
        }
        allocation_records = NULL;
    }
}

void heap_get_stats(heap_t* heap, heap_stats_t* stats) {
    if (!stats) return;
    
    // 復制全局統計信息
    *stats = global_stats;
    
    // 計算當前堆使用情況
    size_t free_size = heap_get_free_size(heap);
    size_t total_heap_size = heap->end_addr - heap->start_addr;
    stats->current_allocated_bytes = total_heap_size - free_size;
}

void heap_print_usage(heap_t* heap) {
    heap_stats_t stats;
    heap_get_stats(heap, &stats);
    
    printf("=== Heap Usage Statistics ===\n");
    printf("Total allocations: %d\n", stats.total_allocations);
    printf("Total frees: %d\n", stats.total_frees);
    printf("Current allocations: %d\n", stats.current_allocations);
    printf("Peak allocations: %d\n", stats.peak_allocations);
    printf("Total allocated bytes: %d\n", stats.total_allocated_bytes);
    printf("Current allocated bytes: %d\n", stats.current_allocated_bytes);
    printf("Peak allocated bytes: %d\n", stats.peak_allocated_bytes);
    printf("Total overhead bytes: %d\n", stats.total_overhead_bytes);
    
    size_t total_heap_size = heap->end_addr - heap->start_addr;
    printf("Total heap size: %d\n", total_heap_size);
    printf("Free heap size: %d\n", heap_get_free_size(heap));
    printf("Heap utilization: %d%%\n", 
           (stats.current_allocated_bytes * 100) / total_heap_size);
    printf("==============================\n");
}

void heap_print_layout(heap_t* heap) {
    if (!heap) return;
    
    printf("=== Heap Layout ===\n");
    printf("Heap start: %p\n", heap->start_addr);
    printf("Heap end: %p\n", heap->end_addr);
    printf("Heap max: %p\n", heap->max_addr);
    
    // 打印塊鏈表
    heap_block_t* current = heap->first_block;
    int block_count = 0;
    
    printf("\nBlock layout:\n");
    while (current) {
        printf("Block %d: addr=%p, size=%d, %s\n", 
               block_count++, 
               current, 
               current->size, 
               current->is_free ? "FREE" : "USED");
        current = current->next;
    }
    printf("=====================\n");
}

int heap_check_integrity(heap_t* heap) {
    if (!heap) return -1;
    
    heap_block_t* current = heap->first_block;
    int errors = 0;
    int block_count = 0;
    
    // 檢查塊鏈表
    while (current) {
        // 檢查地址範圍
        if ((uintptr_t)current < heap->start_addr || 
            (uintptr_t)current + sizeof(heap_block_t) + current->size > heap->end_addr) {
            printf("ERROR: Block %d at %p is outside heap range\n", 
                   block_count, current);
            errors++;
        }
        
        // 檢查與下一個塊的連續性
        if (current->next) {
            uintptr_t expected_next = (uintptr_t)current + sizeof(heap_block_t) + current->size;
            if (expected_next != (uintptr_t)current->next) {
                printf("ERROR: Block %d at %p is not contiguous with next block at %p\n", 
                       block_count, current, current->next);
                errors++;
            }
        }
        
        block_count++;
        current = current->next;
    }
    
    return errors;
}

void heap_track_alloc(void* addr, size_t size, const char* file, int line) {
    if (!debug_enabled || !addr) return;
    
    // 更新統計信息
    global_stats.total_allocations++;
    global_stats.current_allocations++;
    global_stats.total_allocated_bytes += size;
    global_stats.current_allocated_bytes += size;
    global_stats.total_overhead_bytes += sizeof(heap_block_t);
    
    // 更新峰值
    if (global_stats.current_allocations > global_stats.peak_allocations) {
        global_stats.peak_allocations = global_stats.current_allocations;
    }
    if (global_stats.current_allocated_bytes > global_stats.peak_allocated_bytes) {
        global_stats.peak_allocated_bytes = global_stats.current_allocated_bytes;
    }
    
    // 創建分配記錄
    allocation_record_t* record = kmalloc(sizeof(allocation_record_t));
    if (!record) return; // 分配失敗
    
    record->address = addr;
    record->size = size;
    record->file = file;
    record->line = line;
    
    // 添加到鏈表頭部
    record->next = allocation_records;
    allocation_records = record;
}

void heap_track_free(void* addr) {
    if (!debug_enabled || !addr) return;
    
    // 尋找記錄
    allocation_record_t* prev = NULL;
    allocation_record_t* current = allocation_records;
    
    while (current) {
        if (current->address == addr) {
            // 更新統計信息
            global_stats.total_frees++;
            global_stats.current_allocations--;
            global_stats.current_allocated_bytes -= current->size;
            
            // 從鏈表中移除
            if (prev) {
                prev->next = current->next;
            } else {
                allocation_records = current->next;
            }
            
            // 釋放記錄
            kfree(current);
            return;
        }
        
        prev = current;
        current = current->next;
    }
    
    // 如果沒找到記錄
    printf("WARNING: Trying to free untracked memory at %p\n", addr);
}

void heap_print_leaks() {
    if (!debug_enabled) return;
    
    printf("=== Memory Leak Report ===\n");
    if (!allocation_records) {
        printf("No memory leaks detected.\n");
        return;
    }
    
    allocation_record_t* current = allocation_records;
    int leak_count = 0;
    size_t total_leaked = 0;
    
    while (current) {
        printf("Leak %d: %d bytes at %p, allocated in %s:%d\n", 
               leak_count++, 
               current->size, 
               current->address, 
               current->file, 
               current->line);
        
        total_leaked += current->size;
        current = current->next;
    }
    
    printf("Total memory leaks: %d (%d bytes)\n", leak_count, total_leaked);
    printf("==========================\n");
}

#ifdef HEAP_DEBUG
void* kmalloc_debug(size_t size, const char* file, int line) {
    void* ptr = kmalloc(size);
    heap_track_alloc(ptr, size, file, line);
    return ptr;
}

void kfree_debug(void* ptr) {
    heap_track_free(ptr);
    kfree(ptr);
}

void* krealloc_debug(void* ptr, size_t size, const char* file, int line) {
    if (ptr) {
        heap_track_free(ptr);
    }
    
    void* new_ptr = krealloc(ptr, size);
    heap_track_alloc(new_ptr, size, file, line);
    return new_ptr;
}
#endif
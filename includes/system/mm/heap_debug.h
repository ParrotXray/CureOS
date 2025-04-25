#ifndef _LUNAIX_HEAP_DEBUG_H
#define _LUNAIX_HEAP_DEBUG_H

#include <system/mm/heap.h>
#include <stdint.h>
#include <stddef.h>

// 堆統計信息結構
typedef struct {
    size_t total_allocations;      // 總分配次數
    size_t total_frees;            // 總釋放次數
    size_t current_allocations;    // 當前分配數量
    size_t peak_allocations;       // 峰值分配數量
    size_t total_allocated_bytes;  // 總分配字節數
    size_t current_allocated_bytes; // 當前分配字節數
    size_t peak_allocated_bytes;   // 峰值分配字節數
    size_t total_overhead_bytes;   // 總開銷字節數(塊頭的大小)
} heap_stats_t;

// 分配追蹤記錄，用於調試
typedef struct allocation_record {
    void* address;                // 分配的地址
    size_t size;                  // 分配的大小
    const char* file;             // 分配位置的文件
    int line;                     // 分配位置的行號
    struct allocation_record* next; // 鏈表的下一個節點
} allocation_record_t;

/**
 * @brief 啟用/禁用堆調試功能
 * 
 * @param enable 是否啟用
 */
void heap_debug_enable(int enable);

/**
 * @brief 獲取堆統計信息
 * 
 * @param heap 目標堆
 * @param stats 統計信息輸出結構
 */
void heap_get_stats(heap_t* heap, heap_stats_t* stats);

/**
 * @brief 打印堆的使用情況
 * 
 * @param heap 目標堆
 */
void heap_print_usage(heap_t* heap);

/**
 * @brief 打印堆的內存塊布局
 * 
 * @param heap 目標堆
 */
void heap_print_layout(heap_t* heap);

/**
 * @brief 檢查堆是否存在錯誤（如損壞的塊鏈表）
 * 
 * @param heap 目標堆
 * @return int 如果找到錯誤，返回非零值
 */
int heap_check_integrity(heap_t* heap);

/**
 * @brief 追蹤內存分配，記錄分配點
 * 
 * @param addr 分配的地址
 * @param size 分配的大小
 * @param file 源文件名
 * @param line 行號
 */
void heap_track_alloc(void* addr, size_t size, const char* file, int line);

/**
 * @brief 追蹤內存釋放
 * 
 * @param addr 釋放的地址
 */
void heap_track_free(void* addr);

/**
 * @brief 打印所有未釋放的內存分配
 */
void heap_print_leaks();

// 如果啟用了堆調試，則重新定義內存分配函數，包含追蹤信息
#ifdef HEAP_DEBUG
    #define kmalloc(size) kmalloc_debug(size, __FILE__, __LINE__)
    #define kfree(ptr) kfree_debug(ptr)
    #define krealloc(ptr, size) krealloc_debug(ptr, size, __FILE__, __LINE__)
    
    void* kmalloc_debug(size_t size, const char* file, int line);
    void kfree_debug(void* ptr);
    void* krealloc_debug(void* ptr, size_t size, const char* file, int line);
#endif

#endif /* _LUNAIX_HEAP_DEBUG_H */
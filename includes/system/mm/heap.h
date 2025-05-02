#ifndef _LUNAIX_HEAP_H
#define _LUNAIX_HEAP_H

#include <stddef.h>
#include <stdint.h>

// 堆塊的可能狀態
#define HEAP_BLOCK_FREE       0
#define HEAP_BLOCK_USED       1

// 堆標誌
#define HEAP_AUTO_EXTEND      1   // 是否自動擴展堆

// 堆塊頭結構 (確保 16 位元組對齊)
typedef struct heap_block {
    uint32_t size;                 // 塊的大小（不包括此頭）
    uint8_t is_free;               // 塊是否空閒
    struct heap_block* next;       // 下一個塊
} __attribute__((aligned(16))) heap_block_t;

// 堆結構
typedef struct {
    uintptr_t start_addr;          // 堆起始地址
    uintptr_t end_addr;            // 堆結束地址
    uintptr_t max_addr;            // 堆最大地址
    uint8_t flags;                 // 堆標誌
    heap_block_t* first_block;     // 第一個塊
} heap_t;

/**
 * @brief 初始化內核堆
 * 
 * @param start_addr 堆起始地址
 * @param end_addr 堆結束地址
 * @param max_addr 堆最大地址
 * @return heap_t* 返回堆指針
 */
heap_t* heap_init(uintptr_t start_addr, uintptr_t end_addr, uintptr_t max_addr);

/**
 * @brief 在指定堆中分配內存
 * 
 * @param heap 目標堆
 * @param size 欲分配的大小
 * @return void* 分配的內存指針，失敗則NULL
 */
void* heap_alloc(heap_t* heap, size_t size);

/**
 * @brief 在指定堆中釋放內存
 * 
 * @param heap 目標堆
 * @param addr 欲釋放的內存地址
 * @return int 返回1代表成功，0則失敗
 */
int heap_free(heap_t* heap, void* addr);

/**
 * @brief 在指定堆中擴展內存塊
 * 
 * @param heap 目標堆
 * @param addr 欲擴展的內存地址
 * @param new_size 新的大小
 * @return void* 擴展後的內存指針，失敗則NULL
 */
void* heap_realloc(heap_t* heap, void* addr, size_t new_size);

/**
 * @brief 擴展堆
 * 
 * @param heap 目標堆
 * @param size 擴展的大小
 * @return int 返回1代表成功，0則失敗
 */
int heap_expand(heap_t* heap, size_t size);

/**
 * @brief 獲取目標堆的可用內存大小
 * 
 * @param heap 目標堆
 * @return size_t 可用內存大小
 */
size_t heap_get_free_size(heap_t* heap);

// 內核全局堆聲明
extern heap_t* kheap;

/**
 * @brief 內核malloc
 * 
 * @param size 欲分配大小
 * @return void* 分配的內存，失敗則NULL
 */
void* kmalloc(size_t size);

/**
 * @brief 內核free
 * 
 * @param addr 欲釋放的內存
 */
void kfree(void* addr);

/**
 * @brief 內核realloc
 * 
 * @param addr 欲擴展的內存
 * @param size 新的大小
 * @return void* 擴展後的內存，失敗則NULL
 */
void* krealloc(void* addr, size_t size);

#endif /* _LUNAIX_HEAP_H */
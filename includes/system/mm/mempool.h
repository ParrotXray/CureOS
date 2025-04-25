#ifndef _LUNAIX_MEMPOOL_H
#define _LUNAIX_MEMPOOL_H

#include <stddef.h>
#include <stdint.h>

// 內存池塊結構
typedef struct mempool_block {
    struct mempool_block* next;  // 鏈表下一個節點
} mempool_block_t;

// 內存池結構
typedef struct {
    size_t block_size;           // 每個塊的大小（不包括頭部）
    size_t total_blocks;         // 總共分配的塊數
    size_t free_blocks;          // 可用塊數
    mempool_block_t* free_list;  // 空閒塊鏈表
    void* pool_addr;             // 內存池基址
} mempool_t;

/**
 * @brief 創建一個新的內存池
 * 
 * @param block_size 每個塊的大小
 * @param num_blocks 池中塊的總數量
 * @return mempool_t* 內存池指針，失敗則NULL
 */
mempool_t* mempool_create(size_t block_size, size_t num_blocks);

/**
 * @brief 從內存池中分配一個塊
 * 
 * @param pool 目標內存池
 * @return void* 分配的塊指針，失敗則NULL
 */
void* mempool_alloc(mempool_t* pool);

/**
 * @brief 將一個塊釋放回內存池
 * 
 * @param pool 目標內存池
 * @param addr 塊地址
 * @return int 成功返回1，失敗返回0
 */
int mempool_free(mempool_t* pool, void* addr);

/**
 * @brief 銷毀內存池並釋放其佔用的內存
 * 
 * @param pool 目標內存池
 */
void mempool_destroy(mempool_t* pool);

/**
 * @brief 獲取內存池的使用統計信息
 * 
 * @param pool 目標內存池
 * @param total_blocks 總塊數輸出
 * @param free_blocks 空閒塊數輸出
 * @param usage_percent 使用率百分比輸出
 */
void mempool_get_stats(mempool_t* pool, size_t* total_blocks, size_t* free_blocks, int* usage_percent);

/**
 * @brief 擴展內存池
 * 
 * @param pool 目標內存池
 * @param additional_blocks 新增的塊數量
 * @return int 成功返回1，失敗返回0
 */
int mempool_expand(mempool_t* pool, size_t additional_blocks);

#endif /* _LUNAIX_MEMPOOL_H */
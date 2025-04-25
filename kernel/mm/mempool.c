#include <system/mm/mempool.h>
#include <system/mm/heap.h>
#include <libc/string.h>

// 檢查地址是否在內存池範圍內
static int is_address_in_pool(mempool_t* pool, void* addr) {
    uintptr_t pool_start = (uintptr_t)pool->pool_addr;
    uintptr_t pool_end = pool_start + (pool->total_blocks * (pool->block_size + sizeof(mempool_block_t)));
    
    return ((uintptr_t)addr >= pool_start && (uintptr_t)addr < pool_end);
}

mempool_t* mempool_create(size_t block_size, size_t num_blocks) {
    if (block_size == 0 || num_blocks == 0) {
        return NULL;
    }
    
    // 確保塊大小至少能容納一個指針（用於空閒鏈表）
    if (block_size < sizeof(void*)) {
        block_size = sizeof(void*);
    }
    
    // 對齊到8字節邊界
    block_size = (block_size + 7) & ~7;
    
    // 分配內存池結構
    mempool_t* pool = (mempool_t*)kmalloc(sizeof(mempool_t));
    if (!pool) {
        return NULL;
    }
    
    // 分配實際的池內存
    size_t total_size = num_blocks * (block_size + sizeof(mempool_block_t));
    void* pool_mem = kmalloc(total_size);
    if (!pool_mem) {
        kfree(pool);
        return NULL;
    }
    
    // 初始化內存池結構
    pool->block_size = block_size;
    pool->total_blocks = num_blocks;
    pool->free_blocks = num_blocks;
    pool->pool_addr = pool_mem;
    pool->free_list = NULL;
    
    // 初始化空閒鏈表
    for (size_t i = 0; i < num_blocks; i++) {
        mempool_block_t* block = (mempool_block_t*)((uintptr_t)pool_mem + i * (block_size + sizeof(mempool_block_t)));
        block->next = pool->free_list;
        pool->free_list = block;
    }
    
    return pool;
}

void* mempool_alloc(mempool_t* pool) {
    if (!pool || !pool->free_list) {
        return NULL;
    }
    
    // 從空閒鏈表中取出一個塊
    mempool_block_t* block = pool->free_list;
    pool->free_list = block->next;
    pool->free_blocks--;
    
    // 返回塊的數據區域
    return (void*)((uintptr_t)block + sizeof(mempool_block_t));
}

int mempool_free(mempool_t* pool, void* addr) {
    if (!pool || !addr) {
        return 0;
    }
    
    // 獲取塊頭
    mempool_block_t* block = (mempool_block_t*)((uintptr_t)addr - sizeof(mempool_block_t));
    
    // 檢查地址是否在池範圍內
    if (!is_address_in_pool(pool, block)) {
        return 0;
    }
    
    // 添加到空閒鏈表
    block->next = pool->free_list;
    pool->free_list = block;
    pool->free_blocks++;
    
    return 1;
}

void mempool_destroy(mempool_t* pool) {
    if (!pool) {
        return;
    }
    
    // 釋放池內存
    if (pool->pool_addr) {
        kfree(pool->pool_addr);
    }
    
    // 釋放池結構
    kfree(pool);
}

void mempool_get_stats(mempool_t* pool, size_t* total_blocks, size_t* free_blocks, int* usage_percent) {
    if (!pool) {
        return;
    }
    
    if (total_blocks) {
        *total_blocks = pool->total_blocks;
    }
    
    if (free_blocks) {
        *free_blocks = pool->free_blocks;
    }
    
    if (usage_percent) {
        *usage_percent = (pool->total_blocks - pool->free_blocks) * 100 / pool->total_blocks;
    }
}

int mempool_expand(mempool_t* pool, size_t additional_blocks) {
    if (!pool || additional_blocks == 0) {
        return 0;
    }
    
    // 分配新的內存
    size_t new_size = additional_blocks * (pool->block_size + sizeof(mempool_block_t));
    void* new_mem = kmalloc(new_size);
    if (!new_mem) {
        return 0;
    }
    
    // 初始化新塊並添加到空閒鏈表
    for (size_t i = 0; i < additional_blocks; i++) {
        mempool_block_t* block = (mempool_block_t*)((uintptr_t)new_mem + i * (pool->block_size + sizeof(mempool_block_t)));
        block->next = pool->free_list;
        pool->free_list = block;
    }
    
    // 更新池統計信息
    pool->total_blocks += additional_blocks;
    pool->free_blocks += additional_blocks;
    
    return 1;
}
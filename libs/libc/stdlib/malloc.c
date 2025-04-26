#include <libc/stdlib.h>
#include <system/mm/heap.h>
#include <libc/string.h>
#include <libc/stdio.h>

// 標準庫内存分配函數，在内核中直接使用内核堆

void* malloc(size_t size) {
    void* ptr = kmalloc(size);
    return ptr;
}

void free(void* ptr) {
    kfree(ptr);
}

void* realloc(void* ptr, size_t size) {
    void* new_ptr = krealloc(ptr, size);
    return new_ptr;
}

void* calloc(size_t nmemb, size_t size) {    
    // 檢查乘法溢出
    if (size && nmemb > (size_t)-1 / size) {
        return NULL;
    }
    
    size_t total_size = nmemb * size;
    
    void* ptr = kmalloc(total_size);
    if (!ptr) {
        return NULL;
    }
    
    memset(ptr, 0, total_size);
    
    return ptr;
}
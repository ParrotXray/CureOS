#include <libc/stdlib.h>
#include <system/mm/heap.h>
#include <libc/string.h>
#include <libc/stdio.h>

// 標準庫内存分配函數，在内核中直接使用内核堆

void* malloc(size_t size) {
    printf("[MALLOC-DEBUG] Entering malloc with size = %d\n", size);
    void* ptr = kmalloc(size);
    printf("[MALLOC-DEBUG] kmalloc returned ptr = %p\n", ptr);
    return ptr;
}

void free(void* ptr) {
    printf("[FREE-DEBUG] Entering free with ptr = %p\n", ptr);
    kfree(ptr);
    printf("[FREE-DEBUG] Exiting free\n");
}

void* realloc(void* ptr, size_t size) {
    printf("[REALLOC-DEBUG] Entering realloc with ptr = %p, size = %d\n", ptr, size);
    void* new_ptr = krealloc(ptr, size);
    printf("[REALLOC-DEBUG] krealloc returned new_ptr = %p\n", new_ptr);
    return new_ptr;
}

void* calloc(size_t nmemb, size_t size) {
    printf("[CALLOC-DEBUG] Entering calloc with nmemb = %d, size = %d\n", nmemb, size);
    
    // 檢查乘法溢出
    if (size && nmemb > (size_t)-1 / size) {
        printf("[CALLOC-DEBUG] Multiplication overflow detected, returning NULL\n");
        return NULL;
    }
    
    size_t total_size = nmemb * size;
    printf("[CALLOC-DEBUG] Total size = %d, calling kmalloc\n", total_size);
    
    void* ptr = kmalloc(total_size);
    if (!ptr) {
        printf("[CALLOC-DEBUG] kmalloc failed, returning NULL\n");
        return NULL;
    }
    
    printf("[CALLOC-DEBUG] kmalloc returned ptr = %p, clearing memory\n", ptr);
    memset(ptr, 0, total_size);
    
    printf("[CALLOC-DEBUG] Exiting calloc, returning ptr = %p\n", ptr);
    return ptr;
}
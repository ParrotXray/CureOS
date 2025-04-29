#ifndef _SYSTEM_STDLIB_H
#define _SYSTEM_STDLIB_H

#include <stddef.h>  // 添加這一行以定義 size_t 類型

char* __uitoa_internal(unsigned int value, char* str, int base, unsigned int* size);
char* __itoa_internal(int value, char* str, int base, unsigned int* size);

char* itoa(int value, char* str, int base);

/**
 * @brief 分配指定大小的内存
 * 
 * @param size 欲分配的大小
 * @return void* 分配的内存指針，失敗則NULL
 */
void* malloc(size_t size);

/**
 * @brief 釋放内存
 * 
 * @param ptr 欲釋放的内存指針
 */
void free(void* ptr);

/**
 * @brief 重新分配内存
 * 
 * @param ptr 原内存指針
 * @param size 新的大小
 * @return void* 新的内存指針，失敗則NULL
 */
void* realloc(void* ptr, size_t size);

/**
 * @brief 分配並清零内存
 * 
 * @param nmemb 元素數量
 * @param size 元素大小
 * @return void* 分配的内存指針，失敗則NULL
 */
void* calloc(size_t nmemb, size_t size);

#endif /* _SYSTEM_STDLIB_H */
#include <stddef.h>

int memcmp(const void*, const void*, size_t);

void* memcpy(void* __restrict, const void* __restrict, size_t);

void* memmove(void*, const void*, size_t);

void* memset(void*, int, size_t);

size_t strlen(const char* str);

size_t strnlen(const char* str, size_t max_len);

char* strcpy(char* dest, const char* src);

const char* strchr(const char* str, int character);

char* strncpy(char* dest, const char* src, size_t n); 

int strcmp(const char* s1, const char* s2);

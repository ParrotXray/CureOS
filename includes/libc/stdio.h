#include <stdarg.h>
#include <stddef.h>

void __sprintf_internal(char* buffer, char* fmt, size_t max_len, va_list vargs);

void sprintf(char* buffer, char* fmt, ...);
void snprintf(char* buffer, size_t n, char* fmt, ...);
void printf(char* fmt, ...);


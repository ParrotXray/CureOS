#include <stdarg.h>
#include <stddef.h>

void __sprintf_internal(char* buffer, char* fmt, va_list args);

void sprintf(char* buffer, char* fmt, ...);
void printf(char* fmt, ...);


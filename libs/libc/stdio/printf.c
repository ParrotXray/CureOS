#include <libc/stdio.h>
#include <stdarg.h>

#include <system/tty/tty.h>

void printf(char* fmt, ...) {
    char buffer[1024];
    va_list args;
    va_start(args, fmt);
    __sprintf_internal(buffer, fmt, sizeof(buffer), args);
    va_end(args);

    // 輸出到終端
    tty_put_str(buffer);
}
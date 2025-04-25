#include <libc/string.h>
#include <system/tty/tty.h>
#include <system/constants.h>
#include <stdint.h>

#define TTY_WIDTH 80
#define TTY_HEIGHT 25

vga_atrributes* buffer = 0xB8000;
vga_atrributes* tty_vga_buffer = (vga_atrributes*)VGA_BUFFER_PADDR;
vga_atrributes theme_color = VGA_COLOR_BLACK;

uint32_t TTY_COLUMN = 0;    // tty_x
uint16_t TTY_ROW = 0;       // tty_y

void tty_init(void* vga_buf) {
    tty_vga_buffer = (vga_atrributes*)vga_buf;
    tty_clear();
}

void tty_set_buffer(void* vga_buf) {
    tty_vga_buffer = (vga_atrributes*)vga_buf;
}


void tty_set_theme(vga_atrributes fg, vga_atrributes bg) { 
    theme_color = (bg << 4 | fg) << 8;
}

void tty_put_char(char chr) {
    if (chr == '\n') {
        TTY_COLUMN = 0;
        ++TTY_ROW;
    } else if (chr == '\t') {
        TTY_COLUMN += 4;
    } else if (chr == '\r') {
        TTY_COLUMN = 0;
    } else {
        *(buffer + TTY_COLUMN + TTY_ROW * TTY_WIDTH) = (theme_color | chr);
        ++TTY_COLUMN;
        if (TTY_COLUMN >= TTY_WIDTH) {
            TTY_COLUMN = 0;
            ++TTY_ROW;
        }
    }

    if (TTY_COLUMN >= TTY_WIDTH) {
        TTY_COLUMN = 0;
        ++TTY_ROW;
    }
    
    if (TTY_ROW >= TTY_HEIGHT) {
        tty_scroll_up();
        --TTY_ROW;
    }

}

void tty_put_str(char* str) {
    while (*str != '\0') {
        tty_put_char(*str);
        ++str;
    }
}

void tty_scroll_up() {
    size_t last_line = TTY_WIDTH * (TTY_HEIGHT - 1);
    memcpy(tty_vga_buffer, tty_vga_buffer + TTY_WIDTH, last_line * 2);
    for (size_t i = 0; i < TTY_WIDTH; i++) {
        *(tty_vga_buffer + i + last_line) = theme_color;
    }
    TTY_ROW = TTY_ROW == 0 ? 0 : TTY_HEIGHT - 1;
}

void tty_clear() {
    for (uint32_t i = 0; i < TTY_WIDTH * TTY_HEIGHT; i++) {
        *(buffer + i) = theme_color;
    }
    TTY_COLUMN = 0;
    TTY_ROW = 0;
}
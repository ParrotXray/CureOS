#include <hal/io.h>

void io_port_wb(uint8_t port, uint8_t value) {
    __asm__ volatile (
        "movb %0, %%al\n"
        "movw %1, %%dx\n"
        "outb %%al, %%dx\n"
        :: "q"(value), "r"((uint16_t)port)
        : "al", "dx"
    );
}

void io_port_wl(uint8_t port, uint32_t value) {
    __asm__ volatile (
        "movl %0, %%eax\n"
        "movw %1, %%dx\n"
        "outl %%eax, %%dx\n"
        :: "r"(value), "r"((uint16_t)port)
        : "eax", "dx"
    );
}

uint8_t io_port_rb(uint8_t port) {
    uint8_t result;
    __asm__ volatile (
        "movw %1, %%dx\n"
        "inb %%dx, %%al\n"
        "movb %%al, %0\n"
        : "=q"(result)
        : "r"((uint16_t)port)
        : "al", "dx"
    );
    return result;
}

uint32_t io_port_rl(uint8_t port) {
    uint32_t result;
    __asm__ volatile (
        "movw %1, %%dx\n"
        "inl %%dx, %%eax\n"
        "movl %%eax, %0\n"
        : "=r"(result)
        : "r"((uint16_t)port)
        : "eax", "dx"
    );
    return result;
}
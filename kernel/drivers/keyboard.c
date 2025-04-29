#include <arch/x86/interrupts.h>
#include <hal/apic/apic.h>
#include <drivers/keyboard.h>
#include <hal/io.h>
#include <libc/stdio.h>

// 鍵盤控制器端口
#define KB_DATA_PORT    0x60
#define KB_STATUS_PORT  0x64
#define KB_COMMAND_PORT 0x64

// 鍵盤命令
#define KB_COMMAND_READ_CONFIG  0x20
#define KB_COMMAND_WRITE_CONFIG 0x60

// 鍵盤配置位
#define KB_CONFIG_INT_ENABLE    0x01
#define KB_CONFIG_SYSTEM_FLAG   0x04

// 按鍵狀態標誌
#define KEY_RELEASED      0x80
#define KEY_SPECIAL       0xE0
#define KEY_SPECIAL_RELEASED (KEY_SPECIAL | KEY_RELEASED)

// 特殊按鍵掃描碼
#define KEY_LSHIFT    0x2A
#define KEY_RSHIFT    0x36
#define KEY_LCTRL     0x1D
#define KEY_RCTRL     0x1D // 特殊前綴
#define KEY_LALT      0x38
#define KEY_RALT      0x38 // 特殊前綴
#define KEY_CAPS_LOCK 0x3A

// 按鍵修飾符狀態
static struct {
    uint8_t shift : 1;
    uint8_t ctrl : 1;
    uint8_t alt : 1;
    uint8_t caps_lock : 1;
    uint8_t num_lock : 1;
    uint8_t scroll_lock : 1;
    uint8_t special : 1; // 處理特殊鍵的前綴
} key_state = {0};

// 標準US布局鍵盤映射表
static const char keymap_us[128] = {
    0,  27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    0, 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    0, '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 0,
    '*', 0, ' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    '-', 0, 0, 0, '+', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

// Shift鍵映射表
static const char keymap_us_shift[128] = {
    0,  27, '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\b',
    '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n',
    0, 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~',
    0, '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?', 0,
    '*', 0, ' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    '-', 0, 0, 0, '+', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

// 鍵盤中斷處理程序
static void keyboard_handler(isr_param* param) {
    // 讀取鍵盤掃描碼
    uint8_t scancode = io_port_rb(KB_DATA_PORT);
    
    // 處理特殊鍵前綴
    if (scancode == KEY_SPECIAL) {
        key_state.special = 1;
        goto end;
    }
    
    // 檢查是否為按鍵釋放
    uint8_t released = scancode & KEY_RELEASED;
    scancode &= ~KEY_RELEASED;
    
    // 處理修飾鍵狀態
    if (key_state.special) {
        // 處理帶特殊前綴的按鍵
        key_state.special = 0;
        
        if (scancode == KEY_RCTRL) {
            key_state.ctrl = !released;
        } else if (scancode == KEY_RALT) {
            key_state.alt = !released;
        }
        
        goto end;
    }
    
    // 處理一般修飾鍵
    if (scancode == KEY_LSHIFT || scancode == KEY_RSHIFT) {
        key_state.shift = !released;
    } else if (scancode == KEY_LCTRL) {
        key_state.ctrl = !released;
    } else if (scancode == KEY_LALT) {
        key_state.alt = !released;
    } else if (scancode == KEY_CAPS_LOCK && !released) {
        // 切換Caps Lock狀態
        key_state.caps_lock = !key_state.caps_lock;
        printf("[Keyboard] Caps Lock: %s\n", key_state.caps_lock ? "ON" : "OFF");
    } else if (!released && scancode < 128) {
        // 處理一般按鍵
        char c;
        
        // 根據修飾鍵狀態選擇映射表
        if (key_state.shift) {
            c = keymap_us_shift[scancode];
        } else {
            c = keymap_us[scancode];
        }
        
        // 處理Caps Lock對字母的影響
        if (key_state.caps_lock && c >= 'a' && c <= 'z') {
            c = c - 'a' + 'A';
        } else if (key_state.caps_lock && c >= 'A' && c <= 'Z') {
            c = c - 'A' + 'a';
        }
        
        // 控制鍵組合
        if (key_state.ctrl) {
            if (c >= 'a' && c <= 'z') {
                c = c - 'a' + 1; // Ctrl+A -> 1, Ctrl+B -> 2, etc.
            } else if (c >= 'A' && c <= 'Z') {
                c = c - 'A' + 1;
            }
        }
        
        // 在控制台顯示按鍵
        if (c) {
            printf("%c", c);
        }
    }
    
end:
    // 發送EOI
    apic_send_eoi();
}

// 初始化鍵盤
void keyboard_init() {
    printf("[Keyboard] Initializing keyboard\n");
    // 啟用特定IRQ
    // 鍵盤IRQ
    enable_irq(1);
    
    // 讀取鍵盤控制器配置
    io_port_wb(KB_COMMAND_PORT, KB_COMMAND_READ_CONFIG);
    uint8_t config = io_port_rb(KB_DATA_PORT);
    
    // 啟用鍵盤中斷
    config |= KB_CONFIG_INT_ENABLE;
    
    // 寫回配置
    io_port_wb(KB_COMMAND_PORT, KB_COMMAND_WRITE_CONFIG);
    io_port_wb(KB_DATA_PORT, config);
    
    // 清空輸入緩衝區
    while (io_port_rb(KB_STATUS_PORT) & 1) {
        io_port_rb(KB_DATA_PORT);
    }
    
    // 註冊中斷處理程序
    register_irq_handler(1, keyboard_handler);
    
    printf("[Keyboard] Keyboard initialized\n");
}
// src/kernel/tty/tty.rs

use crate::hal::io;
use core::ptr::NonNull;

/// VGA 屬性類型 (16位)
#[allow(dead_code)]
pub type VgaAttribute = u16;

// VGA 顏色常量
#[allow(dead_code)]
pub const VGA_COLOR_BLACK: u8 = 0;
#[allow(dead_code)]
pub const VGA_COLOR_BLUE: u8 = 1;
#[allow(dead_code)]
pub const VGA_COLOR_GREEN: u8 = 2;
#[allow(dead_code)]
pub const VGA_COLOR_CYAN: u8 = 3;
#[allow(dead_code)]
pub const VGA_COLOR_RED: u8 = 4;
#[allow(dead_code)]
pub const VGA_COLOR_MAGENTA: u8 = 5;
#[allow(dead_code)]
pub const VGA_COLOR_BROWN: u8 = 6;
#[allow(dead_code)]
pub const VGA_COLOR_LIGHT_GREY: u8 = 7;
#[allow(dead_code)]
pub const VGA_COLOR_DARK_GREY: u8 = 8;
#[allow(dead_code)]
pub const VGA_COLOR_LIGHT_BLUE: u8 = 9;
#[allow(dead_code)]
pub const VGA_COLOR_LIGHT_GREEN: u8 = 10;
#[allow(dead_code)]
pub const VGA_COLOR_LIGHT_CYAN: u8 = 11;
#[allow(dead_code)]
pub const VGA_COLOR_LIGHT_RED: u8 = 12;
#[allow(dead_code)]
pub const VGA_COLOR_LIGHT_MAGENTA: u8 = 13;
#[allow(dead_code)]
pub const VGA_COLOR_LIGHT_BROWN: u8 = 14;
#[allow(dead_code)]
pub const VGA_COLOR_WHITE: u8 = 15;
#[allow(dead_code)]
pub const VGA_BUFFER_PADDR: usize = 0xB8000;

#[allow(dead_code)]
const TTY_WIDTH: usize = 80;
#[allow(dead_code)]
const TTY_HEIGHT: usize = 25;
#[allow(dead_code)]
const VGA_CTRL_PORT: u16 = 0x3D4;
#[allow(dead_code)]
const VGA_DATA_PORT: u16 = 0x3D5;

// 全局 TTY 狀態
pub struct TTYState {
    vga_buffer: Option<NonNull<VgaAttribute>>,
    theme_color: VgaAttribute,
    x: usize,
    y: usize,
}

impl TTYState {
    const fn new() -> Self {
        Self {
            vga_buffer: None,
            theme_color: (VGA_COLOR_BLACK as VgaAttribute) << 8,
            x: 0,
            y: 0,
        }
    }
}

static mut TTY_STATE: TTYState = TTYState::new();

// #[no_mangle]
// fn update_cursor() {
//     unsafe {
//         let pos = TTY_STATE.y * TTY_WIDTH + TTY_STATE.x;
        
//         io::io_port_wb(VGA_CTRL_PORT, 0x0F);
//         io::io_port_wb(VGA_DATA_PORT, (pos & 0xFF) as u8);
//         io::io_port_wb(VGA_CTRL_PORT, 0x0E);
//         io::io_port_wb(VGA_DATA_PORT, ((pos >> 8) & 0xFF) as u8);
//     }
// }

/// 初始化 TTY
// vga_buf: *mut u8
#[no_mangle]
pub fn tty_init(vga_buf: usize) {
    unsafe {
        TTY_STATE.vga_buffer = NonNull::new(vga_buf as *mut VgaAttribute);
        tty_clear();
    }
}

/// 設置 VGA 緩衝區
#[no_mangle]
pub fn tty_set_buffer(vga_buf: usize) {
    unsafe {
        TTY_STATE.vga_buffer = NonNull::new(vga_buf as *mut VgaAttribute);
    }
}

/// 設置主題顏色（前景色和背景色）
#[no_mangle]
pub fn tty_set_theme(fg: u8, bg: u8) {
    unsafe {
        TTY_STATE.theme_color = ((bg << 4 | fg) as VgaAttribute) << 8;
    }
}

/// 向 TTY 輸出單個字符
/// 
/// # 注意
/// - 僅支援 ASCII 範圍 (0-255) 的字符
/// - 多字節 Unicode 字符會被截斷為低 8 位
/// - '\t' 擴展為 4 個空格
/// - '\n' 換行並將游標移至行首
/// - '\r' 將游標移至行首
#[no_mangle]
pub fn tty_put_char(chr: char) {
    unsafe {
        if let Some(vga_ptr) = TTY_STATE.vga_buffer {
            match chr {
                '\t' => {
                    TTY_STATE.x += 4;
                }
                '\n' => {
                    TTY_STATE.y += 1;
                    TTY_STATE.x = 0;
                }
                '\r' => {
                    TTY_STATE.x = 0;
                }
                _ => {
                    let offset = TTY_STATE.x + TTY_STATE.y * TTY_WIDTH;
                    // VGA 文本模式僅支持 8 位字符
                    *vga_ptr.as_ptr().add(offset) = TTY_STATE.theme_color | (chr as u8) as VgaAttribute;
                    TTY_STATE.x += 1;
                }
            }

            if TTY_STATE.x >= TTY_WIDTH {
                TTY_STATE.x = 0;
                TTY_STATE.y += 1;
            }
            
            if TTY_STATE.y >= TTY_HEIGHT {
                tty_scroll_up();
            }

            // update_cursor();
        }
    }
}

/// 向 TTY 輸出字符串
/// 
/// # 注意
/// - 字符串中的多字節 Unicode 字符會被截斷為低 8 位
/// - 支持 Rust 字符串切片 (&str)
#[no_mangle]
pub fn tty_put_str(s: &str) {
    for chr in s.chars() {
        tty_put_char(chr);
    }
}

/// 向上滾動一行
#[no_mangle]
pub fn tty_scroll_up() {
    unsafe {
        if let Some(vga_ptr) = TTY_STATE.vga_buffer {
            let last_line = TTY_WIDTH * (TTY_HEIGHT - 1);
            let buffer_ptr = vga_ptr.as_ptr();
            
            // 將所有行向上移動一行
            core::ptr::copy(
                buffer_ptr.add(TTY_WIDTH),
                buffer_ptr,
                last_line
            );
            
            // 清空最後一行
            for i in 0..TTY_WIDTH {
                *buffer_ptr.add(i + last_line) = TTY_STATE.theme_color;
            }
            
            TTY_STATE.y = if TTY_STATE.y == 0 { 0 } else { TTY_HEIGHT - 1 };
        }
    }
}

/// 清空屏幕
#[no_mangle]
pub fn tty_clear() {
    unsafe {
        if let Some(vga_ptr) = TTY_STATE.vga_buffer {
            let buffer_ptr = vga_ptr.as_ptr();

            for i in 0..(TTY_WIDTH * TTY_HEIGHT) {
                *buffer_ptr.add(i) = TTY_STATE.theme_color;
            }

            TTY_STATE.x = 0;
            TTY_STATE.y = 0;
            // update_cursor();
        }
    }
}

/// 清空指定行
#[no_mangle]
pub fn tty_clear_line(y: usize) {
    unsafe {
        if let Some(vga_ptr) = TTY_STATE.vga_buffer {
            let buffer_ptr = vga_ptr.as_ptr();

            for i in 0..TTY_WIDTH {
                *buffer_ptr.add(i + y * TTY_WIDTH) = TTY_STATE.theme_color;
            }
        }
    }
}

/// 設置游標位置
#[no_mangle]
pub fn tty_set_cpos(x: usize, y: usize) {
    unsafe {
        TTY_STATE.x = x % TTY_WIDTH;
        TTY_STATE.y = y % TTY_HEIGHT;
        // update_cursor();
    }
}

/// 獲取游標位置
#[no_mangle]
pub fn tty_get_cpos() -> (usize, usize) {
    unsafe {
        (TTY_STATE.x, TTY_STATE.y)
    }
}

/// 獲取當前主題顏色
#[no_mangle]
pub fn tty_get_theme() -> VgaAttribute {
    unsafe {
        TTY_STATE.theme_color
    }
}

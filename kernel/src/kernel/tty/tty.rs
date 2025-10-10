use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use crate::kernel::tty::font::FONT_BASIC;
use core::ptr::{addr_of, addr_of_mut};

pub struct TTYState {
    framebuffer: *mut u8,
    fb_len: usize,
    info: Option<FrameBufferInfo>,
    cursor_x: usize,
    cursor_y: usize,
}

impl TTYState {
    const fn new() -> Self {
        Self {
            framebuffer: core::ptr::null_mut(),
            fb_len: 0,
            info: None,
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}

static mut TTY_STATE: TTYState = TTYState::new();

pub fn init(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    unsafe {
        let state = &mut *addr_of_mut!(TTY_STATE);
        state.framebuffer = buffer.as_mut_ptr();
        state.fb_len = buffer.len();
        state.info = Some(info);
        state.cursor_x = 0;
        state.cursor_y = 0;
    }

    clear(0x000000);
}

pub fn clear(color: u32) {
    unsafe {
        let state = &*addr_of!(TTY_STATE);

        if state.framebuffer.is_null() {
            return;
        }

        let info = match state.info.as_ref() {
            Some(i) => i,
            None => return,
        };

        let (r, g, b) = u32_to_rgb(color);
        let buffer = core::slice::from_raw_parts_mut(state.framebuffer, state.fb_len);

        for pixel in buffer.chunks_exact_mut(info.bytes_per_pixel) {
            write_pixel(info, pixel, r, g, b);
        }

        let state_mut = &mut *addr_of_mut!(TTY_STATE);
        state_mut.cursor_x = 0;
        state_mut.cursor_y = 0;
    }
}

fn write_pixel(info: &FrameBufferInfo, pixel: &mut [u8], r: u8, g: u8, b: u8) {
    match info.pixel_format {
        PixelFormat::Rgb => {
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
        }
        PixelFormat::Bgr => {
            pixel[0] = b;
            pixel[1] = g;
            pixel[2] = r;
        }
        _ => {}
    }
}

pub fn draw_pixel(x: usize, y: usize, color: u32) {
    unsafe {
        let state = &*addr_of!(TTY_STATE);

        if state.framebuffer.is_null() {
            return;
        }

        let info = match state.info.as_ref() {
            Some(i) => i,
            None => return,
        };

        if x >= info.width || y >= info.height {
            return;
        }

        let pixel_offset = (y * info.stride + x) * info.bytes_per_pixel;
        let (r, g, b) = u32_to_rgb(color);

        let buffer = core::slice::from_raw_parts_mut(state.framebuffer, state.fb_len);

        if let Some(pixel) = buffer.get_mut(pixel_offset..pixel_offset + info.bytes_per_pixel) {
            write_pixel(info, pixel, r, g, b);
        }
    }
}

pub fn draw_char(c: char, color: u32) {
    const CHAR_WIDTH: usize = 8;
    const CHAR_HEIGHT: usize = 16;

    unsafe {
        let state = &*addr_of!(TTY_STATE);

        if state.framebuffer.is_null() {
            return;
        }

        let info = match state.info.as_ref() {
            Some(i) => i,
            None => return,
        };

        let state_mut = &mut *addr_of_mut!(TTY_STATE);

        if c == '\n' {
            state_mut.cursor_x = 0;
            state_mut.cursor_y += CHAR_HEIGHT;
            return;
        }

        if state_mut.cursor_x + CHAR_WIDTH > info.width {
            state_mut.cursor_x = 0;
            state_mut.cursor_y += CHAR_HEIGHT;
        }

        if state_mut.cursor_y + CHAR_HEIGHT > info.height {
            state_mut.cursor_y = 0;
        }

        let idx = c as usize;
        if idx >= FONT_BASIC.len() {
            return;
        }
        let glyph = &FONT_BASIC[idx];

        let cursor_x = state_mut.cursor_x;
        let cursor_y = state_mut.cursor_y;

        for (y, row) in glyph.iter().enumerate() {
            for x in 0..8 {
                if (row >> (7 - x)) & 1 == 1 {
                    draw_pixel(cursor_x + x, cursor_y + y, color);
                }
            }
        }

        state_mut.cursor_x += CHAR_WIDTH;
    }
}

pub fn write_str(s: &str, color: u32) {
    for c in s.chars() {
        draw_char(c, color);
    }
}

pub fn tty_put_str(s: &str, color: Option<u32>) {
    if let Some(c) = color {
        write_str(s, c);
    } else {
        write_str(s, 0xFFFFFF);
    }
}

fn u32_to_rgb(color: u32) -> (u8, u8, u8) {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    (r, g, b)
}
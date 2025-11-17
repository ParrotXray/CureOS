use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use x86_64::instructions::interrupts;
use crate::tty::font::FONT_BASIC;
use spin::Mutex;

pub struct TTYState {
    framebuffer: *mut u8,
    fb_len: usize,
    info: FrameBufferInfo,
    cursor_x: usize,
    cursor_y: usize,
}

unsafe impl Send for TTYState {}

impl TTYState {
    const fn new_empty() -> Self {
        Self {
            framebuffer: core::ptr::null_mut(),
            fb_len: 0,
            info: FrameBufferInfo {
                byte_len: 0,
                width: 0,
                height: 0,
                pixel_format: PixelFormat::Rgb,
                bytes_per_pixel: 0,
                stride: 0,
            },
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    fn init(&mut self, framebuffer: *mut u8, fb_len: usize, info: FrameBufferInfo) {
        self.framebuffer = framebuffer;
        self.fb_len = fb_len;
        self.info = info;
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn is_initialized(&self) -> bool {
        !self.framebuffer.is_null()
    }

    fn clear(&mut self, color: u32) {
        if !self.is_initialized() {
            return;
        }

        let (r, g, b) = u32_to_rgb(color);
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(self.framebuffer, self.fb_len)
        };

        for pixel in buffer.chunks_exact_mut(self.info.bytes_per_pixel) {
            write_pixel(&self.info, pixel, r, g, b);
        }

        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if !self.is_initialized() {
            return;
        }

        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let pixel_offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        let (r, g, b) = u32_to_rgb(color);

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(self.framebuffer, self.fb_len)
        };

        if let Some(pixel) = buffer.get_mut(pixel_offset..pixel_offset + self.info.bytes_per_pixel) {
            write_pixel(&self.info, pixel, r, g, b);
        }
    }

    fn scroll_up(&mut self) {
        const CHAR_HEIGHT: usize = 16;

        if !self.is_initialized() {
            return;
        }

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(self.framebuffer, self.fb_len)
        };

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let stride = self.info.stride;
        let width = self.info.width;
        let height = self.info.height;

        for y in CHAR_HEIGHT..height {
            let src_offset = y * stride * bytes_per_pixel;
            let dst_offset = (y - CHAR_HEIGHT) * stride * bytes_per_pixel;
            let row_size = width * bytes_per_pixel;

            if src_offset + row_size <= buffer.len() && dst_offset + row_size <= buffer.len() {
                buffer.copy_within(src_offset..src_offset + row_size, dst_offset);
            }
        }

        for y in (height.saturating_sub(CHAR_HEIGHT))..height {
            for x in 0..width {
                let pixel_offset = (y * stride + x) * bytes_per_pixel;
                if let Some(pixel) = buffer.get_mut(pixel_offset..pixel_offset + bytes_per_pixel) {
                    write_pixel(&self.info, pixel, 0, 0, 0);
                }
            }
        }

        self.cursor_x = 0;
        self.cursor_y = height.saturating_sub(CHAR_HEIGHT);
    }

    fn draw_char(&mut self, c: char, color: u32) {
        const CHAR_WIDTH: usize = 8;
        const CHAR_HEIGHT: usize = 16;

        if !self.is_initialized() {
            return;
        }

        if c == '\x08' {
            if self.cursor_x >= CHAR_WIDTH {
                self.cursor_x -= CHAR_WIDTH;

                for y in 0..CHAR_HEIGHT {
                    for x in 0..CHAR_WIDTH {
                        self.draw_pixel(self.cursor_x + x, self.cursor_y + y, 0x000000);
                    }
                }
            } else if self.cursor_y >= CHAR_HEIGHT {
                self.cursor_y -= CHAR_HEIGHT;
                self.cursor_x = (self.info.width / CHAR_WIDTH - 1) * CHAR_WIDTH;

                for y in 0..CHAR_HEIGHT {
                    for x in 0..CHAR_WIDTH {
                        self.draw_pixel(self.cursor_x + x, self.cursor_y + y, 0x000000);
                    }
                }
            }
            return;
        }

        if c == '\n' {
            self.cursor_x = 0;
            self.cursor_y += CHAR_HEIGHT;

            if self.cursor_y + CHAR_HEIGHT > self.info.height {
                self.scroll_up();
            }
            return;
        }

        if self.cursor_x + CHAR_WIDTH > self.info.width {
            self.cursor_x = 0;
            self.cursor_y += CHAR_HEIGHT;
        }

        if self.cursor_y + CHAR_HEIGHT > self.info.height {
            self.scroll_up();
        }

        let idx = c as usize;
        if idx >= FONT_BASIC.len() {
            return;
        }
        let glyph = &FONT_BASIC[idx];

        let cursor_x = self.cursor_x;
        let cursor_y = self.cursor_y;

        for (y, row) in glyph.iter().enumerate() {
            for x in 0..8 {
                if (row >> (7 - x)) & 1 == 1 {
                    self.draw_pixel(cursor_x + x, cursor_y + y, color);
                } else {
                    // 同時清除背景
                    self.draw_pixel(cursor_x + x, cursor_y + y, 0x000000);
                }
            }
        }

        self.cursor_x += CHAR_WIDTH;
    }

    fn write_str(&mut self, s: &str, color: u32) {
        for c in s.chars() {
            self.draw_char(c, color);
        }
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn set_cursor_pos(&mut self, x: usize, y: usize) {
        self.cursor_x = x;
        self.cursor_y = y;
    }
}

static TTY: Mutex<TTYState> = Mutex::new(TTYState::new_empty());

pub fn init(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    interrupts::without_interrupts(|| {
        TTY.lock().init(buffer.as_mut_ptr(), buffer.len(), info);
    });
}

pub fn clear(color: u32) {
    interrupts::without_interrupts(|| {
        TTY.lock().clear(color);
    });
}

pub fn draw_pixel(x: usize, y: usize, color: u32) {
    interrupts::without_interrupts(|| {
        TTY.lock().draw_pixel(x, y, color);
    });
}

pub fn draw_char(c: char, color: u32) {
    interrupts::without_interrupts(|| {
        TTY.lock().draw_char(c, color);
    });
}

pub fn write_str(s: &str, color: u32) {
    interrupts::without_interrupts(|| {
        TTY.lock().write_str(s, color);
    });
}

pub fn tty_put_str(s: &str, color: Option<u32>) {
    let c = color.unwrap_or(0xFFFFFF);
    write_str(s, c);
}

pub fn get_cursor_pos() -> (usize, usize) {
    interrupts::without_interrupts(|| {
        TTY.lock().get_cursor_pos()
    })
}

pub fn set_cursor_pos(x: usize, y: usize) {
    interrupts::without_interrupts(|| {
        TTY.lock().set_cursor_pos(x, y);
    });
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

fn u32_to_rgb(color: u32) -> (u8, u8, u8) {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    (r, g, b)
}
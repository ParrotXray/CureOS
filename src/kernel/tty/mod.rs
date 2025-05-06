// src/kernel/tty/mod.rs

pub mod tty;

pub use tty::tty_init;
pub use tty::tty_set_buffer;
pub use tty::tty_set_theme;
pub use tty::tty_put_char;
pub use tty::tty_put_str;
pub use tty::tty_scroll_up;
pub use tty::tty_clear;
pub use tty::tty_clear_line;
pub use tty::tty_set_cpos;
pub use tty::tty_get_cpos;
pub use tty::tty_get_theme;
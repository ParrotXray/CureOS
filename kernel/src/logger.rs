// kernel/src/logger.rs
use core::fmt;
use crate::kernel::tty::tty;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub const fn color(&self) -> u32 {
        match self {
            LogLevel::Trace => 0x808080,  // 灰色
            LogLevel::Debug => 0x00FFFF,  // 青色
            LogLevel::Info => 0x00FF00,   // 綠色
            LogLevel::Warn => 0xFFFF00,   // 黃色
            LogLevel::Error => 0xFF6600,  // 橙色
            LogLevel::Fatal => 0xFF0000,  // 紅色
        }
    }

    /// 獲取日誌級別標籤
    pub const fn tag(&self) -> &'static str {
        match self {
            LogLevel::Trace => "[TRACE]",
            LogLevel::Debug => "[DEBUG]",
            LogLevel::Info => "[INFO] ",
            LogLevel::Warn => "[WARN] ",
            LogLevel::Error => "[ERROR]",
            LogLevel::Fatal => "[FATAL]",
        }
    }
}

pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub show_timestamps: bool,
    pub show_location: bool,
}

impl LoggerConfig {
    pub const fn new() -> Self {
        Self {
            min_level: LogLevel::Trace,
            show_timestamps: false,
            show_location: false,
        }
    }

    pub const fn with_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    pub const fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    pub const fn with_location(mut self, show: bool) -> Self {
        self.show_location = show;
        self
    }
}

struct LoggerState {
    config: LoggerConfig,
}

impl LoggerState {
    const fn new() -> Self {
        Self {
            config: LoggerConfig::new(),
        }
    }

    fn set_config(&mut self, config: LoggerConfig) {
        self.config = config;
    }

    fn log(&self, level: LogLevel, args: fmt::Arguments, file: Option<&str>, line: Option<u32>) {
        if level < self.config.min_level {
            return;
        }

        tty::write_str(level.tag(), level.color());
        tty::write_str(" ", 0xFFFFFF);

        // 如果啟用位置信息
        if self.config.show_location {
            if let (Some(f), Some(l)) = (file, line) {
                let mut location_buf = [0u8; 128];
                if let Ok(location_str) = fmt_to_buffer(&mut location_buf, format_args!("[{}:{}] ", f, l)) {
                    tty::write_str(location_str, 0x888888);
                }
            }
        }

        let mut msg_buf = [0u8; 512];
        if let Ok(msg) = fmt_to_buffer(&mut msg_buf, args) {
            tty::write_str(msg, 0xFFFFFF);
        }

        tty::write_str("\n", 0xFFFFFF);
    }
}

static LOGGER: Mutex<LoggerState> = Mutex::new(LoggerState::new());

pub fn init(config: LoggerConfig) {
    LOGGER.lock().set_config(config);
}

pub fn set_level(level: LogLevel) {
    LOGGER.lock().config.min_level = level;
}

pub fn _log(level: LogLevel, args: fmt::Arguments, file: Option<&str>, line: Option<u32>) {
    LOGGER.lock().log(level, args, file, line);
}

fn fmt_to_buffer<'a>(buf: &'a mut [u8], args: fmt::Arguments) -> Result<&'a str, ()> {
    use core::fmt::Write;

    struct BufWriter<'a> {
        buf: &'a mut [u8],
        pos: usize,
    }

    impl<'a> Write for BufWriter<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let remaining = self.buf.len() - self.pos;
            let to_write = core::cmp::min(bytes.len(), remaining);

            if to_write > 0 {
                self.buf[self.pos..self.pos + to_write].copy_from_slice(&bytes[..to_write]);
                self.pos += to_write;
            }

            Ok(())
        }
    }

    let mut writer = BufWriter { buf, pos: 0 };
    writer.write_fmt(args).map_err(|_| ())?;

    core::str::from_utf8(&writer.buf[..writer.pos]).map_err(|_| ())
}

/// Trace
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::logger::_log(
            $crate::logger::LogLevel::Trace,
            format_args!($($arg)*),
            Some(file!()),
            Some(line!())
        )
    };
}

/// Debug
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::_log(
            $crate::logger::LogLevel::Debug,
            format_args!($($arg)*),
            Some(file!()),
            Some(line!())
        )
    };
}

/// Info
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::_log(
            $crate::logger::LogLevel::Info,
            format_args!($($arg)*),
            Some(file!()),
            Some(line!())
        )
    };
}

/// Warn
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::_log(
            $crate::logger::LogLevel::Warn,
            format_args!($($arg)*),
            Some(file!()),
            Some(line!())
        )
    };
}

/// Error
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::_log(
            $crate::logger::LogLevel::Error,
            format_args!($($arg)*),
            Some(file!()),
            Some(line!())
        )
    };
}

/// Fatal
#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        $crate::logger::_log(
            $crate::logger::LogLevel::Fatal,
            format_args!($($arg)*),
            Some(file!()),
            Some(line!())
        )
    };
}
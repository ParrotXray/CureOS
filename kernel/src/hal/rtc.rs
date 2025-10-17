// kernel/src/hal/rtc.rs
use crate::hal::io::{io_port_rb, io_port_wb};
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use crate::log_info;

const RTC_INDEX_PORT: u16 = 0x70;
const RTC_TARGET_PORT: u16 = 0x71;
const WITH_NMI_DISABLED: u8 = 0x80;

const RTC_REG_SEC: u8 = 0x00;
const RTC_REG_MIN: u8 = 0x02;
const RTC_REG_HRS: u8 = 0x04;
const RTC_REG_WDY: u8 = 0x06;  // Weekday
const RTC_REG_DAY: u8 = 0x07;
const RTC_REG_MTH: u8 = 0x08;
const RTC_REG_YRS: u8 = 0x09;

const RTC_REG_A: u8 = 0x0A;
const RTC_REG_B: u8 = 0x0B;
const RTC_REG_C: u8 = 0x0C;
const RTC_REG_D: u8 = 0x0D;

const RTC_UPDATE_IN_PROGRESS: u8 = 0x80;  // Status Register A bit 7
const RTC_BIN_ENCODED_BIT: u8 = 0x04;     // Status Register B bit 2
const RTC_24HRS_ENCODED_BIT: u8 = 0x02;   // Status Register B bit 1

const RTC_TIMER_ON: u8 = 0x40;            // Enable periodic interrupt (bit 6)
const RTC_FREQUENCY_1024HZ: u8 = 0b110;   // Rate selector for 1024Hz
const RTC_DIVIDER_33KHZ: u8 = 0b010 << 4; // 32.768kHz crystal divider
const RTC_TIMER_BASE_FREQUENCY: u32 = 1024;

const RTC_CURRENT_CENTURY: u16 = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub weekday: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    pub fn format(&self) -> alloc::string::String {
        alloc::format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day,
            self.hour, self.minute, self.second
        )
    }

    pub fn weekday_name(&self) -> &'static str {
        match self.weekday {
            1 => "Sunday",
            2 => "Monday",
            3 => "Tuesday",
            4 => "Wednesday",
            5 => "Thursday",
            6 => "Friday",
            7 => "Saturday",
            _ => "Unknown",
        }
    }

    pub fn month_name(&self) -> &'static str {
        match self.month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }
}

pub struct Rtc {
    binary_mode: bool,
    hour_24_mode: bool,
}

impl Rtc {
    pub fn new() -> Self {
        Self {
            binary_mode: false,
            hour_24_mode: false,
        }
    }

    /// Read CMOS registers (with NMI disabled) - 公開的靜態方法
    pub unsafe fn read_register(reg: u8) -> u8 {
        io_port_wb(RTC_INDEX_PORT, reg | WITH_NMI_DISABLED);
        io_port_rb(RTC_TARGET_PORT)
    }

    /// Write to CMOS register (with NMI disabled) - 公開的靜態方法
    pub unsafe fn write_register(reg: u8, value: u8) {
        io_port_wb(RTC_INDEX_PORT, reg | WITH_NMI_DISABLED);
        io_port_wb(RTC_TARGET_PORT, value);
    }

    /// Check if RTC is updating
    unsafe fn is_updating() -> bool {
        (Self::read_register(RTC_REG_A) & RTC_UPDATE_IN_PROGRESS) != 0
    }

    /// Wait for RTC update to complete (with timeout)
    unsafe fn wait_for_update() -> bool {
        const MAX_ATTEMPTS: u32 = 100000;
        let mut attempts = 0;

        while Self::is_updating() && attempts < MAX_ATTEMPTS {
            core::hint::spin_loop();
            attempts += 1;
        }

        if attempts >= MAX_ATTEMPTS {
            crate::log_warn!("RTC wait_for_update timeout");
            Self::read_register(RTC_REG_C); // Force clear
            return false;
        }

        true
    }

    /// Convert BCD to binary
    fn bcd_to_binary(bcd: u8) -> u8 {
        (bcd & 0x0F) + ((bcd >> 4) * 10)
    }

    pub fn init(&mut self) {
        unsafe {
            let status_b = Self::read_register(RTC_REG_B | WITH_NMI_DISABLED);
            self.binary_mode = (status_b & RTC_BIN_ENCODED_BIT) != 0;
            self.hour_24_mode = (status_b & RTC_24HRS_ENCODED_BIT) != 0;

            let mut reg_a = Self::read_register(RTC_REG_A | WITH_NMI_DISABLED);
            reg_a = (reg_a & 0xF0) | RTC_DIVIDER_33KHZ | RTC_FREQUENCY_1024HZ;
            Self::write_register(RTC_REG_A | WITH_NMI_DISABLED, reg_a);

            Self::read_register(RTC_REG_C);

            self.disable_timer();

            Self::read_register(RTC_REG_C);
        }
    }

    /// Read raw data directly (without waiting for update)
    unsafe fn read_raw_no_wait(&self) -> (u8, u8, u8, u8, u8, u8, u8) {
        let second = Self::read_register(RTC_REG_SEC);
        let minute = Self::read_register(RTC_REG_MIN);
        let hour = Self::read_register(RTC_REG_HRS);
        let day = Self::read_register(RTC_REG_DAY);
        let month = Self::read_register(RTC_REG_MTH);
        let year = Self::read_register(RTC_REG_YRS);
        let weekday = Self::read_register(RTC_REG_WDY);

        (second, minute, hour, day, month, year, weekday)
    }

    /// Wait for update and then read (for initialization)
    unsafe fn read_raw(&self) -> (u8, u8, u8, u8, u8, u8, u8) {
        Self::wait_for_update();
        self.read_raw_no_wait()
    }

    /// Convert the value based on encoding mode
    fn convert_value(&self, value: u8) -> u8 {
        if self.binary_mode {
            value
        } else {
            Self::bcd_to_binary(value)
        }
    }

    /// Read the RTC time (safe version - no wait during interrupts)
    pub fn read_time(&self) -> DateTime {
        unsafe {
            let (mut second, mut minute, mut hour, mut day, mut month, mut year, weekday) =
                self.read_raw_no_wait();

            // Convert from BCD to binary when needed
            second = self.convert_value(second);
            minute = self.convert_value(minute);
            day = self.convert_value(day);
            month = self.convert_value(month);
            year = self.convert_value(year);

            // Handle 12-hour format
            let pm_bit = hour & 0x80;
            hour = self.convert_value(hour & 0x7F);

            if !self.hour_24_mode && pm_bit != 0 {
                hour = (hour + 12) % 24;
            }

            let full_year = RTC_CURRENT_CENTURY + year as u16;

            DateTime {
                year: full_year,
                month,
                day,
                weekday,
                hour,
                minute,
                second,
            }
        }
    }

    /// Read time with retry (for initialization)
    pub fn read_time_stable(&self) -> DateTime {
        unsafe {
            loop {
                Self::wait_for_update();
                let time1 = self.read_time();
                let time2 = self.read_time();

                if time1.second == time2.second
                    && time1.minute == time2.minute
                    && time1.hour == time2.hour
                {
                    return time1;
                }
            }
        }
    }

    /// Enable RTC timer interrupt (1024Hz)
    pub fn enable_timer(&self) {
        unsafe {
            // 步驟 1: 先確保關閉
            self.disable_timer();
            Self::read_register(RTC_REG_C);

            // 步驟 2: 設置頻率
            let mut reg_a = Self::read_register(RTC_REG_A | WITH_NMI_DISABLED);
            reg_a = (reg_a & 0xF0) | RTC_DIVIDER_33KHZ | RTC_FREQUENCY_1024HZ;
            Self::write_register(RTC_REG_A | WITH_NMI_DISABLED, reg_a);

            // 步驟 3: 啟用週期性中斷
            let mut reg_b = Self::read_register(RTC_REG_B | WITH_NMI_DISABLED);
            reg_b |= RTC_TIMER_ON;
            Self::write_register(RTC_REG_B | WITH_NMI_DISABLED, reg_b);

            // 步驟 4: 清除中斷標誌
            Self::read_register(RTC_REG_C);

            log_info!("RTC timer enabled at {}Hz", RTC_TIMER_BASE_FREQUENCY);
        }
    }

    /// Disable RTC timer interrupt
    pub fn disable_timer(&self) {
        unsafe {
            let mut reg_b = Self::read_register(RTC_REG_B | WITH_NMI_DISABLED);
            reg_b &= !RTC_TIMER_ON;
            Self::write_register(RTC_REG_B | WITH_NMI_DISABLED, reg_b);

            Self::read_register(RTC_REG_C);
        }
    }
}

static RTC_DEVICE: Mutex<Option<Rtc>> = Mutex::new(None);

static RTC_TIME_CACHE: Mutex<Option<DateTime>> = Mutex::new(None);

static RTC_TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// Initialize RTC
pub fn init() {
    let mut rtc = Rtc::new();
    rtc.init();

    let initial_time = rtc.read_time_stable();

    *RTC_DEVICE.lock() = Some(rtc);
    *RTC_TIME_CACHE.lock() = Some(initial_time);

    log_info!("RTC initialized");
}

/// Get cached time (safe, no deadlock)
pub fn get_time() -> Option<DateTime> {
    RTC_TIME_CACHE.lock().clone()
}

/// Update time cache (call periodically in main loop, NOT in interrupt)
pub fn update_time_cache() {
    if let Some(rtc) = RTC_DEVICE.lock().as_ref() {
        let time = rtc.read_time();
        *RTC_TIME_CACHE.lock() = Some(time);
    }
}

/// Force read time from RTC (slow, use sparingly)
pub fn force_read_time() -> Option<DateTime> {
    let rtc = RTC_DEVICE.lock();
    let rtc = rtc.as_ref()?;
    let time = rtc.read_time();
    // Update cache
    *RTC_TIME_CACHE.lock() = Some(time);
    Some(time)
}

/// Print current time info
pub fn print_info() {
    if let Some(time) = get_time() {
        log_info!(
            "{}, {} {}, {} - {:02}:{:02}:{:02}",
            time.weekday_name(),
            time.month_name(),
            time.day,
            time.year,
            time.hour,
            time.minute,
            time.second
        );
    } else {
        crate::log_warn!("RTC not initialized");
    }
}

/// Enable RTC timer
pub fn enable_timer() {
    if let Some(rtc) = RTC_DEVICE.lock().as_ref() {
        rtc.enable_timer();
    }
}

/// Disable the RTC timer
pub fn disable_timer() {
    if let Some(rtc) = RTC_DEVICE.lock().as_ref() {
        rtc.disable_timer();
    }
}

/// Get RTC tick count (lock-free, safe in interrupts)
pub fn get_tick_count() -> u64 {
    RTC_TICK_COUNT.load(Ordering::Relaxed)
}

/// Reset tick count (for testing)
pub fn reset_tick_count() {
    RTC_TICK_COUNT.store(0, Ordering::Relaxed);
}

/// Handle RTC interrupt (called in IRQ 8 handler)
pub fn handle_interrupt() {
    unsafe {
        Rtc::read_register(RTC_REG_C);
    }

    RTC_TICK_COUNT.fetch_add(1, Ordering::Relaxed);
}
// kernel/src/hal/rtc.rs
use crate::hal::io::{io_port_rb, io_port_wb};
use core::sync::atomic::{AtomicU64, Ordering};
use crate::log_info;

const RTC_INDEX_PORT: u16 = 0x70;
const RTC_TARGET_PORT: u16 = 0x71;
const WITH_NMI_DISABLED: u8 = 0x80;

const RTC_REG_SEC: u8 = 0x00;
const RTC_REG_MIN: u8 = 0x02;
const RTC_REG_HRS: u8 = 0x04;
const RTC_REG_WDY: u8 = 0x06;
const RTC_REG_DAY: u8 = 0x07;
const RTC_REG_MTH: u8 = 0x08;
const RTC_REG_YRS: u8 = 0x09;

const RTC_REG_A: u8 = 0x0A;
const RTC_REG_B: u8 = 0x0B;
const RTC_REG_C: u8 = 0x0C;

const RTC_UPDATE_IN_PROGRESS: u8 = 0x80;
const RTC_BIN_ENCODED_BIT: u8 = 0x04;
const RTC_24HRS_ENCODED_BIT: u8 = 0x02;

const RTC_TIMER_ON: u8 = 0x40;
const RTC_FREQUENCY_1024HZ: u8 = 0b110;
const RTC_DIVIDER_33KHZ: u8 = 0b010 << 4;

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

// RTC Configuration
static mut RTC_CONFIG: RtcConfig = RtcConfig::new();

// Tick Counter
static RTC_TICK_COUNT: AtomicU64 = AtomicU64::new(0);

struct RtcConfig {
    binary_mode: bool,
    hour_24_mode: bool,
}

impl RtcConfig {
    const fn new() -> Self {
        Self {
            binary_mode: false,
            hour_24_mode: false,
        }
    }
}

/// Read CMOS register
#[inline]
pub unsafe fn read_register(reg: u8) -> u8 {
    io_port_wb(RTC_INDEX_PORT, reg | WITH_NMI_DISABLED);
    io_port_rb(RTC_TARGET_PORT)
}

/// Write to CMOS register
#[inline]
pub unsafe fn write_register(reg: u8, value: u8) {
    io_port_wb(RTC_INDEX_PORT, reg | WITH_NMI_DISABLED);
    io_port_wb(RTC_TARGET_PORT, value);
}

/// Check if RTC is updating
#[inline]
unsafe fn is_updating() -> bool {
    (read_register(RTC_REG_A) & RTC_UPDATE_IN_PROGRESS) != 0
}

/// Wait for RTC update to complete
unsafe fn wait_for_update() {
    while is_updating() {
        core::hint::spin_loop();
    }
}

/// Convert BCD to binary
#[inline]
fn bcd_to_binary(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

pub fn init() {
    unsafe {
        // Read configuration
        let status_b = read_register(RTC_REG_B);
        RTC_CONFIG.binary_mode = (status_b & RTC_BIN_ENCODED_BIT) != 0;
        RTC_CONFIG.hour_24_mode = (status_b & RTC_24HRS_ENCODED_BIT) != 0;

        // Configuring frequencies and dividers
        let mut reg_a = read_register(RTC_REG_A);
        reg_a = (reg_a & 0xF0) | RTC_DIVIDER_33KHZ | RTC_FREQUENCY_1024HZ;
        write_register(RTC_REG_A, reg_a);

        // 清除中斷
        read_register(RTC_REG_C);

        // 確保 timer 關閉
        disable_timer();

        // 再次清除
        read_register(RTC_REG_C);
    }

    log_info!("RTC initialized (lock-free design)");
}

pub fn get_time() -> DateTime {
    unsafe {
        loop {
            // Wait for RTC to be ready
            wait_for_update();

            // Read twice quickly
            let time1 = read_time_raw();
            let time2 = read_time_raw();

            // If the two reads are consistent, return the result
            if time1 == time2 {
                return time1;
            }

            // Retry if inconsistent (maybe just rollover in seconds)
        }
    }
}

/// Directly read the RTC register
unsafe fn read_time_raw() -> DateTime {
    let config = &RTC_CONFIG;

    let mut second = read_register(RTC_REG_SEC);
    let mut minute = read_register(RTC_REG_MIN);
    let mut hour = read_register(RTC_REG_HRS);
    let mut day = read_register(RTC_REG_DAY);
    let mut month = read_register(RTC_REG_MTH);
    let mut year = read_register(RTC_REG_YRS);
    let weekday = read_register(RTC_REG_WDY);

    if !config.binary_mode {
        second = bcd_to_binary(second);
        minute = bcd_to_binary(minute);
        day = bcd_to_binary(day);
        month = bcd_to_binary(month);
        year = bcd_to_binary(year);
    }

    let pm_bit = hour & 0x80;
    hour = if config.binary_mode {
        hour & 0x7F
    } else {
        bcd_to_binary(hour & 0x7F)
    };

    if !config.hour_24_mode && pm_bit != 0 {
        hour = (hour + 12) % 24;
    }

    DateTime {
        year: RTC_CURRENT_CENTURY + year as u16,
        month,
        day,
        weekday,
        hour,
        minute,
        second,
    }
}

/// Enable RTC periodic interrupt (1024Hz)
pub fn enable_timer() {
    unsafe {
        // Close first
        disable_timer();
        read_register(RTC_REG_C);

        // Setting the frequency
        let mut reg_a = read_register(RTC_REG_A);
        reg_a = (reg_a & 0xF0) | RTC_DIVIDER_33KHZ | RTC_FREQUENCY_1024HZ;
        write_register(RTC_REG_A, reg_a);

        // Enable periodic interrupts
        let mut reg_b = read_register(RTC_REG_B);
        reg_b |= RTC_TIMER_ON;
        write_register(RTC_REG_B, reg_b);

        // Clear interrupt flag
        read_register(RTC_REG_C);
    }

    log_info!("RTC timer enabled at 1024Hz");
}

/// Disable RTC periodic interrupt
pub fn disable_timer() {
    unsafe {
        let mut reg_b = read_register(RTC_REG_B);
        reg_b &= !RTC_TIMER_ON;
        write_register(RTC_REG_B, reg_b);

        read_register(RTC_REG_C);
    }
}

/// RTC interrupt handler
///
/// This is the only function called in interrupt context.
/// Clears the interrupt flag
/// Increments the tick counter
#[inline]
pub fn handle_interrupt() {
    unsafe {
        read_register(RTC_REG_C);
    }

    RTC_TICK_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Get RTC tick count (lock-free)
#[inline]
pub fn get_tick_count() -> u64 {
    RTC_TICK_COUNT.load(Ordering::Relaxed)
}

/// Reset tick count
#[inline]
pub fn reset_tick_count() {
    RTC_TICK_COUNT.store(0, Ordering::Relaxed);
}

/// Print current time information
pub fn print_info() {
    let time = get_time();

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
}
// kernel/src/hal/rtc.rs
use crate::hal::io::{io_port_rb, io_port_wb};
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

    /// Read CMOS registers (with NMI disabled)
    unsafe fn read_register(reg: u8) -> u8 {
        io_port_wb(RTC_INDEX_PORT, reg | WITH_NMI_DISABLED);
        io_port_rb(RTC_TARGET_PORT)
    }

    /// Write to CMOS register (with NMI disabled)
    unsafe fn write_register(reg: u8, value: u8) {
        io_port_wb(RTC_INDEX_PORT, reg | WITH_NMI_DISABLED);
        io_port_wb(RTC_TARGET_PORT, value);
    }

    /// Check if RTC is updating
    unsafe fn is_updating() -> bool {
        (Self::read_register(RTC_REG_A) & RTC_UPDATE_IN_PROGRESS) != 0
    }

    /// Wait for RTC update to complete
    unsafe fn wait_for_update() {
        while Self::is_updating() {
            core::hint::spin_loop();
        }
    }

    /// Convert BCD to binary
    fn bcd_to_binary(bcd: u8) -> u8 {
        (bcd & 0x0F) + ((bcd >> 4) * 10)
    }

    /// Convert binary to BCD
    #[allow(dead_code)]
    fn binary_to_bcd(bin: u8) -> u8 {
        ((bin / 10) << 4) | (bin % 10)
    }

    pub fn init(&mut self) {
        unsafe {
            let status_b = Self::read_register(RTC_REG_B | WITH_NMI_DISABLED);
            self.binary_mode = (status_b & RTC_BIN_ENCODED_BIT) != 0;
            self.hour_24_mode = (status_b & RTC_24HRS_ENCODED_BIT) != 0;

            let mut reg_a = Self::read_register(RTC_REG_A | WITH_NMI_DISABLED);
            reg_a = (reg_a & 0xF0) | RTC_DIVIDER_33KHZ | RTC_FREQUENCY_1024HZ;
            Self::write_register(RTC_REG_A | WITH_NMI_DISABLED, reg_a);

            // ⭐ CRITICAL: Read Register C to clear any pending interrupts!
            Self::read_register(RTC_REG_C);

            self.disable_timer();
        }
    }

    /// Read raw RTC time data
    unsafe fn read_raw(&self) -> (u8, u8, u8, u8, u8, u8, u8) {
        Self::wait_for_update();

        let second = Self::read_register(RTC_REG_SEC);
        let minute = Self::read_register(RTC_REG_MIN);
        let hour = Self::read_register(RTC_REG_HRS);
        let day = Self::read_register(RTC_REG_DAY);
        let month = Self::read_register(RTC_REG_MTH);
        let year = Self::read_register(RTC_REG_YRS);
        let weekday = Self::read_register(RTC_REG_WDY);

        (second, minute, hour, day, month, year, weekday)
    }

    /// Convert the value based on encoding mode
    fn convert_value(&self, value: u8) -> u8 {
        if self.binary_mode {
            value
        } else {
            Self::bcd_to_binary(value)
        }
    }

    /// Read the RTC time
    pub fn read_time(&self) -> DateTime {
        unsafe {
            let (mut second, mut minute, mut hour, mut day, mut month, mut year, weekday) =
                self.read_raw();

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

    /// Read multiple times and ensure consistency
    pub fn read_time_stable(&self) -> DateTime {
        loop {
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

    /// Enable RTC timer interrupt (1024Hz)
    pub fn enable_timer(&self) {
        unsafe {
            let mut reg_b = Self::read_register(RTC_REG_B | WITH_NMI_DISABLED);
            reg_b |= RTC_TIMER_ON;
            Self::write_register(RTC_REG_B | WITH_NMI_DISABLED, reg_b);

            log_info!("RTC timer enabled at {}Hz", RTC_TIMER_BASE_FREQUENCY);
        }
    }

    /// Disable RTC timer interrupt
    pub fn disable_timer(&self) {
        unsafe {
            let mut reg_b = Self::read_register(RTC_REG_B | WITH_NMI_DISABLED);
            reg_b &= !RTC_TIMER_ON;
            Self::write_register(RTC_REG_B | WITH_NMI_DISABLED, reg_b);

            log_info!("RTC timer disabled");
        }
    }

    /// Read and clear RTC interrupt status (must be called in the interrupt handler)
    pub fn read_interrupt_status(&self) -> u8 {
        unsafe { Self::read_register(RTC_REG_C) }
    }
}

static RTC: Mutex<Option<Rtc>> = Mutex::new(None);

pub fn init() {
    let mut rtc = Rtc::new();
    rtc.init();
    *RTC.lock() = Some(rtc);

    log_info!("RTC initialized");
}

pub fn get_time() -> Option<DateTime> {
    let rtc = RTC.lock();
    let rtc = rtc.as_ref()?;
    Some(rtc.read_time_stable())
}

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
    if let Some(rtc) = RTC.lock().as_ref() {
        rtc.enable_timer();
    }
}

/// Disable the RTC timer
pub fn disable_timer() {
    if let Some(rtc) = RTC.lock().as_ref() {
        rtc.disable_timer();
    }
}

/// Handle RTC interrupt (needs to be called in IRQ 8 handler)
pub fn handle_interrupt() {
    if let Some(rtc) = RTC.lock().as_ref() {
        // CRITICAL: Must read Register C to clear the interrupt flag
        // Otherwise the RTC will not send the next interrupt!
        let status = rtc.read_interrupt_status();

        // bit 6 = periodic interrupt
        if (status & 0x40) != 0 {
            on_periodic_interrupt();
        }

        // bit 5 = alarm interrupt
        if (status & 0x20) != 0 {
            on_alarm_interrupt();
        }
    }
}

/// RTC periodic interrupt callback (can be overwritten by other modules)
#[allow(dead_code)]
fn on_periodic_interrupt() {
    // Handle timer events here
    // e.g., update system time, schedule tasks, etc.

    // For testing: increment counter
    unsafe {
        RTC_TICK_COUNT += 1;
    }
}

/// RTC alarm interrupt callback
#[allow(dead_code)]
fn on_alarm_interrupt() {
    // Handle alarm events here
}

// Test counter
static mut RTC_TICK_COUNT: u64 = 0;

/// Get RTC tick count (for testing)
pub fn get_tick_count() -> u64 {
    unsafe { RTC_TICK_COUNT }
}

/// Reset tick count (for testing)
pub fn reset_tick_count() {
    unsafe { RTC_TICK_COUNT = 0; }
}
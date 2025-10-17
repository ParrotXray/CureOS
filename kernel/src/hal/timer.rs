// kernel/src/hal/timer

use crate::hal::{lapic, rtc, cpu, ioapic};
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use crate::{log_info, log_debug, log_warn, log_error};

const APIC_CALIBRATION_CONST: u32 = 0x100000;
const RTC_BASE_FREQUENCY: u32 = 1024;

// APIC Timer register offset
const APIC_LVT_TIMER: u32 = 0x320;
const APIC_TIMER_ICR: u32 = 0x380;
const APIC_TIMER_DCR: u32 = 0x3E0;

/// APIC Timer divider
#[repr(u32)]
#[allow(dead_code)]
pub enum ApicTimerDivider {
    Div1 = 0b1011,
    Div2 = 0b0000,
    Div4 = 0b0001,
    Div8 = 0b0010,
    Div16 = 0b0011,
    Div32 = 0b1000,
    Div64 = 0b1001,
    Div128 = 0b1010,
}

// Timer configuration
static mut TIMER_CONFIG: TimerConfig = TimerConfig::new();

// Calibration status
static mut CALIBRATION: CalibrationState = CalibrationState::new();

// Runtime counter
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

// Calibration flag
static IS_CALIBRATING: AtomicBool = AtomicBool::new(false);

static TIMEOUT: AtomicU64 = AtomicU64::new(100_000_000);

struct TimerConfig {
    base_frequency: u32,
    running_frequency: u32,
    tick_interval: u32,
    initialized: bool,
}

impl TimerConfig {
    const fn new() -> Self {
        Self {
            base_frequency: 0,
            running_frequency: 0,
            tick_interval: 0,
            initialized: false,
        }
    }
}

struct CalibrationState {
    rtc_ticks: u64,
    done: bool,
    frequency: u64,
}

impl CalibrationState {
    const fn new() -> Self {
        Self {
            rtc_ticks: 0,
            done: false,
            frequency: 0,
        }
    }

    fn reset(&mut self) {
        self.rtc_ticks = 0;
        self.done = false;
        self.frequency = 0;
    }
}

/// Check if calibration is in progress (lock-free)
#[inline]
pub fn is_calibrating() -> bool {
    IS_CALIBRATING.load(Ordering::Relaxed)
}

/// Initialize and calibrate the APIC Timer
///
/// # Parameters
/// - `target_frequency`: Target frequency (Hz), recommended range: 100-1000
/// - `apic_id`: APIC ID of the current CPU
///
/// # Returns
/// Whether initialization was successful
pub fn init(target_frequency: u32, apic_id: u8) -> bool {
    if lapic::get_base_vaddr().is_none() {
        log_error!("LAPIC not initialized!");
        return false;
    }

    unsafe {
        CALIBRATION.reset();
    }

    IS_CALIBRATING.store(true, Ordering::SeqCst);

    cpu::cpu_disable_interrupts();

    log_debug!("Setting up APIC Timer for calibration...");

    unsafe {
        // Configure LVT Timer: one-shot mode, vector 32, masked
        lapic::write_apic_reg_raw(APIC_LVT_TIMER, 32 | (1 << 16));

        // Set the divider to 64
        lapic::write_apic_reg_raw(APIC_TIMER_DCR, ApicTimerDivider::Div64 as u32);
    }

    log_debug!("Configuring interrupts...");

    // Configure RTC interrupt (IRQ 8 -> Vector 40)
    ioapic::set_irq_redirect(8, 40, apic_id, false, false);
    ioapic::unmask_irq(8);

    log_info!("Starting calibration...");

    rtc::reset_tick_count();
    rtc::enable_timer();

    cpu::cpu_pause(1000);

    unsafe {
        // Unmask APIC Timer
        lapic::write_apic_reg_raw(APIC_LVT_TIMER, 32);

        // Write the initial count value and start counting down
        lapic::write_apic_reg_raw(APIC_TIMER_ICR, APIC_CALIBRATION_CONST);
    }

    log_debug!("Waiting for calibration...");

    cpu::cpu_enable_interrupts();

    let mut remaining = TIMEOUT.load(Ordering::Relaxed);
    while !unsafe { CALIBRATION.done } && remaining > 0 {
        cpu::cpu_pause(0);
        remaining -= 1;
    }
    cpu::cpu_disable_interrupts();

    if remaining == 0 {
        log_error!("Calibration timeout!");
        IS_CALIBRATING.store(false, Ordering::SeqCst);
        return false;
    }

    let base_frequency = unsafe { CALIBRATION.frequency as u32 };
    let rtc_ticks = unsafe { CALIBRATION.rtc_ticks };

    if base_frequency == 0 {
        log_error!("Calibration failed (freq = 0)!");
        IS_CALIBRATING.store(false, Ordering::SeqCst);
        return false;
    }

    log_info!("Calibration complete!");
    log_info!("RTC ticks: {}", rtc_ticks);
    log_info!("Base frequency: {} Hz", base_frequency);
    log_info!("Bus speed: ~{} MHz", base_frequency * 64 / 1_000_000);

    // Calculating the tick interval
    let tick_interval = base_frequency / target_frequency;

    log_info!("Configuring periodic timer...");
    log_info!("Target: {} Hz", target_frequency);
    log_info!("Interval: {}", tick_interval);

    unsafe {
        TIMER_CONFIG = TimerConfig {
            base_frequency,
            running_frequency: target_frequency,
            tick_interval,
            initialized: true,
        };
    }

    unsafe {
        // Configure for periodic mode: periodic bit | vector 32
        lapic::write_apic_reg_raw(APIC_LVT_TIMER, (1 << 17) | 32);

        // Set the count value
        lapic::write_apic_reg_raw(APIC_TIMER_ICR, tick_interval);
    }

    // Mark calibration completed
    IS_CALIBRATING.store(false, Ordering::SeqCst);

    // Ensure all writes are complete
    core::sync::atomic::fence(Ordering::SeqCst);

    log_info!("APIC Timer ready at {} Hz", target_frequency);
    log_info!("APIC Timer started successfully!");

    cpu::cpu_enable_interrupts();

    true
}

/// RTC interrupt handling (calibration phase)
///
/// Only called during calibration, only counts
#[inline]
pub fn rtc_calibration_handler() {
    unsafe {
        CALIBRATION.rtc_ticks += 1;
    }
}

/// APIC Timer Interrupt Handling (Calibration Phase)
///
/// Calculate frequency and mark completion
pub fn apic_calibration_handler() {
    let rtc_ticks = unsafe { CALIBRATION.rtc_ticks };

    if rtc_ticks == 0 {
        log_warn!("APIC Timer fired but RTC = 0!");
        unsafe {
            CALIBRATION.done = true;
        }
        return;
    }

    // 計算頻率: base_freq = (CONST / ticks) * RTC_FREQ
    let base_frequency = ((APIC_CALIBRATION_CONST as u64) * (RTC_BASE_FREQUENCY as u64))
        / rtc_ticks;

    log_debug!("Calibration: {} ticks -> {} Hz", rtc_ticks, base_frequency);

    unsafe {
        CALIBRATION.frequency = base_frequency;
        CALIBRATION.done = true;
    }

    // 停止 RTC
    rtc::disable_timer();
}

/// APIC Timer Periodic Tick Processing (Run Phase)
///
/// Counts only, does nothing else
#[inline]
pub fn timer_tick_handler() {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Get timer information
pub fn get_info() -> Option<(u32, u32, u64)> {
    unsafe {
        if TIMER_CONFIG.initialized {
            Some((
                TIMER_CONFIG.base_frequency,
                TIMER_CONFIG.running_frequency,
                TICK_COUNTER.load(Ordering::Relaxed)
            ))
        } else {
            None
        }
    }
}

/// Get the total number of ticks (lock-free)
#[inline]
pub fn get_tick_count() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// Reset the tick counter
#[inline]
pub fn reset_tick_count() {
    TICK_COUNTER.store(0, Ordering::Relaxed);
}

/// Get the basic frequency
#[inline]
pub fn get_base_frequency() -> Option<u32> {
    unsafe {
        if TIMER_CONFIG.initialized {
            Some(TIMER_CONFIG.base_frequency)
        } else {
            None
        }
    }
}

/// Get the running frequency
#[inline]
pub fn get_running_frequency() -> Option<u32> {
    unsafe {
        if TIMER_CONFIG.initialized {
            Some(TIMER_CONFIG.running_frequency)
        } else {
            None
        }
    }
}

/// Wait for the specified number of milliseconds (busy wait)
pub fn busy_wait_ms(ms: u64) {
    if let Some(freq) = get_running_frequency() {
        let ticks_to_wait = (ms * freq as u64) / 1000;
        let start = get_tick_count();

        while get_tick_count() - start < ticks_to_wait {
            cpu::cpu_pause(0);
        }
    }
}

/// Wait for the specified number of microseconds (busy wait)
pub fn busy_wait_us(us: u64) {
    if let Some(freq) = get_running_frequency() {
        let ticks_to_wait = (us * freq as u64) / 1_000_000;
        let start = get_tick_count();

        while get_tick_count() - start < ticks_to_wait {
            cpu::cpu_pause(0);
        }
    }
}

/// Print timer information
pub fn print_info() {
    if let Some((base_freq, running_freq, ticks)) = get_info() {
        log_info!("Base Frequency: {} Hz", base_freq);
        log_info!("Running Frequency: {} Hz", running_freq);
        log_info!("Total Ticks: {}", ticks);
        log_info!("Bus Speed: ~{} MHz", base_freq * 64 / 1_000_000);
    } else {
        log_warn!("APIC Timer not initialized");
    }
}
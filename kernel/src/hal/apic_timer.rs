// kernel/src/hal/apic_timer.rs - 使用 lapic 公開 API 的簡化版

use crate::hal::{lapic, rtc, cpu, ioapic};
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;
use crate::{log_info, log_debug, log_warn, log_error};

const APIC_CALIBRATION_CONST: u32 = 0x100000;
const RTC_BASE_FREQUENCY: u32 = 1024;

// APIC Timer 寄存器偏移
const APIC_LVT_TIMER: u32 = 0x320;
const APIC_TIMER_ICR: u32 = 0x380;
const APIC_TIMER_DCR: u32 = 0x3E0;

/// APIC Timer 分頻器
#[repr(u32)]
pub enum ApicTimerDivider {
    Div64 = 0b1001,
}

/// APIC Timer 上下文
pub struct ApicTimer {
    base_frequency: u32,
    running_frequency: u32,
    tick_interval: u32,
}

// 全局狀態
static APIC_TIMER: Mutex<Option<ApicTimer>> = Mutex::new(None);
static RTC_COUNTER: AtomicU64 = AtomicU64::new(0);
static CALIBRATION_DONE: AtomicBool = AtomicBool::new(false);
static CALIBRATED_FREQUENCY: AtomicU64 = AtomicU64::new(0);
static IS_CALIBRATING: AtomicBool = AtomicBool::new(false);
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

impl ApicTimer {
    fn new(base_frequency: u32, target_frequency: u32) -> Self {
        let tick_interval = base_frequency / target_frequency;

        Self {
            base_frequency,
            running_frequency: target_frequency,
            tick_interval,
        }
    }
}

/// 檢查是否正在校準
#[inline]
pub fn is_calibrating() -> bool {
    IS_CALIBRATING.load(Ordering::Relaxed)
}

/// 初始化並校準 APIC Timer
///
/// # Parameters
/// - `target_frequency`: 目標頻率 (Hz)，建議 100-1000
/// - `apic_id`: 當前 CPU 的 APIC ID
///
/// # Returns
/// 是否成功初始化
pub fn init(target_frequency: u32, apic_id: u8) -> bool {
    log_info!("=== APIC Timer Initialization ===");

    // 檢查 LAPIC 是否已初始化
    if lapic::get_base_vaddr().is_none() {
        log_error!("LAPIC not initialized!");
        return false;
    }

    // 重置校準狀態
    IS_CALIBRATING.store(true, Ordering::SeqCst);
    RTC_COUNTER.store(0, Ordering::SeqCst);
    CALIBRATION_DONE.store(false, Ordering::SeqCst);
    CALIBRATED_FREQUENCY.store(0, Ordering::SeqCst);

    // 禁用中斷
    cpu::cpu_disable_interrupts();

    log_debug!("Setting up APIC Timer for calibration...");

    unsafe {
        // 配置 LVT Timer: one-shot 模式, vector 32, masked
        lapic::write_apic_reg_raw(APIC_LVT_TIMER, 32 | (1 << 16));

        // 設置分頻器為 64
        lapic::write_apic_reg_raw(APIC_TIMER_DCR, ApicTimerDivider::Div64 as u32);
    }

    log_debug!("Configuring interrupts...");

    // 配置 RTC 中斷（IRQ 8 -> Vector 40）
    ioapic::set_irq_redirect(
        8,          // IRQ 8 (RTC)
        40,         // Vector 40
        apic_id,
        false,      // Edge triggered
        false       // Active high
    );
    ioapic::unmask_irq(8);

    log_info!("Starting calibration...");

    // 啟動 RTC
    rtc::reset_tick_count();
    rtc::enable_timer();

    // 延遲確保 RTC 啟動
    for _ in 0..1000 {
        cpu::cpu_pause();
    }

    unsafe {
        // Unmask APIC Timer
        lapic::write_apic_reg_raw(APIC_LVT_TIMER, 32);

        // 寫入初始計數值，開始倒數
        lapic::write_apic_reg_raw(APIC_TIMER_ICR, APIC_CALIBRATION_CONST);
    }

    log_debug!("Waiting for calibration...");

    // 啟用中斷
    cpu::cpu_enable_interrupts();

    // 等待校準完成（最多 3 秒）
    let mut timeout = 3_000_000;
    while !CALIBRATION_DONE.load(Ordering::SeqCst) && timeout > 0 {
        cpu::cpu_pause();
        timeout -= 1;
    }

    cpu::cpu_disable_interrupts();

    // 檢查超時
    if timeout == 0 {
        log_error!("Calibration timeout!");
        IS_CALIBRATING.store(false, Ordering::SeqCst);
        return false;
    }

    let base_frequency = CALIBRATED_FREQUENCY.load(Ordering::SeqCst) as u32;
    let rtc_ticks = RTC_COUNTER.load(Ordering::SeqCst);

    if base_frequency == 0 {
        log_error!("Calibration failed (freq = 0)!");
        IS_CALIBRATING.store(false, Ordering::SeqCst);
        return false;
    }

    log_info!("Calibration complete!");
    log_info!("  RTC ticks: {}", rtc_ticks);
    log_info!("  Base frequency: {} Hz", base_frequency);
    log_info!("  Bus speed: ~{} MHz", base_frequency * 64 / 1_000_000);

    // 創建 timer
    let timer = ApicTimer::new(base_frequency, target_frequency);

    log_info!("Configuring periodic timer...");
    log_info!("  Target: {} Hz", target_frequency);
    log_info!("  Interval: {}", timer.tick_interval);

    unsafe {
        // 配置為週期模式: periodic bit | vector 32
        lapic::write_apic_reg_raw(APIC_LVT_TIMER, (1 << 17) | 32);

        // 設置計數值
        lapic::write_apic_reg_raw(APIC_TIMER_ICR, timer.tick_interval);
    }

    // 先設置為非校準模式，再存儲 timer
    IS_CALIBRATING.store(false, Ordering::SeqCst);

    // 確保所有寫入完成
    core::sync::atomic::fence(Ordering::SeqCst);

    *APIC_TIMER.lock() = Some(timer);

    log_info!("APIC Timer ready at {} Hz", target_frequency);

    log_info!("APIC Timer started successfully!");

    true
}

/// RTC 中斷處理（校準階段）
#[inline]
pub fn rtc_calibration_handler() {
    RTC_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// APIC Timer 中斷處理（校準階段）
pub fn apic_calibration_handler() {
    let rtc_ticks = RTC_COUNTER.load(Ordering::Relaxed);

    if rtc_ticks == 0 {
        log_warn!("APIC Timer fired but RTC = 0!");
        CALIBRATION_DONE.store(true, Ordering::SeqCst);
        return;
    }

    // 計算頻率: base_freq = (CONST / ticks) * RTC_FREQ
    let base_frequency = ((APIC_CALIBRATION_CONST as u64) * (RTC_BASE_FREQUENCY as u64))
        / rtc_ticks;

    log_debug!("Calibration: {} ticks -> {} Hz", rtc_ticks, base_frequency);

    CALIBRATED_FREQUENCY.store(base_frequency, Ordering::SeqCst);
    CALIBRATION_DONE.store(true, Ordering::SeqCst);

    // 停止 RTC
    rtc::disable_timer();
}

/// APIC Timer 週期 tick 處理
pub fn timer_tick_handler() {
    let ticks = TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// 獲取 timer 信息
pub fn get_info() -> Option<(u32, u32, u64)> {
    APIC_TIMER.lock().as_ref().map(|t| {
        (
            t.base_frequency,
            t.running_frequency,
            TICK_COUNTER.load(Ordering::Relaxed)
        )
    })
}

/// 獲取總 tick 數
pub fn get_tick_count() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// 重置 tick 計數器
pub fn reset_tick_count() {
    TICK_COUNTER.store(0, Ordering::SeqCst);
}
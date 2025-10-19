// kernel/src/process/scheduler.rs
use super::process::{Process, ProcessId, ProcessState};
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use crate::{log_debug, log_info, log_warn};

const MAX_PROCESSES: usize = 256;
const DEFAULT_TIME_SLICE: u64 = 10;

static mut PROCESSES: [Option<Process>; MAX_PROCESSES] = {
    const NONE: Option<Process> = None;
    [NONE; MAX_PROCESSES]
};

static NEXT_PID: AtomicU64 = AtomicU64::new(1);
static CURRENT_PROCESS: AtomicUsize = AtomicUsize::new(0);
static PROCESS_COUNT: AtomicUsize = AtomicUsize::new(0);
static SCHEDULER_ENABLED: AtomicBool = AtomicBool::new(false);
static NEED_RESCHEDULE: AtomicBool = AtomicBool::new(false);

pub struct Scheduler;

impl Scheduler {
    pub fn init() {
        log_info!("Initializing preemptive scheduler");
        SCHEDULER_ENABLED.store(true, Ordering::Release);
    }

    /// 創建新進程
    pub fn spawn<F>(f: F, priority: u8) -> Option<ProcessId>
    where
        F: FnOnce(&corosensei::Yielder<(), ()>, ()) + 'static,
    {
        let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);

        // 創建進程（可能失敗）
        let process = match Process::new(pid, priority, DEFAULT_TIME_SLICE).spawn(f) {
            Some(p) => p,
            None => {
                log_warn!("Failed to create process: stack allocation failed");
                return None;
            }
        };

        unsafe {
            for i in 0..MAX_PROCESSES {
                if PROCESSES[i].is_none() {
                    PROCESSES[i] = Some(process);
                    PROCESS_COUNT.fetch_add(1, Ordering::Release);
                    log_debug!("Spawned process {} at slot {}", pid, i);
                    return Some(pid);
                }
            }
        }

        log_warn!("Failed to spawn process: no free slots");
        None
    }

    /// Timer 中斷處理器調用
    pub fn on_timer_tick() {
        if !SCHEDULER_ENABLED.load(Ordering::Acquire) {
            return;
        }

        let current_idx = CURRENT_PROCESS.load(Ordering::Acquire);

        unsafe {
            if let Some(ref mut process) = PROCESSES[current_idx] {
                if process.state == ProcessState::Running {
                    if process.time_slice > 0 {
                        process.time_slice -= 1;
                    }

                    if process.time_slice == 0 {
                        NEED_RESCHEDULE.store(true, Ordering::Release);
                    }
                }
            }
        }
    }

    /// 在安全點檢查是否需要調度
    pub fn check_reschedule() {
        if NEED_RESCHEDULE.swap(false, Ordering::AcqRel) {
            Self::schedule();
        }
    }

    /// 執行調度
    fn schedule() {
        let count = PROCESS_COUNT.load(Ordering::Acquire);
        if count == 0 {
            return;
        }

        let mut current_idx = CURRENT_PROCESS.load(Ordering::Acquire);
        let start_idx = current_idx;
        let mut found = false;

        unsafe {
            // 將當前進程設為 Ready（如果還在運行）
            if let Some(ref mut process) = PROCESSES[current_idx] {
                if process.state == ProcessState::Running {
                    process.state = ProcessState::Ready;
                    process.time_slice = DEFAULT_TIME_SLICE;
                }
            }

            // Round-robin 查找下一個可運行的進程
            loop {
                current_idx = (current_idx + 1) % MAX_PROCESSES;

                if let Some(ref mut process) = PROCESSES[current_idx] {
                    if process.state == ProcessState::Ready {
                        CURRENT_PROCESS.store(current_idx, Ordering::Release);
                        log_debug!("Switching to process {} (slot {})", process.id, current_idx);

                        // Resume 進程
                        log_debug!("About to resume process {}", process.id);
                        let result = process.resume();
                        log_debug!("Process {} resume returned: {}", process.id, result);

                        if !result {
                            log_debug!("Process {} terminated", process.id);
                            PROCESSES[current_idx] = None;
                            PROCESS_COUNT.fetch_sub(1, Ordering::Release);
                            continue;
                        }

                        found = true;
                        break;
                    }
                }

                // 遍歷一圈
                if current_idx == start_idx {
                    break;
                }
            }

            if !found {
                log_debug!("No runnable process found");
            }
        }
    }

    /// 主調度循環
    pub fn run() -> ! {
        log_info!("Starting scheduler main loop");

        loop {
            Self::schedule();

            if PROCESS_COUNT.load(Ordering::Acquire) == 0 {
                crate::hal::cpu::cpu_halt();
            }

            crate::hal::cpu::cpu_pause(0);
        }
    }

    /// 獲取統計信息
    pub fn stats() -> (usize, usize) {
        let count = PROCESS_COUNT.load(Ordering::Acquire);
        let current = CURRENT_PROCESS.load(Ordering::Acquire);
        (count, current)
    }
}
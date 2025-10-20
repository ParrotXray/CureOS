// kernel/src/process/scheduler.rs
use super::process::{Process, ProcessId, ProcessState};
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use crate::{hal, log_debug, log_info, log_warn};

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
        log_info!("Initializing scheduler with blocking support");
        SCHEDULER_ENABLED.store(true, Ordering::Release);
    }

    /// Create a new process
    pub fn spawn<F>(f: F, priority: u8) -> Option<ProcessId>
    where
        F: FnOnce(&corosensei::Yielder<(), ()>, ()) + 'static,
    {
        let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);

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
                    // log_debug!("Spawned process {} at slot {}", pid, i);
                    return Some(pid);
                }
            }
        }

        log_warn!("Failed to spawn process: no free slots");
        None
    }

    /// Block the current process
    pub fn block_current() {
        let current_idx = CURRENT_PROCESS.load(Ordering::Acquire);

        unsafe {
            if let Some(ref mut process) = PROCESSES[current_idx] {
                process.state = ProcessState::Blocked;
                // log_debug!("Process {} (slot {}) blocked", process.id, current_idx);
            }
        }
    }

    /// Wake up the specified process
    pub fn wake_process(pid: u64) {
        unsafe {
            for i in 0..MAX_PROCESSES {
                if let Some(ref mut process) = PROCESSES[i] {
                    if process.id == pid && process.state == ProcessState::Blocked {
                        process.state = ProcessState::Ready;
                        // log_debug!("Process {} (slot {}) woken up", process.id, i);
                        NEED_RESCHEDULE.store(true, Ordering::Release);
                        return;
                    }
                }
            }
        }
        log_warn!("Tried to wake non-existent or non-blocked process {}", pid);
    }

    /// Timer interrupt call
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

    /// Check if rescheduling is needed
    pub fn check_reschedule() {
        if NEED_RESCHEDULE.swap(false, Ordering::AcqRel) {
            Self::schedule();
        }
    }

    /// Execute scheduling
    fn schedule() {
        let count = PROCESS_COUNT.load(Ordering::Acquire);
        if count == 0 {
            return;
        }

        unsafe {
            let current_idx = CURRENT_PROCESS.load(Ordering::Acquire);

            // Set the current process to Ready (if it is running)
            if let Some(ref mut process) = PROCESSES[current_idx] {
                if process.state == ProcessState::Running {
                    process.state = ProcessState::Ready;
                    process.time_slice = DEFAULT_TIME_SLICE;
                }
            }

            // Round-robin to find the next Ready process
            let mut next_idx = current_idx;
            let mut attempts = 0;

            loop {
                next_idx = (next_idx + 1) % MAX_PROCESSES;
                attempts += 1;

                if attempts > MAX_PROCESSES {
                    // There is no Ready process, all processes are blocked
                    // log_debug!("All processes blocked or terminated");
                    return;
                }

                if let Some(ref mut process) = PROCESSES[next_idx] {
                    if process.state == ProcessState::Ready {
                        CURRENT_PROCESS.store(next_idx, Ordering::Release);

                        // log_debug!("Switching to process {} (slot {})", process.id, next_idx);

                        let result = process.resume();

                        if !result {
                            // log_debug!("Process {} terminated", process.id);
                            PROCESSES[next_idx] = None;
                            PROCESS_COUNT.fetch_sub(1, Ordering::Release);
                            continue;
                        }

                        return;
                    }
                }
            }
        }
    }

    /// Main scheduling loop
    pub fn run() -> ! {
        loop {
            Self::schedule();

            let count = PROCESS_COUNT.load(Ordering::Acquire);
            if count == 0 {
                log_info!("No processes remaining, system idle");
                hal::cpu::cpu_halt();
            }

            hal::cpu::cpu_pause(500);
        }
    }

    /// Get the current process ID
    pub fn current_pid() -> Option<u64> {
        let current_idx = CURRENT_PROCESS.load(Ordering::Acquire);
        unsafe {
            PROCESSES[current_idx].as_ref().map(|p| p.id)
        }
    }
}
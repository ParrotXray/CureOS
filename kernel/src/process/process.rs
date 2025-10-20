// kernel/src/process/process.rs
use corosensei::{Coroutine, CoroutineResult, Yielder};
use super::stack::ProcessStack;

pub type ProcessId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

pub struct Process {
    pub id: ProcessId,
    pub state: ProcessState,
    pub coroutine: Option<Coroutine<(), (), (), ProcessStack>>,
    pub time_slice: u64,
    pub priority: u8,
}

impl Process {
    pub fn new(id: ProcessId, priority: u8, time_slice: u64) -> Self {
        Self {
            id,
            state: ProcessState::Ready,
            coroutine: None,
            time_slice,
            priority,
        }
    }

    pub fn spawn<F>(mut self, f: F) -> Option<Self>
    where
        F: FnOnce(&Yielder<(), ()>, ()) + 'static,
    {
        let stack = ProcessStack::new()?;
        let coro = Coroutine::with_stack(stack, f);
        self.coroutine = Some(coro);
        Some(self)
    }

    pub fn resume(&mut self) -> bool {
        if let Some(ref mut coro) = self.coroutine {
            self.state = ProcessState::Running;

            match coro.resume(()) {
                CoroutineResult::Yield(()) => {
                    // Set to Ready only when not in Blocked state
                    if self.state == ProcessState::Running {
                        self.state = ProcessState::Ready;
                    }
                    true
                }
                CoroutineResult::Return(()) => {
                    self.state = ProcessState::Terminated;
                    false
                }
            }
        } else {
            false
        }
    }
}
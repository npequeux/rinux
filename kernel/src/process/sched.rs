//! Scheduler
//!
//! Basic round-robin process scheduler implementation.

use super::task::{Task, TaskState};
use crate::types::Pid;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// Maximum number of tasks
#[allow(dead_code)]
const MAX_TASKS: usize = 256;

/// Global scheduler state
static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Scheduler initialization flag
static SCHEDULER_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Scheduler structure
pub struct Scheduler {
    /// Ready queue for runnable tasks
    ready_queue: VecDeque<Pid>,
    /// All tasks indexed by PID
    tasks: Vec<Option<Task>>,
    /// Current running task
    current: Option<Pid>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// Create a new scheduler
    pub const fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            tasks: Vec::new(),
            current: None,
        }
    }

    /// Add a task to the scheduler
    pub fn add_task(&mut self, task: Task) {
        let pid = task.pid;

        // Ensure the tasks vector is large enough
        while self.tasks.len() <= pid as usize {
            self.tasks.push(None);
        }

        self.tasks[pid as usize] = Some(task);
        self.ready_queue.push_back(pid);
    }

    /// Remove a task from the scheduler
    pub fn remove_task(&mut self, pid: Pid) {
        if let Some(task_slot) = self.tasks.get_mut(pid as usize) {
            *task_slot = None;
        }
        self.ready_queue.retain(|&p| p != pid);
        if self.current == Some(pid) {
            self.current = None;
        }
    }

    /// Get a task by PID
    pub fn get_task(&self, pid: Pid) -> Option<&Task> {
        self.tasks.get(pid as usize).and_then(|t| t.as_ref())
    }

    /// Get a mutable task by PID
    pub fn get_task_mut(&mut self, pid: Pid) -> Option<&mut Task> {
        self.tasks.get_mut(pid as usize).and_then(|t| t.as_mut())
    }

    /// Schedule next task (round-robin)
    pub fn schedule_next(&mut self) -> Option<Pid> {
        // Move current task back to ready queue if it's still running
        if let Some(current_pid) = self.current {
            if let Some(task) = self.get_task(current_pid) {
                if task.state == TaskState::Running {
                    self.ready_queue.push_back(current_pid);
                }
            }
        }

        // Get next task from ready queue
        while let Some(pid) = self.ready_queue.pop_front() {
            if let Some(task) = self.get_task_mut(pid) {
                if task.state == TaskState::Running {
                    self.current = Some(pid);
                    return Some(pid);
                }
            }
        }

        self.current = None;
        None
    }

    /// Get current task PID
    pub fn current_pid(&self) -> Option<Pid> {
        self.current
    }

    /// Mark current task as yielding
    pub fn yield_current(&mut self) {
        if let Some(current_pid) = self.current {
            // Move current task to back of ready queue
            self.ready_queue.push_back(current_pid);
            self.current = None;
        }
    }

    /// Get number of tasks
    pub fn task_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.is_some()).count()
    }

    /// Get number of ready tasks
    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }
}

/// Initialize the scheduler
pub fn init() {
    if SCHEDULER_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    let mut sched = SCHEDULER.lock();

    // Create idle task (PID 0)
    let idle_task = Task::new(0);
    sched.add_task(idle_task);

    SCHEDULER_INITIALIZED.store(true, Ordering::Release);

    crate::printk::printk("  Scheduler initialized (round-robin)\n");
}

/// Schedule next task
pub fn schedule() {
    let mut sched = SCHEDULER.lock();
    if let Some(_next_pid) = sched.schedule_next() {
        // TODO: Perform actual context switch using arch-specific code
        // For now, we're updating the scheduler state
        // In a complete implementation, this would:
        // 1. Save current task's CPU context
        // 2. Load next task's CPU context
        // 3. Switch page tables
        // 4. Jump to next task
        drop(sched);

        // Note: Context switching would be implemented like:
        // if let Some(current_ctx) = get_current_context() {
        //     if let Some(next_ctx) = get_task_context(next_pid) {
        //         unsafe {
        //             rinux_arch_x86::context::switch_context(current_ctx, next_ctx);
        //         }
        //     }
        // }
    }
}

/// Yield CPU to another task
pub fn yield_now() {
    let mut sched = SCHEDULER.lock();
    sched.yield_current();
    drop(sched);
    schedule();
}

/// Add a task to the scheduler
pub fn add_task(task: Task) {
    let mut sched = SCHEDULER.lock();
    sched.add_task(task);
}

/// Remove a task from the scheduler
pub fn remove_task(pid: Pid) {
    let mut sched = SCHEDULER.lock();
    sched.remove_task(pid);
}

/// Get current task PID
pub fn current_pid() -> Option<Pid> {
    let sched = SCHEDULER.lock();
    sched.current_pid()
}

/// Get task count
pub fn task_count() -> usize {
    let sched = SCHEDULER.lock();
    sched.task_count()
}

/// Get ready task count
pub fn ready_count() -> usize {
    let sched = SCHEDULER.lock();
    sched.ready_count()
}

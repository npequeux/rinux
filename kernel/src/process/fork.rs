//! Process Fork Implementation
//!
//! Implementation of process forking (clone system call).

use super::task::{Task, TaskState};
use crate::types::Pid;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicI32, Ordering};
use spin::Mutex;

/// Next available PID
static NEXT_PID: AtomicI32 = AtomicI32::new(1);

/// Allocate a new PID
fn alloc_pid() -> Pid {
    NEXT_PID.fetch_add(1, Ordering::SeqCst)
}

/// Process memory context
#[derive(Clone)]
pub struct MemoryContext {
    /// Page table pointer
    pub page_table: u64,
    /// Heap start
    pub heap_start: u64,
    /// Heap end
    pub heap_end: u64,
    /// Stack start
    pub stack_start: u64,
    /// Stack end
    pub stack_end: u64,
}

impl MemoryContext {
    /// Create a new memory context
    pub fn new() -> Self {
        Self {
            page_table: 0,
            heap_start: 0,
            heap_end: 0,
            stack_start: 0,
            stack_end: 0,
        }
    }

    /// Clone the memory context (copy-on-write would be implemented here)
    pub fn clone_for_fork(&self) -> Self {
        // In a real implementation, this would:
        // 1. Create a new page table
        // 2. Copy or mark pages as copy-on-write
        // 3. Set up proper memory mappings
        Self {
            page_table: self.page_table, // TODO: Clone page tables
            heap_start: self.heap_start,
            heap_end: self.heap_end,
            stack_start: self.stack_start,
            stack_end: self.stack_end,
        }
    }
}

/// CPU register state
#[derive(Clone, Copy)]
#[repr(C)]
pub struct RegisterState {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

impl RegisterState {
    /// Create a new register state
    pub const fn new() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rflags: 0,
        }
    }
}

/// Extended task structure with fork support
pub struct ExtendedTask {
    /// Base task structure
    pub task: Task,
    /// Memory context
    pub memory: MemoryContext,
    /// CPU register state
    pub registers: RegisterState,
}

impl ExtendedTask {
    /// Create a new extended task
    pub fn new(pid: Pid) -> Self {
        Self {
            task: Task::new(pid),
            memory: MemoryContext::new(),
            registers: RegisterState::new(),
        }
    }

    /// Fork this task, creating a child process
    pub fn fork(&self) -> Result<ExtendedTask, &'static str> {
        let child_pid = alloc_pid();

        let mut child = ExtendedTask {
            task: Task::new_with_parent(child_pid, self.task.pid),
            memory: self.memory.clone_for_fork(),
            registers: self.registers,
        };

        // Child process should return 0 from fork
        child.registers.rax = 0;

        // Copy credentials
        child.task.uid = self.task.uid;
        child.task.gid = self.task.gid;

        Ok(child)
    }
}

/// Global task list with fork support
static EXTENDED_TASKS: Mutex<Vec<ExtendedTask>> = Mutex::new(Vec::new());

/// Fork the current process
pub fn do_fork() -> Result<Pid, &'static str> {
    let mut tasks = EXTENDED_TASKS.lock();

    // Get current process (simplified - in reality would use scheduler's current)
    let current_idx = tasks
        .iter()
        .position(|t| t.task.state == TaskState::Running)
        .ok_or("No running process")?;

    let current = &tasks[current_idx];
    let child = current.fork()?;
    let child_pid = child.task.pid;

    tasks.push(child);

    Ok(child_pid)
}

/// Initialize fork subsystem
pub fn init() {
    // Create init process (PID 1)
    let init_task = ExtendedTask::new(1);
    let mut tasks = EXTENDED_TASKS.lock();
    tasks.push(init_task);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_allocation() {
        let pid1 = alloc_pid();
        let pid2 = alloc_pid();
        assert!(pid2 > pid1);
    }

    #[test]
    fn test_memory_context_new() {
        let ctx = MemoryContext::new();
        assert_eq!(ctx.page_table, 0);
        assert_eq!(ctx.heap_start, 0);
    }

    #[test]
    fn test_register_state_new() {
        let regs = RegisterState::new();
        assert_eq!(regs.rax, 0);
        assert_eq!(regs.rip, 0);
    }

    #[test]
    fn test_extended_task_new() {
        let task = ExtendedTask::new(42);
        assert_eq!(task.task.pid, 42);
    }
}

//! Task Structure
//!
//! Process/thread task structure.

use crate::types::{Gid, Pid, Uid};

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is running
    Running,
    /// Task is sleeping/waiting
    Sleeping,
    /// Task is stopped
    Stopped,
    /// Task has exited but not reaped
    Zombie,
}

/// Task priority (lower number = higher priority)
pub type Priority = u8;

/// Default task priority
pub const DEFAULT_PRIORITY: Priority = 120;

/// Task structure
pub struct Task {
    /// Process ID
    pub pid: Pid,
    /// User ID
    pub uid: Uid,
    /// Group ID
    pub gid: Gid,
    /// Task state
    pub state: TaskState,
    /// Priority (0-255, lower is higher priority)
    pub priority: Priority,
    /// Parent process ID
    pub parent_pid: Option<Pid>,
    /// Exit code (if zombie)
    pub exit_code: Option<i32>,
}

impl Task {
    /// Create a new task with default values
    pub fn new(pid: Pid) -> Self {
        Task {
            pid,
            uid: 0,
            gid: 0,
            state: TaskState::Running,
            priority: DEFAULT_PRIORITY,
            parent_pid: None,
            exit_code: None,
        }
    }

    /// Create a new task with parent
    pub fn new_with_parent(pid: Pid, parent_pid: Pid) -> Self {
        Task {
            pid,
            uid: 0,
            gid: 0,
            state: TaskState::Running,
            priority: DEFAULT_PRIORITY,
            parent_pid: Some(parent_pid),
            exit_code: None,
        }
    }

    /// Set task state
    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    /// Set priority
    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    /// Mark task as exited
    pub fn exit(&mut self, code: i32) {
        self.state = TaskState::Zombie;
        self.exit_code = Some(code);
    }

    /// Check if task is runnable
    pub fn is_runnable(&self) -> bool {
        self.state == TaskState::Running
    }
}

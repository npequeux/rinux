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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_variants() {
        // Test that all TaskState variants exist
        let _running = TaskState::Running;
        let _sleeping = TaskState::Sleeping;
        let _stopped = TaskState::Stopped;
        let _zombie = TaskState::Zombie;
    }

    #[test]
    fn test_task_state_equality() {
        assert_eq!(TaskState::Running, TaskState::Running);
        assert_eq!(TaskState::Sleeping, TaskState::Sleeping);
        assert_eq!(TaskState::Stopped, TaskState::Stopped);
        assert_eq!(TaskState::Zombie, TaskState::Zombie);
        
        assert_ne!(TaskState::Running, TaskState::Sleeping);
        assert_ne!(TaskState::Running, TaskState::Stopped);
        assert_ne!(TaskState::Running, TaskState::Zombie);
    }

    #[test]
    fn test_task_state_clone() {
        let state1 = TaskState::Running;
        let state2 = state1.clone();
        assert_eq!(state1, state2);
    }

    #[test]
    fn test_task_state_copy() {
        let state1 = TaskState::Running;
        let state2 = state1;
        assert_eq!(state1, state2);
    }

    #[test]
    fn test_task_new() {
        let task = Task::new(42);
        assert_eq!(task.pid, 42);
        assert_eq!(task.uid, 0);
        assert_eq!(task.gid, 0);
        assert_eq!(task.state, TaskState::Running);
    }

    #[test]
    fn test_task_new_with_different_pids() {
        let task1 = Task::new(1);
        let task2 = Task::new(100);
        let task3 = Task::new(-1);
        
        assert_eq!(task1.pid, 1);
        assert_eq!(task2.pid, 100);
        assert_eq!(task3.pid, -1);
    }

    #[test]
    fn test_task_initial_state() {
        let task = Task::new(1);
        assert_eq!(task.state, TaskState::Running);
    }

    #[test]
    fn test_task_modify_state() {
        let mut task = Task::new(1);
        assert_eq!(task.state, TaskState::Running);
        
        task.state = TaskState::Sleeping;
        assert_eq!(task.state, TaskState::Sleeping);
        
        task.state = TaskState::Stopped;
        assert_eq!(task.state, TaskState::Stopped);
        
        task.state = TaskState::Zombie;
        assert_eq!(task.state, TaskState::Zombie);
    }

    #[test]
    fn test_task_modify_uid_gid() {
        let mut task = Task::new(1);
        assert_eq!(task.uid, 0);
        assert_eq!(task.gid, 0);
        
        task.uid = 1000;
        task.gid = 1000;
        
        assert_eq!(task.uid, 1000);
        assert_eq!(task.gid, 1000);
    }

    #[test]
    fn test_task_fields_independent() {
        let mut task = Task::new(42);
        
        task.pid = 100;
        task.uid = 1000;
        task.gid = 2000;
        task.state = TaskState::Sleeping;
        
        assert_eq!(task.pid, 100);
        assert_eq!(task.uid, 1000);
        assert_eq!(task.gid, 2000);
        assert_eq!(task.state, TaskState::Sleeping);
    }
}

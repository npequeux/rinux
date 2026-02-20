//! Task Structure
//!
//! Process/thread task structure.

use crate::types::{Gid, Pid, Uid};

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Sleeping,
    Stopped,
    Zombie,
}

/// Task structure
pub struct Task {
    pub pid: Pid,
    pub uid: Uid,
    pub gid: Gid,
    pub state: TaskState,
    // TODO: Add more fields
}

impl Task {
    pub fn new(pid: Pid) -> Self {
        Task {
            pid,
            uid: 0,
            gid: 0,
            state: TaskState::Running,
        }
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

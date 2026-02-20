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

//! Wait System Calls
//!
//! Implementation of wait, waitpid, wait4 system calls.

use super::task::{Task, TaskState};
use crate::types::Pid;
use alloc::vec::Vec;
use spin::Mutex;

/// Wait options
pub mod wait_options {
    /// Wait for any child
    pub const WNOHANG: i32 = 1;
    /// Report status of stopped children
    pub const WUNTRACED: i32 = 2;
    /// Wait for continued children
    pub const WCONTINUED: i32 = 8;
}

/// Exit status
#[derive(Debug, Clone, Copy)]
pub struct ExitStatus {
    /// Raw status value
    pub status: i32,
}

impl ExitStatus {
    /// Create exit status from exit code
    pub fn exited(code: i32) -> Self {
        ExitStatus {
            status: (code & 0xFF) << 8,
        }
    }

    /// Create status for signal termination
    pub fn signaled(signal: i32) -> Self {
        ExitStatus {
            status: signal & 0x7F,
        }
    }

    /// Check if process exited normally
    pub fn is_exited(&self) -> bool {
        (self.status & 0x7F) == 0
    }

    /// Get exit code if exited normally
    pub fn exit_code(&self) -> Option<i32> {
        if self.is_exited() {
            Some((self.status >> 8) & 0xFF)
        } else {
            None
        }
    }

    /// Check if process was terminated by signal
    pub fn is_signaled(&self) -> bool {
        ((self.status & 0x7F) + 1) as i8 >> 1 > 0
    }

    /// Get terminating signal if signaled
    pub fn signal(&self) -> Option<i32> {
        if self.is_signaled() {
            Some(self.status & 0x7F)
        } else {
            None
        }
    }
}

/// Wait result
pub enum WaitResult {
    /// Child exited
    Exited(Pid, ExitStatus),
    /// No child available (WNOHANG)
    NoChild,
    /// Still waiting
    Waiting,
}

/// Global zombie process list
static ZOMBIE_PROCESSES: Mutex<Vec<(Pid, Pid, ExitStatus)>> = Mutex::new(Vec::new());

/// Register a zombie process
pub fn register_zombie(pid: Pid, parent_pid: Pid, exit_code: i32) {
    let status = ExitStatus::exited(exit_code);
    ZOMBIE_PROCESSES.lock().push((pid, parent_pid, status));
}

/// Wait for any child process
pub fn wait_any(parent_pid: Pid, options: i32) -> Result<WaitResult, &'static str> {
    let mut zombies = ZOMBIE_PROCESSES.lock();
    
    // Look for zombie child of this parent
    if let Some(idx) = zombies.iter().position(|(_, ppid, _)| *ppid == parent_pid) {
        let (child_pid, _, status) = zombies.remove(idx);
        return Ok(WaitResult::Exited(child_pid, status));
    }
    
    // Check if WNOHANG is set
    if (options & wait_options::WNOHANG) != 0 {
        return Ok(WaitResult::NoChild);
    }
    
    // Would need to block the parent process here
    // For now, return waiting
    Ok(WaitResult::Waiting)
}

/// Wait for a specific child process
pub fn wait_pid(parent_pid: Pid, child_pid: Pid, options: i32) -> Result<WaitResult, &'static str> {
    let mut zombies = ZOMBIE_PROCESSES.lock();
    
    // Look for specific zombie child
    if let Some(idx) = zombies.iter().position(|(pid, ppid, _)| *pid == child_pid && *ppid == parent_pid) {
        let (_, _, status) = zombies.remove(idx);
        return Ok(WaitResult::Exited(child_pid, status));
    }
    
    // Check if WNOHANG is set
    if (options & wait_options::WNOHANG) != 0 {
        return Ok(WaitResult::NoChild);
    }
    
    // Would need to block the parent process here
    Ok(WaitResult::Waiting)
}

/// Process exit - convert to zombie state
pub fn process_exit(task: &mut Task, exit_code: i32) {
    task.exit(exit_code);
    
    // Register as zombie if has parent
    if let Some(parent_pid) = task.parent_pid {
        register_zombie(task.pid, parent_pid, exit_code);
        
        // TODO: Send SIGCHLD to parent
    } else {
        // Init process or orphan - clean up immediately
        // TODO: Reap resources
    }
}

/// Handle orphaned processes (parent died)
pub fn reparent_to_init(orphaned_pid: Pid) {
    // In Linux, orphaned processes are reparented to init (PID 1)
    // TODO: Update parent_pid in task structure
    let _ = orphaned_pid;
}

/// Reap zombie process resources
pub fn reap_zombie(pid: Pid) -> Result<(), &'static str> {
    // Clean up process resources
    // TODO: Free page tables, memory, file descriptors, etc.
    let _ = pid;
    Ok(())
}

/// Initialize wait subsystem
pub fn init() {
    // Nothing to initialize yet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_status_exited() {
        let status = ExitStatus::exited(42);
        assert!(status.is_exited());
        assert_eq!(status.exit_code(), Some(42));
        assert!(!status.is_signaled());
    }

    #[test]
    fn test_exit_status_signaled() {
        let status = ExitStatus::signaled(9); // SIGKILL
        assert!(!status.is_exited());
        assert_eq!(status.exit_code(), None);
        assert!(status.is_signaled());
        assert_eq!(status.signal(), Some(9));
    }

    #[test]
    fn test_wait_options() {
        assert_eq!(wait_options::WNOHANG, 1);
        assert_eq!(wait_options::WUNTRACED, 2);
        assert_eq!(wait_options::WCONTINUED, 8);
    }
}

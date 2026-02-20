//! PID Management
//!
//! Process ID allocation and management.

use crate::types::Pid;
use spin::Mutex;

static NEXT_PID: Mutex<Pid> = Mutex::new(1);

/// Allocate a new PID
pub fn allocate_pid() -> Pid {
    let mut next = NEXT_PID.lock();
    let pid = *next;
    *next += 1;
    pid
}

/// Free a PID
pub fn free_pid(_pid: Pid) {
    // TODO: Implement PID recycling
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec::Vec;

    #[test]
    fn test_allocate_pid_positive() {
        let pid = allocate_pid();
        assert!(pid >= 1);
    }

    #[test]
    fn test_allocate_pid_increments() {
        let pid1 = allocate_pid();
        let pid2 = allocate_pid();
        let pid3 = allocate_pid();

        // PIDs should be strictly increasing for successive allocations
        assert!(pid2 > pid1);
        assert!(pid3 > pid2);
    }

    #[test]
    fn test_allocate_pid_unique() {
        let pid1 = allocate_pid();
        let pid2 = allocate_pid();

        // PIDs should be unique
        assert_ne!(pid1, pid2);
    }

    #[test]
    fn test_free_pid_no_panic() {
        // free_pid should not panic (even though it's a stub)
        let pid = allocate_pid();
        free_pid(pid);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut pids = Vec::new();
        for _ in 0..10 {
            pids.push(allocate_pid());
        }

        // All PIDs should be unique
        for i in 0..pids.len() {
            for j in (i + 1)..pids.len() {
                assert_ne!(pids[i], pids[j]);
            }
        }
    }
}

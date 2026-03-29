//! PID Management
//!
//! Process ID allocation and management.

use crate::types::Pid;
use alloc::collections::VecDeque;
use spin::Mutex;

/// Maximum PID value before wrapping
const PID_MAX: Pid = 32768;

/// PID allocator state
struct PidAllocator {
    next: Pid,
    recycled: VecDeque<Pid>,
}

impl PidAllocator {
    const fn new() -> Self {
        PidAllocator {
            next: 1,
            recycled: VecDeque::new(),
        }
    }

    fn allocate(&mut self) -> Pid {
        // Prefer recycled PIDs
        if let Some(pid) = self.recycled.pop_front() {
            return pid;
        }
        let pid = self.next;
        // Wrap to 1 after PID_MAX so that PID_MAX itself is allocated before wrapping
        self.next = if self.next >= PID_MAX {
            1
        } else {
            self.next + 1
        };
        pid
    }

    fn free(&mut self, pid: Pid) {
        if pid > 0 {
            self.recycled.push_back(pid);
        }
    }
}

static PID_ALLOCATOR: Mutex<PidAllocator> = Mutex::new(PidAllocator::new());

/// Allocate a new PID
pub fn allocate_pid() -> Pid {
    PID_ALLOCATOR.lock().allocate()
}

/// Free a PID back to the pool for reuse
pub fn free_pid(pid: Pid) {
    PID_ALLOCATOR.lock().free(pid);
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
    fn test_free_pid_recycled() {
        // Allocate a PID, free it, then allocate again - should get recycled PID
        let mut allocator = PidAllocator::new();
        let pid1 = allocator.allocate();
        allocator.free(pid1);
        let pid2 = allocator.allocate();
        assert_eq!(pid1, pid2, "Freed PID should be recycled");
    }

    #[test]
    fn test_free_pid_no_panic() {
        // free_pid should not panic
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

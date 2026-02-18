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

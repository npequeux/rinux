//! Out-of-Memory (OOM) Killer
//!
//! Handles out-of-memory situations by selecting and killing processes.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;
use alloc::vec::Vec;

/// OOM killer statistics
static OOM_KILLS: AtomicU64 = AtomicU64::new(0);
static OOM_ENABLED: AtomicBool = AtomicBool::new(true);

/// Process information for OOM scoring
#[derive(Debug, Clone)]
pub struct ProcessOomInfo {
    pub pid: i32,
    pub memory_usage: u64,  // in bytes
    pub oom_score_adj: i16, // -1000 to 1000, user-adjustable
    pub is_kernel: bool,
    pub is_init: bool,
}

impl ProcessOomInfo {
    /// Calculate OOM score for this process
    /// Higher score = more likely to be killed
    pub fn oom_score(&self) -> i64 {
        // Never kill kernel processes or init
        if self.is_kernel || self.is_init {
            return i64::MIN;
        }

        // Base score from memory usage (in MB)
        let mut score = (self.memory_usage / (1024 * 1024)) as i64;

        // Adjust based on user preference (-1000 to 1000)
        // oom_score_adj of -1000 makes process unkillable (unless root)
        score += self.oom_score_adj as i64;

        // Ensure score is non-negative
        score.max(0)
    }
}

/// OOM killer state
struct OomKiller {
    enabled: bool,
    min_free_memory: u64, // Minimum free memory before OOM kicks in
    _last_kill_time: u64,  // Timestamp of last kill
}

impl Default for OomKiller {
    fn default() -> Self {
        Self::new()
    }
}

impl OomKiller {
    const fn new() -> Self {
        OomKiller {
            enabled: true,
            min_free_memory: 16 * 1024 * 1024, // 16 MB
            _last_kill_time: 0,
        }
    }

    /// Select a victim process to kill
    fn select_victim(&self, processes: &[ProcessOomInfo]) -> Option<i32> {
        let mut best_score = i64::MIN;
        let mut victim_pid = None;

        for proc in processes {
            let score = proc.oom_score();
            if score > best_score {
                best_score = score;
                victim_pid = Some(proc.pid);
            }
        }

        victim_pid
    }
}

static OOM_KILLER: Mutex<OomKiller> = Mutex::new(OomKiller::new());

/// Initialize OOM killer
pub fn init() {
    OOM_ENABLED.store(true, Ordering::Release);
}

/// Check if OOM killer is enabled
pub fn is_enabled() -> bool {
    OOM_ENABLED.load(Ordering::Acquire)
}

/// Enable OOM killer
pub fn enable() {
    OOM_ENABLED.store(true, Ordering::Release);
    let mut killer = OOM_KILLER.lock();
    killer.enabled = true;
}

/// Disable OOM killer (dangerous!)
pub fn disable() {
    OOM_ENABLED.store(false, Ordering::Release);
    let mut killer = OOM_KILLER.lock();
    killer.enabled = false;
}

/// Trigger OOM killer when out of memory
///
/// Returns the PID of the killed process, if any
pub fn trigger_oom(free_memory: u64) -> Option<i32> {
    if !is_enabled() {
        return None;
    }

    let killer = OOM_KILLER.lock();
    if free_memory >= killer.min_free_memory {
        return None; // Still have enough memory
    }

    // TODO: Get actual process list from scheduler
    // For now, use a placeholder
    let processes = get_process_list();
    
    if let Some(victim_pid) = killer.select_victim(&processes) {
        drop(killer); // Release lock before killing
        
        // Kill the victim process
        if kill_process(victim_pid) {
            OOM_KILLS.fetch_add(1, Ordering::SeqCst);
            return Some(victim_pid);
        }
    }

    None
}

/// Get list of processes (stub - to be integrated with process management)
fn get_process_list() -> Vec<ProcessOomInfo> {
    // TODO: Integrate with actual process management system
    // This is a placeholder that returns an empty list
    Vec::new()
}

/// Kill a process by PID
fn kill_process(pid: i32) -> bool {
    // TODO: Integrate with process management to actually kill the process
    // This would involve:
    // 1. Sending SIGKILL to the process
    // 2. Freeing all its memory
    // 3. Closing all its file descriptors
    // 4. Cleaning up any other resources
    
    let _ = pid;
    false // Stub implementation
}

/// Get OOM kill statistics
pub fn get_stats() -> u64 {
    OOM_KILLS.load(Ordering::Acquire)
}

/// Set minimum free memory threshold
pub fn set_min_free_memory(bytes: u64) {
    let mut killer = OOM_KILLER.lock();
    killer.min_free_memory = bytes;
}

/// Check if system is under memory pressure
pub fn is_under_memory_pressure() -> bool {
    let (total, _allocated, free) = crate::frame::get_stats();
    let total_bytes = total * 4096;
    let free_bytes = free * 4096;
    
    if total_bytes == 0 {
        return false;
    }

    // Consider under pressure if less than 10% memory free
    free_bytes < total_bytes / 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oom_score_kernel_process() {
        let proc = ProcessOomInfo {
            pid: 1,
            memory_usage: 100 * 1024 * 1024,
            oom_score_adj: 0,
            is_kernel: true,
            is_init: false,
        };
        assert_eq!(proc.oom_score(), i64::MIN);
    }

    #[test]
    fn test_oom_score_init_process() {
        let proc = ProcessOomInfo {
            pid: 1,
            memory_usage: 100 * 1024 * 1024,
            oom_score_adj: 0,
            is_kernel: false,
            is_init: true,
        };
        assert_eq!(proc.oom_score(), i64::MIN);
    }

    #[test]
    fn test_oom_score_regular_process() {
        let proc = ProcessOomInfo {
            pid: 100,
            memory_usage: 50 * 1024 * 1024, // 50 MB
            oom_score_adj: 0,
            is_kernel: false,
            is_init: false,
        };
        assert_eq!(proc.oom_score(), 50); // 50 MB
    }

    #[test]
    fn test_oom_score_with_adjustment() {
        let proc = ProcessOomInfo {
            pid: 100,
            memory_usage: 50 * 1024 * 1024, // 50 MB
            oom_score_adj: 100,
            is_kernel: false,
            is_init: false,
        };
        assert_eq!(proc.oom_score(), 150); // 50 + 100
    }

    #[test]
    fn test_victim_selection() {
        let killer = OomKiller::new();
        let processes = vec![
            ProcessOomInfo {
                pid: 1,
                memory_usage: 10 * 1024 * 1024,
                oom_score_adj: 0,
                is_kernel: false,
                is_init: true, // Init shouldn't be killed
            },
            ProcessOomInfo {
                pid: 100,
                memory_usage: 100 * 1024 * 1024,
                oom_score_adj: 0,
                is_kernel: false,
                is_init: false,
            },
            ProcessOomInfo {
                pid: 200,
                memory_usage: 50 * 1024 * 1024,
                oom_score_adj: 0,
                is_kernel: false,
                is_init: false,
            },
        ];

        let victim = killer.select_victim(&processes);
        assert_eq!(victim, Some(100)); // Process with most memory
    }
}

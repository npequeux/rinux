//! Time Management
//!
//! System time tracking and timer management.

pub mod timer;

pub use timer::{Timer, TimerId};

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// System uptime in milliseconds
static UPTIME_MS: AtomicU64 = AtomicU64::new(0);

/// Time subsystem initialized flag
static TIME_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize time subsystem
pub fn init() {
    if TIME_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    timer::init();

    TIME_INITIALIZED.store(true, Ordering::Release);
    crate::printk::printk("  Time subsystem initialized\n");
}

/// Check if time subsystem is initialized
pub fn is_initialized() -> bool {
    TIME_INITIALIZED.load(Ordering::Acquire)
}

/// Get system uptime in milliseconds
pub fn uptime_ms() -> u64 {
    UPTIME_MS.load(Ordering::Relaxed)
}

/// Increment uptime (called by timer interrupt)
pub fn tick(ms: u64) {
    UPTIME_MS.fetch_add(ms, Ordering::Relaxed);
    // Run timer processing with interrupts disabled to avoid deadlocks when
    // `timer::tick()` acquires locks such as `spin::Mutex` from interrupt context.
    x86_64::instructions::interrupts::without_interrupts(|| {
        timer::tick();
    });
}

/// Get system uptime in seconds
pub fn uptime_sec() -> u64 {
    uptime_ms() / 1000
}

/// Sleep for specified milliseconds
///
/// # Warning
///
/// This function uses a busy-wait implementation that wastes CPU cycles
/// and prevents other tasks from running. **Do not use for production code
/// or long sleep durations** as it will severely impact system performance.
///
/// This is a temporary implementation until proper scheduler-integrated
/// sleep/wake mechanisms are added.
pub fn sleep_ms(ms: u64) {
    let target = uptime_ms() + ms;
    while uptime_ms() < target {
        // Busy wait for now
        // TODO: Implement proper sleep with scheduler integration
        core::hint::spin_loop();
    }
}

/// System time structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemTime {
    /// Seconds since Unix epoch
    pub seconds: u64,
    /// Nanoseconds
    pub nanoseconds: u32,
}

impl SystemTime {
    /// Create a new system time
    pub const fn new(seconds: u64, nanoseconds: u32) -> Self {
        SystemTime {
            seconds,
            nanoseconds,
        }
    }

    /// Get current system time (stub - returns uptime)
    pub fn now() -> Self {
        let uptime_s = uptime_sec();
        SystemTime::new(uptime_s, 0)
    }
}

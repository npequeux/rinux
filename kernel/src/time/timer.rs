//! Timer Management
//!
//! Kernel timers for scheduling delayed tasks.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Timer ID type
pub type TimerId = usize;

/// Timer callback function
pub type TimerCallback = fn();

/// Timer structure
pub struct Timer {
    id: TimerId,
    expires_at: u64,
    callback: TimerCallback,
    periodic: bool,
    interval: u64,
}

impl Timer {
    /// Create a new timer
    pub fn new(id: TimerId, expires_at: u64, callback: TimerCallback) -> Self {
        Timer {
            id,
            expires_at,
            callback,
            periodic: false,
            interval: 0,
        }
    }

    /// Create a periodic timer
    pub fn new_periodic(id: TimerId, interval: u64, callback: TimerCallback) -> Self {
        let expires_at = super::uptime_ms() + interval;
        Timer {
            id,
            expires_at,
            callback,
            periodic: true,
            interval,
        }
    }

    /// Check if timer has expired
    pub fn has_expired(&self, current_time: u64) -> bool {
        current_time >= self.expires_at
    }

    /// Reset timer for next period
    pub fn reset(&mut self) {
        if self.periodic {
            self.expires_at += self.interval;
        }
    }
}

/// Global timer registry
static TIMERS: Mutex<BTreeMap<TimerId, Timer>> = Mutex::new(BTreeMap::new());
static NEXT_TIMER_ID: Mutex<TimerId> = Mutex::new(1);

/// Initialize timer subsystem
pub fn init() {
    let mut timers = TIMERS.lock();
    *timers = BTreeMap::new();
}

/// Create a new one-shot timer
pub fn create_timer(delay_ms: u64, callback: TimerCallback) -> Result<TimerId, ()> {
    let mut id_counter = NEXT_TIMER_ID.lock();
    let id = *id_counter;
    *id_counter += 1;
    drop(id_counter);

    let expires_at = super::uptime_ms() + delay_ms;
    let timer = Timer::new(id, expires_at, callback);

    let mut timers = TIMERS.lock();
    timers.insert(id, timer);
    Ok(id)
}

/// Create a new periodic timer
pub fn create_periodic_timer(interval_ms: u64, callback: TimerCallback) -> Result<TimerId, ()> {
    let mut id_counter = NEXT_TIMER_ID.lock();
    let id = *id_counter;
    *id_counter += 1;
    drop(id_counter);

    let timer = Timer::new_periodic(id, interval_ms, callback);

    let mut timers = TIMERS.lock();
    timers.insert(id, timer);
    Ok(id)
}

/// Cancel a timer
pub fn cancel_timer(timer_id: TimerId) -> Result<(), ()> {
    let mut timers = TIMERS.lock();
    if timers.remove(&timer_id).is_some() {
        Ok(())
    } else {
        Err(())
    }
}

/// Process timer ticks
pub fn tick() {
    let current_time = super::uptime_ms();
    let mut timers = TIMERS.lock();
    let mut expired_timers = Vec::new();

    // Find expired timers
    for (id, timer) in timers.iter() {
        if timer.has_expired(current_time) {
            expired_timers.push((*id, timer.callback, timer.periodic));
        }
    }

    // Process expired timers
    for (id, callback, periodic) in expired_timers {
        // Reset or remove timer
        if periodic {
            if let Some(timer) = timers.get_mut(&id) {
                timer.reset();
            }
        } else {
            timers.remove(&id);
        }

        // Call callback (drop lock first)
        drop(timers);
        callback();
        timers = TIMERS.lock();
    }
}

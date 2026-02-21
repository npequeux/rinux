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

    let mut next_id = NEXT_TIMER_ID.lock();
    *next_id = 1;
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
    
    // Collect expired timer callbacks
    let mut callbacks_to_execute = Vec::new();
    
    {
        let mut timers = TIMERS.lock();
        let mut to_remove = Vec::new();
        
        // Find expired timers and collect their callbacks
        for (id, timer) in timers.iter_mut() {
            if timer.has_expired(current_time) {
                callbacks_to_execute.push(timer.callback);
                
                // Reset or mark for removal
                if timer.periodic {
                    timer.reset();
                } else {
                    to_remove.push(*id);
                }
            }
        }
        
        // Remove one-shot timers
        for id in to_remove {
            timers.remove(&id);
        }
    } // Lock is dropped here
    
    // Execute callbacks without holding the lock
    for callback in callbacks_to_execute {
        callback();
    }
}

//! RISC-V Timers
//!
//! Timer management using rdtime instruction.

use crate::csr;
use crate::sbi;

/// Timer frequency (typically 10 MHz for RISC-V)
const TIMER_FREQ: u64 = 10_000_000;

/// Initialize timers
pub fn init() {
    let time = read_time();
    
    kernel::printk!("[RISCV] Timer initialized\n");
    kernel::printk!("  Current time: {}\n", time);
    kernel::printk!("  Frequency: {} Hz ({} MHz)\n", TIMER_FREQ, TIMER_FREQ / 1_000_000);
}

/// Read current time
pub fn read_time() -> u64 {
    csr::read_time()
}

/// Convert ticks to nanoseconds
pub fn ticks_to_ns(ticks: u64) -> u64 {
    (ticks * 1_000_000_000) / TIMER_FREQ
}

/// Convert nanoseconds to ticks
pub fn ns_to_ticks(ns: u64) -> u64 {
    (ns * TIMER_FREQ) / 1_000_000_000
}

/// Set timer for a specific time value
pub fn set_timer(time: u64) {
    sbi::set_timer(time);
}

/// Set timer to fire after a delay in nanoseconds
pub fn set_timer_ns(ns: u64) {
    let current = read_time();
    let ticks = ns_to_ticks(ns);
    set_timer(current + ticks);
}

/// Delay for a number of nanoseconds
pub fn delay_ns(ns: u64) {
    let ticks = ns_to_ticks(ns);
    let start = read_time();
    while read_time() - start < ticks {
        core::hint::spin_loop();
    }
}

/// Delay for a number of microseconds
pub fn delay_us(us: u64) {
    delay_ns(us * 1000);
}

/// Delay for a number of milliseconds
pub fn delay_ms(ms: u64) {
    delay_ns(ms * 1_000_000);
}

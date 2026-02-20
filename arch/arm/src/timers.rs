//! ARM64 Timers
//!
//! Generic Timer support for ARM64.

use core::arch::asm;

/// Read CNTFRQ_EL0 (Counter Frequency Register)
#[inline]
pub fn read_frequency() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, cntfrq_el0", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read CNTPCT_EL0 (Physical Counter)
#[inline]
pub fn read_physical_count() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, cntpct_el0", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read CNTVCT_EL0 (Virtual Counter)
#[inline]
pub fn read_virtual_count() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, cntvct_el0", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Initialize timers
pub fn init() {
    let freq = read_frequency();
    let count = read_physical_count();
    
    kernel::printk!("[ARM64] Generic Timer:\n");
    kernel::printk!("  Frequency: {} Hz ({} MHz)\n", freq, freq / 1_000_000);
    kernel::printk!("  Count: {}\n", count);
}

/// Convert ticks to nanoseconds
pub fn ticks_to_ns(ticks: u64) -> u64 {
    let freq = read_frequency();
    if freq == 0 {
        return 0;
    }
    (ticks * 1_000_000_000) / freq
}

/// Convert nanoseconds to ticks
pub fn ns_to_ticks(ns: u64) -> u64 {
    let freq = read_frequency();
    if freq == 0 {
        return 0;
    }
    (ns * freq) / 1_000_000_000
}

/// Delay for a number of nanoseconds
pub fn delay_ns(ns: u64) {
    let ticks = ns_to_ticks(ns);
    let start = read_physical_count();
    while read_physical_count() - start < ticks {
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

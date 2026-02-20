//! High Precision Timers
//!
//! Support for TSC (Time Stamp Counter) and HPET (High Precision Event Timer).

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};
use crate::long_mode::rdmsr;

/// TSC frequency in Hz (calibrated at runtime)
static mut TSC_FREQUENCY: u64 = 0;

/// HPET base address
static mut HPET_BASE: Option<u64> = None;

/// HPET register offsets
mod hpet_reg {
    pub const GENERAL_CAPS: usize = 0x000;
    pub const GENERAL_CONFIG: usize = 0x010;
    pub const GENERAL_INT_STATUS: usize = 0x020;
    pub const MAIN_COUNTER: usize = 0x0F0;
    pub const TIMER0_CONFIG: usize = 0x100;
    pub const TIMER0_COMPARATOR: usize = 0x108;
}

/// Read Time Stamp Counter
#[inline]
pub fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack)
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Read Time Stamp Counter and Processor ID (serializing)
#[inline]
pub fn rdtscp() -> (u64, u32) {
    let low: u32;
    let high: u32;
    let aux: u32;
    unsafe {
        asm!(
            "rdtscp",
            out("eax") low,
            out("edx") high,
            out("ecx") aux,
            options(nomem, nostack)
        );
    }
    (((high as u64) << 32) | (low as u64), aux)
}

/// Check if TSC is available
pub fn has_tsc() -> bool {
    use crate::cpu::cpuid;
    let (_, _, _, edx) = cpuid(1);
    (edx & (1 << 4)) != 0
}

/// Check if TSC is invariant (constant rate)
pub fn has_invariant_tsc() -> bool {
    use crate::cpu::cpuid;
    let (_, _, _, edx) = cpuid(0x80000007);
    (edx & (1 << 8)) != 0
}

/// Check if RDTSCP is available
pub fn has_rdtscp() -> bool {
    use crate::cpu::cpuid;
    let (_, _, _, edx) = cpuid(0x80000001);
    (edx & (1 << 27)) != 0
}

/// Get TSC frequency from CPUID (Intel only, may not be available)
fn get_tsc_frequency_cpuid() -> Option<u64> {
    use crate::cpu::cpuid;
    
    // Check if leaf 0x15 is available
    let (max_leaf, _, _, _) = cpuid(0);
    if max_leaf < 0x15 {
        return None;
    }

    let (eax, ebx, ecx, _) = cpuid(0x15);
    
    if eax == 0 || ebx == 0 {
        return None;
    }

    // Crystal clock frequency
    let crystal_freq = if ecx != 0 {
        ecx as u64
    } else {
        // Default crystal frequency for some Intel CPUs
        24_000_000 // 24 MHz
    };

    // TSC frequency = (crystal_freq * EBX) / EAX
    Some((crystal_freq * ebx as u64) / eax as u64)
}

/// Calibrate TSC using PIT (Programmable Interval Timer)
fn calibrate_tsc_pit() -> u64 {
    use crate::io::{inb, outb};
    
    const PIT_FREQ: u64 = 1193182; // PIT frequency in Hz
    const CALIBRATION_MS: u64 = 50; // Calibrate for 50ms
    
    // Calculate PIT ticks for calibration period
    let pit_ticks = (PIT_FREQ * CALIBRATION_MS) / 1000;
    
    unsafe {
        // Program PIT channel 2 for one-shot mode
        outb(0x43, 0xB0); // Channel 2, mode 0
        outb(0x42, (pit_ticks & 0xFF) as u8);
        outb(0x42, ((pit_ticks >> 8) & 0xFF) as u8);
        
        // Start PIT
        let gate = inb(0x61);
        outb(0x61, (gate & 0xFD) | 1);
        
        // Read TSC before
        let tsc_start = rdtsc();
        
        // Wait for PIT to complete
        loop {
            let status = inb(0x61);
            if (status & 0x20) != 0 {
                break;
            }
        }
        
        // Read TSC after
        let tsc_end = rdtsc();
        
        // Calculate frequency
        let tsc_ticks = tsc_end - tsc_start;
        (tsc_ticks * 1000) / CALIBRATION_MS
    }
}

/// Initialize TSC
pub fn init_tsc() {
    if !has_tsc() {
        kernel::printk!("[TSC] Time Stamp Counter not available\n");
        return;
    }

    kernel::printk!("[TSC] Initializing Time Stamp Counter...\n");
    
    // Try to get frequency from CPUID
    let freq = get_tsc_frequency_cpuid().unwrap_or_else(|| {
        kernel::printk!("[TSC] Calibrating using PIT...\n");
        calibrate_tsc_pit()
    });

    unsafe {
        TSC_FREQUENCY = freq;
    }

    kernel::printk!("[TSC] Frequency: {} Hz ({} MHz)\n", freq, freq / 1_000_000);
    kernel::printk!("[TSC] Invariant: {}\n", has_invariant_tsc());
    kernel::printk!("[TSC] RDTSCP: {}\n", has_rdtscp());
}

/// Get TSC frequency
pub fn get_tsc_frequency() -> u64 {
    unsafe { TSC_FREQUENCY }
}

/// Convert TSC ticks to nanoseconds
pub fn tsc_to_ns(ticks: u64) -> u64 {
    let freq = get_tsc_frequency();
    if freq == 0 {
        return 0;
    }
    (ticks * 1_000_000_000) / freq
}

/// Convert nanoseconds to TSC ticks
pub fn ns_to_tsc(ns: u64) -> u64 {
    let freq = get_tsc_frequency();
    if freq == 0 {
        return 0;
    }
    (ns * freq) / 1_000_000_000
}

/// Read HPET register
unsafe fn read_hpet(offset: usize) -> u64 {
    if let Some(base) = HPET_BASE {
        let addr = (base + offset as u64) as *const u64;
        read_volatile(addr)
    } else {
        0
    }
}

/// Write HPET register
unsafe fn write_hpet(offset: usize, value: u64) {
    if let Some(base) = HPET_BASE {
        let addr = (base + offset as u64) as *mut u64;
        write_volatile(addr, value);
    }
}

/// Find HPET via ACPI tables
fn find_hpet_base() -> Option<u64> {
    // TODO: Parse ACPI HPET table
    // For now, try the common address
    let common_addr = 0xFED00000u64;
    
    // Verify HPET is present by reading capabilities
    unsafe {
        let caps = read_volatile(common_addr as *const u64);
        if caps != 0 && caps != 0xFFFFFFFFFFFFFFFF {
            return Some(common_addr);
        }
    }
    
    None
}

/// Get HPET frequency (in femtoseconds per tick)
pub fn get_hpet_period() -> Option<u64> {
    unsafe {
        if HPET_BASE.is_some() {
            let caps = read_hpet(hpet_reg::GENERAL_CAPS);
            Some(caps >> 32) // Period in femtoseconds
        } else {
            None
        }
    }
}

/// Read HPET main counter
pub fn read_hpet_counter() -> u64 {
    unsafe { read_hpet(hpet_reg::MAIN_COUNTER) }
}

/// Initialize HPET
pub fn init_hpet() {
    kernel::printk!("[HPET] Searching for High Precision Event Timer...\n");
    
    if let Some(base) = find_hpet_base() {
        unsafe {
            HPET_BASE = Some(base);
            
            // Read capabilities
            let caps = read_hpet(hpet_reg::GENERAL_CAPS);
            let vendor_id = (caps >> 16) & 0xFFFF;
            let num_timers = ((caps >> 8) & 0x1F) + 1;
            let period = caps >> 32;
            
            kernel::printk!("[HPET] Found at {:#x}\n", base);
            kernel::printk!("[HPET] Vendor ID: {:#x}\n", vendor_id);
            kernel::printk!("[HPET] Timers: {}\n", num_timers);
            kernel::printk!("[HPET] Period: {} fs\n", period);
            kernel::printk!("[HPET] Frequency: {} Hz\n", 1_000_000_000_000_000u64 / period);
            
            // Enable HPET
            let config = read_hpet(hpet_reg::GENERAL_CONFIG);
            write_hpet(hpet_reg::GENERAL_CONFIG, config | 1);
            
            kernel::printk!("[HPET] Enabled\n");
        }
    } else {
        kernel::printk!("[HPET] Not found\n");
    }
}

/// Initialize all timers
pub fn init() {
    kernel::printk!("[TIMERS] Initializing high-precision timers...\n");
    init_tsc();
    init_hpet();
    kernel::printk!("[TIMERS] Initialization complete\n");
}

/// Busy wait for a number of nanoseconds using TSC
pub fn delay_ns(ns: u64) {
    let ticks = ns_to_tsc(ns);
    let start = rdtsc();
    while rdtsc() - start < ticks {
        core::hint::spin_loop();
    }
}

/// Busy wait for a number of microseconds using TSC
pub fn delay_us(us: u64) {
    delay_ns(us * 1000);
}

/// Busy wait for a number of milliseconds using TSC
pub fn delay_ms(ms: u64) {
    delay_ns(ms * 1_000_000);
}

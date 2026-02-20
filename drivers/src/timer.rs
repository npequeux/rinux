//! Timer Driver
//!
//! Programmable Interval Timer (PIT) driver for x86.

use rinux_arch_x86::io::{inb, outb};
use spin::Mutex;

/// PIT frequency (1.193182 MHz)
const PIT_FREQUENCY: u32 = 1193182;

/// PIT command port
const PIT_COMMAND: u16 = 0x43;
/// PIT channel 0 data port
const PIT_CHANNEL_0: u16 = 0x40;

/// Global timer state
static TIMER: Mutex<Timer> = Mutex::new(Timer {
    ticks: 0,
    frequency: 0,
});

/// Timer structure
pub struct Timer {
    ticks: u64,
    frequency: u32,
}

impl Timer {
    /// Initialize the PIT
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn init(&mut self, frequency: u32) {
        if frequency == 0 || frequency > PIT_FREQUENCY {
            rinux_kernel::printk::printk("  [ERROR] Invalid PIT frequency: ");
            rinux_kernel::printk::printk("must be between 1 and 1193182 Hz\n");
            return;
        }

        self.frequency = frequency;
        let divisor = PIT_FREQUENCY / frequency;

        // Send command: channel 0, access mode lo/hi byte, rate generator
        outb(PIT_COMMAND, 0x36);

        // Send divisor
        outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);
    }

    /// Handle timer interrupt
    fn tick(&mut self) {
        self.ticks = self.ticks.wrapping_add(1);
    }

    /// Get current tick count
    fn get_ticks(&self) -> u64 {
        self.ticks
    }

    /// Get timer frequency
    fn get_frequency(&self) -> u32 {
        self.frequency
    }
}

/// Initialize timer with specified frequency (Hz)
///
/// Common values: 100 Hz (10ms), 1000 Hz (1ms)
pub fn init(frequency: u32) {
    let mut timer = TIMER.lock();
    unsafe {
        timer.init(frequency);
    }
}

/// Handle timer interrupt (call from IRQ handler)
pub fn tick() {
    let mut timer = TIMER.lock();
    timer.tick();
}

/// Get current tick count
pub fn get_ticks() -> u64 {
    let timer = TIMER.lock();
    timer.get_ticks()
}

/// Get timer frequency
pub fn get_frequency() -> u32 {
    let timer = TIMER.lock();
    timer.get_frequency()
}

/// Get uptime in milliseconds (approximate)
pub fn get_uptime_ms() -> u64 {
    let timer = TIMER.lock();
    if timer.frequency == 0 {
        return 0;
    }
    (timer.ticks * 1000) / timer.frequency as u64
}

/// Get uptime in seconds (approximate)
pub fn get_uptime_secs() -> u64 {
    let timer = TIMER.lock();
    if timer.frequency == 0 {
        return 0;
    }
    timer.ticks / timer.frequency as u64
}

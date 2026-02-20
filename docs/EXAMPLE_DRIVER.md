# Example: Adding a New Hardware Driver to Rinux

This document walks through adding a simple hardware driver to demonstrate the process.

## Scenario

We'll add a simple timer device driver for the 8254 Programmable Interval Timer (PIT), a standard PC timer chip.

## Step 1: Research the Hardware

### 8254 PIT Specifications

- **I/O Ports:**
  - 0x40: Channel 0 data port
  - 0x41: Channel 1 data port (not used)
  - 0x42: Channel 2 data port (PC speaker)
  - 0x43: Mode/Command register

- **Base Frequency:** 1.193182 MHz (1193182 Hz)

- **Modes:**
  - Mode 0: Interrupt on terminal count
  - Mode 2: Rate generator (what we'll use)
  - Mode 3: Square wave generator

## Step 2: Create the Driver File

Create `drivers/src/timer_pit.rs`:

```rust
//! 8254 Programmable Interval Timer (PIT) Driver
//!
//! The PIT is a standard PC hardware component that generates periodic
//! timer interrupts.

use core::sync::atomic::{AtomicU64, Ordering};
use rinux_arch_x86::io::{inb, outb};

/// PIT I/O ports
const PIT_CHANNEL0: u16 = 0x40; // Channel 0 data port (system timer)
const PIT_CHANNEL1: u16 = 0x41; // Channel 1 data port (RAM refresh)
const PIT_CHANNEL2: u16 = 0x42; // Channel 2 data port (PC speaker)
const PIT_COMMAND: u16 = 0x43;  // Mode/Command register

/// PIT base frequency (Hz)
const PIT_FREQUENCY: u32 = 1193182;

/// Target timer frequency (Hz) - 100 Hz = 10ms ticks
const TIMER_FREQUENCY: u32 = 100;

/// System tick counter
static TICKS: AtomicU64 = AtomicU64::new(0);

/// PIT timer driver
pub struct PitTimer {
    frequency: u32,
    initialized: bool,
}

impl PitTimer {
    /// Create a new PIT timer driver
    pub const fn new() -> Self {
        Self {
            frequency: TIMER_FREQUENCY,
            initialized: false,
        }
    }

    /// Initialize the PIT timer
    ///
    /// # Arguments
    ///
    /// * `frequency` - Desired timer frequency in Hz (10-1193182)
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error message on failure.
    pub fn init(&mut self, frequency: u32) -> Result<(), &'static str> {
        if self.initialized {
            return Ok(());
        }

        if frequency == 0 {
            return Err("Frequency must be greater than 0");
        }

        if frequency > PIT_FREQUENCY {
            return Err("Frequency too high");
        }

        self.frequency = frequency;

        // Calculate the divisor
        let divisor = PIT_FREQUENCY / frequency;
        if divisor > 65535 {
            return Err("Frequency too low (divisor overflow)");
        }

        unsafe {
            // Send the command byte to the PIT
            // Channel 0, lobyte/hibyte, rate generator, binary mode
            // 0x36 = 0b00110110
            //   00    - Select channel 0
            //   11    - Access mode: lobyte/hibyte
            //   011   - Operating mode 3 (square wave generator)
            //   0     - Binary mode (not BCD)
            outb(PIT_COMMAND, 0x36);

            // Send divisor
            let low = (divisor & 0xFF) as u8;
            let high = ((divisor >> 8) & 0xFF) as u8;
            
            outb(PIT_CHANNEL0, low);
            outb(PIT_CHANNEL0, high);
        }

        self.initialized = true;
        Ok(())
    }

    /// Get the configured frequency
    pub fn frequency(&self) -> u32 {
        self.frequency
    }

    /// Check if the timer is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Handle timer tick (called from interrupt handler)
    pub fn tick(&self) {
        TICKS.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current tick count
    pub fn ticks() -> u64 {
        TICKS.load(Ordering::Relaxed)
    }

    /// Get elapsed time in milliseconds
    pub fn uptime_ms(&self) -> u64 {
        if self.frequency == 0 {
            return 0;
        }
        
        let ticks = Self::ticks();
        (ticks * 1000) / self.frequency as u64
    }

    /// Sleep for approximately the given number of milliseconds
    ///
    /// Note: This is a busy-wait and will consume CPU.
    pub fn sleep_ms(&self, ms: u64) {
        let start = Self::ticks();
        let target_ticks = (ms * self.frequency as u64) / 1000;
        
        while (Self::ticks() - start) < target_ticks {
            core::hint::spin_loop();
        }
    }
}

/// Global PIT timer instance
static mut PIT: PitTimer = PitTimer::new();

/// Initialize the PIT timer
pub fn init() {
    rinux_kernel::printk::printk("Initializing PIT timer...\n");

    #[allow(static_mut_refs)]
    unsafe {
        match PIT.init(TIMER_FREQUENCY) {
            Ok(()) => {
                rinux_kernel::printk::printk("PIT: Initialized at ");
                // TODO: Format frequency
                rinux_kernel::printk::printk(" Hz\n");
            }
            Err(e) => {
                rinux_kernel::printk::printk("PIT: Initialization failed: ");
                rinux_kernel::printk::printk(e);
                rinux_kernel::printk::printk("\n");
            }
        }
    }
}

/// Handle timer interrupt
///
/// This should be called from the IRQ0 interrupt handler.
pub fn handle_interrupt() {
    #[allow(static_mut_refs)]
    unsafe {
        PIT.tick();
    }
}

/// Get current system ticks
pub fn ticks() -> u64 {
    PitTimer::ticks()
}

/// Get system uptime in milliseconds
pub fn uptime_ms() -> u64 {
    #[allow(static_mut_refs)]
    unsafe {
        PIT.uptime_ms()
    }
}

/// Sleep for the given number of milliseconds
pub fn sleep_ms(ms: u64) {
    #[allow(static_mut_refs)]
    unsafe {
        PIT.sleep_ms(ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = PitTimer::new();
        assert!(!timer.is_initialized());
        assert_eq!(timer.frequency(), TIMER_FREQUENCY);
    }

    #[test]
    fn test_invalid_frequency() {
        let mut timer = PitTimer::new();
        assert!(timer.init(0).is_err());
        assert!(timer.init(PIT_FREQUENCY + 1).is_err());
    }

    #[test]
    fn test_frequency_calculation() {
        // 100 Hz should work
        let mut timer = PitTimer::new();
        assert!(timer.init(100).is_ok());
        
        // 1000 Hz should work
        let mut timer = PitTimer::new();
        assert!(timer.init(1000).is_ok());
    }
}
```

## Step 3: Register the Driver

Update `drivers/src/lib.rs`:

```rust
// Add to module declarations
pub mod timer_pit;

// Add to init() function
pub fn init() {
    serial::init();
    keyboard::init();
    vga::init();

    // Initialize ACPI for power management and system info
    acpi::init();

    // Initialize PCI bus
    pci::init();

    // Initialize PIT timer
    timer_pit::init(); // <-- Add this line

    // Initialize graphics subsystem
    graphics::init();

    // ... rest of initialization ...
}
```

## Step 4: Build and Test

```bash
# Build the kernel
make build

# Run in QEMU
make run
```

You should see in the output:
```
Initializing PIT timer...
PIT: Initialized at 100 Hz
```

## Step 5: Use the Timer

Now other code can use the timer:

```rust
use rinux_drivers::timer_pit;

// Get current ticks
let ticks = timer_pit::ticks();

// Get uptime in milliseconds
let uptime = timer_pit::uptime_ms();
rinux_kernel::printk::printk("System uptime: ");
// TODO: Format uptime
rinux_kernel::printk::printk(" ms\n");

// Sleep for 100ms (busy-wait)
timer_pit::sleep_ms(100);
```

## What This Example Demonstrates

### Good Practices

1. **Clear Documentation**: Module and function docs explain what the driver does
2. **Type Safety**: Uses Rust types to prevent errors
3. **Error Handling**: Returns `Result<>` for operations that can fail
4. **Safe API**: Unsafe code is contained in specific functions
5. **Atomic Operations**: Uses `AtomicU64` for thread-safe tick counter
6. **Testing**: Includes unit tests
7. **Constants**: Uses named constants instead of magic numbers

### Hardware Access

1. **Port I/O**: Uses `inb`/`outb` for hardware access
2. **Hardware Specification**: Follows 8254 PIT datasheet
3. **Initialization**: Proper hardware setup sequence
4. **Frequency Calculation**: Correct divisor calculation

### Integration

1. **Module Organization**: Follows Rinux driver structure
2. **Initialization**: Integrates with driver init sequence
3. **Public API**: Provides clean interface for other code
4. **Global State**: Uses static mut with proper safety annotations

## Next Steps

### Enhance the Driver

1. **Add Interrupt Handler**: Connect to IRQ0 for automatic ticks
2. **Add Calibration**: More accurate frequency calibration
3. **Add Other Channels**: Support PIT channels 1 and 2
4. **High Resolution**: Add support for other timer sources (TSC, HPET)

### Use in Kernel

1. **Scheduler**: Use for task scheduling time slices
2. **Timeouts**: Implement timeout mechanisms
3. **Profiling**: Add performance profiling support
4. **Sleep**: Implement proper sleep (not busy-wait)

## Common Pitfalls to Avoid

### 1. Incorrect Divisor Calculation

```rust
// ❌ Wrong: Integer division loses precision
let divisor = 1193182 / frequency;

// ✅ Correct: Check for overflow first
if frequency > PIT_FREQUENCY {
    return Err("Frequency too high");
}
let divisor = PIT_FREQUENCY / frequency;
```

### 2. Wrong Command Byte

```rust
// ❌ Wrong: Incorrect mode bits
outb(PIT_COMMAND, 0x34);  // Wrong mode

// ✅ Correct: Mode 3 (square wave)
outb(PIT_COMMAND, 0x36);  // Correct mode
```

### 3. Byte Order

```rust
// ❌ Wrong: Send high byte first
outb(PIT_CHANNEL0, high);
outb(PIT_CHANNEL0, low);

// ✅ Correct: Send low byte first
outb(PIT_CHANNEL0, low);
outb(PIT_CHANNEL0, high);
```

### 4. Not Checking Initialization

```rust
// ❌ Bad: Using uninitialized driver
pub fn uptime_ms(&self) -> u64 {
    let ticks = Self::ticks();
    (ticks * 1000) / self.frequency as u64  // Division by zero if not initialized!
}

// ✅ Good: Check initialization
pub fn uptime_ms(&self) -> u64 {
    if self.frequency == 0 {
        return 0;
    }
    let ticks = Self::ticks();
    (ticks * 1000) / self.frequency as u64
}
```

## Resources

- [8254 PIT Datasheet](https://pdos.csail.mit.edu/6.828/2014/readings/hardware/8254.pdf)
- [OSDev PIT Wiki](https://wiki.osdev.org/Programmable_Interval_Timer)
- [Intel Manual](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)

## Summary

This example shows:
- ✅ How to create a simple driver
- ✅ How to access hardware (port I/O)
- ✅ How to integrate with Rinux
- ✅ Best practices for driver development
- ✅ Testing and validation

Use this as a template for your own drivers!

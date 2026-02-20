# Driver Development Guide for Rinux

## Introduction

This guide provides detailed information for developing device drivers in the Rinux operating system kernel. It covers the driver model, best practices, and examples.

## Prerequisites

Before writing drivers for Rinux, you should be familiar with:

- **Rust Programming**: Rinux is written entirely in Rust
- **OS Concepts**: Interrupts, memory management, I/O
- **Hardware Specifications**: Read your device's datasheet
- **x86_64 Architecture**: Current target platform

## Driver Architecture

### Kernel Space Only

All Rinux drivers run in kernel space with full system privileges. There is currently:
- No user space
- No privilege separation
- No driver isolation

**Security Note**: Buggy drivers can crash the entire system.

### No Standard Library

Drivers are `#![no_std]` environments and cannot use:
- Standard library (`std`)
- Heap allocation (limited)
- Threading (not yet implemented)
- File I/O (no filesystem)

Available:
- Core library (`core`)
- Spin locks (`spin` crate)
- Basic memory allocation
- Hardware I/O ports

## Project Structure

### Driver Location

Place your driver in the appropriate location:

```
drivers/src/
├── block/          # Block devices (future)
├── char/           # Character devices  
├── graphics/       # Graphics drivers
├── net/            # Network drivers (future)
├── usb/            # USB drivers
└── your_driver.rs  # Standalone driver
```

### Module Declaration

Add your driver to `drivers/src/lib.rs`:

```rust
#![no_std]

// Existing drivers
pub mod vga;
pub mod serial;
// ... others ...

// Your new driver
pub mod your_driver;

/// Initialize all drivers
pub fn init() {
    // ... existing init code ...
    
    // Initialize your driver
    your_driver::init();
}
```

## Basic Driver Template

### Minimal Driver

```rust
//! Your Driver Name
//!
//! Description of what this driver does.

use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// Driver state structure
pub struct YourDriver {
    initialized: bool,
    // Add your driver fields here
}

impl YourDriver {
    /// Create a new driver instance
    pub const fn new() -> Self {
        Self {
            initialized: false,
        }
    }
    
    /// Initialize the driver
    pub fn init(&mut self) -> Result<(), &'static str> {
        if self.initialized {
            return Ok(());
        }
        
        // Perform hardware initialization
        rinux_kernel::printk::printk("YourDriver: Initializing...\n");
        
        // TODO: Your initialization code here
        
        self.initialized = true;
        rinux_kernel::printk::printk("YourDriver: Initialized\n");
        Ok(())
    }
    
    /// Check if driver is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Global driver instance protected by a mutex
static DRIVER: Mutex<YourDriver> = Mutex::new(YourDriver::new());

/// Module initialization function
pub fn init() {
    let mut driver = DRIVER.lock();
    match driver.init() {
        Ok(()) => {},
        Err(e) => {
            rinux_kernel::printk::printk("YourDriver: Initialization failed: ");
            rinux_kernel::printk::printk(e);
            rinux_kernel::printk::printk("\n");
        }
    }
}

/// Get driver instance
pub fn get() -> &'static Mutex<YourDriver> {
    &DRIVER
}
```

## Hardware Access

### Port I/O (x86)

For x86 I/O port access:

```rust
use rinux_arch_x86::io::{inb, outb, inw, outw, inl, outl};

/// Read a byte from I/O port
unsafe fn read_port_u8(port: u16) -> u8 {
    inb(port)
}

/// Write a byte to I/O port
unsafe fn write_port_u8(port: u16, value: u8) {
    outb(port, value)
}

/// Read a word (16-bit) from I/O port
unsafe fn read_port_u16(port: u16) -> u16 {
    inw(port)
}

/// Write a word to I/O port
unsafe fn write_port_u16(port: u16, value: u16) {
    outw(port, value)
}

/// Read a double word (32-bit) from I/O port
unsafe fn read_port_u32(port: u16) -> u32 {
    inl(port)
}

/// Write a double word to I/O port
unsafe fn write_port_u32(port: u16, value: u32) {
    outl(port, value)
}
```

**Safety**: Port I/O is `unsafe` because:
- Wrong port can hang system
- Wrong value can damage hardware (rare)
- No bounds checking

### Memory-Mapped I/O (MMIO)

For memory-mapped devices:

```rust
use core::ptr::{read_volatile, write_volatile};

/// Read from MMIO address
unsafe fn mmio_read<T>(addr: usize) -> T 
where
    T: Copy,
{
    let ptr = addr as *const T;
    read_volatile(ptr)
}

/// Write to MMIO address
unsafe fn mmio_write<T>(addr: usize, value: T) 
where
    T: Copy,
{
    let ptr = addr as *mut T;
    write_volatile(ptr, value);
}

// Example usage
pub struct MmioDevice {
    base_addr: usize,
}

impl MmioDevice {
    /// Read 32-bit register
    unsafe fn read_reg(&self, offset: usize) -> u32 {
        mmio_read(self.base_addr + offset)
    }
    
    /// Write 32-bit register
    unsafe fn write_reg(&self, offset: usize, value: u32) {
        mmio_write(self.base_addr + offset, value);
    }
}
```

**Safety**: MMIO is `unsafe` because:
- Invalid address can page fault
- Wrong value can break device
- Concurrent access can corrupt state

### PCI Configuration Space

Accessing PCI devices:

```rust
use crate::pci::{self, PciDevice};

pub fn init() {
    let scanner = pci::scanner();
    
    // Iterate over all PCI devices
    for i in 0..scanner.device_count() {
        if let Some(device) = scanner.get_device(i) {
            // Check vendor and device ID
            if device.vendor_id == 0x8086 && device.device_id == 0x100E {
                // Found Intel E1000 network card
                init_e1000(device);
            }
        }
    }
}

fn init_e1000(device: &PciDevice) {
    rinux_kernel::printk::printk("Initializing E1000 network card\n");
    
    // Enable bus mastering
    device.enable_bus_mastering();
    
    // Enable memory space
    device.enable_memory_space();
    
    // Read BAR0 (Memory mapped I/O base address)
    let mmio_base = device.bars[0] & !0xF; // Mask out flags
    
    // Read interrupt line
    let irq = device.interrupt_line();
    
    // TODO: Initialize device with MMIO base and IRQ
}
```

## Synchronization

### Spinlocks

Use spinlocks for mutual exclusion:

```rust
use spin::Mutex;

static DEVICE: Mutex<Option<MyDevice>> = Mutex::new(None);

pub fn init() {
    let mut device = DEVICE.lock();
    *device = Some(MyDevice::new());
}

pub fn do_operation() -> Result<(), &'static str> {
    let device = DEVICE.lock();
    match device.as_ref() {
        Some(dev) => dev.operation(),
        None => Err("Device not initialized"),
    }
}
```

**Important**: Spinlocks spin-wait. Don't hold them long:
- No long computations inside lock
- No waiting for I/O inside lock
- No nested locks (deadlock risk)

### Atomic Operations

For simple flags and counters:

```rust
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static DEVICE_READY: AtomicBool = AtomicBool::new(false);
static REQUEST_COUNT: AtomicU32 = AtomicU32::new(0);

pub fn mark_ready() {
    DEVICE_READY.store(true, Ordering::Release);
}

pub fn is_ready() -> bool {
    DEVICE_READY.load(Ordering::Acquire)
}

pub fn increment_requests() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
}
```

## Error Handling

### Result Types

Always use `Result` for fallible operations:

```rust
pub fn init(&mut self) -> Result<(), &'static str> {
    // Try to initialize
    if !self.detect_hardware() {
        return Err("Hardware not found");
    }
    
    if !self.reset_device() {
        return Err("Device reset failed");
    }
    
    Ok(())
}

pub fn read(&self, addr: u32) -> Result<u8, &'static str> {
    if !self.initialized {
        return Err("Device not initialized");
    }
    
    if addr >= self.size {
        return Err("Address out of bounds");
    }
    
    Ok(unsafe { self.read_unchecked(addr) })
}
```

### Error Messages

Keep error messages concise and informative:

```rust
✅ Good:
Err("Hardware not found")
Err("Invalid register address")
Err("Timeout waiting for device")

❌ Bad:
Err("Error")
Err("Something went wrong")
Err("Failed")
```

### Logging

Use `printk` for debug output:

```rust
use rinux_kernel::printk::printk;

pub fn init() {
    printk("[MyDriver] Initializing...\n");
    
    if let Some(version) = self.detect_version() {
        printk("[MyDriver] Hardware version: ");
        // TODO: Format version number
        printk("\n");
    } else {
        printk("[MyDriver] Warning: Could not detect version\n");
    }
    
    printk("[MyDriver] Initialization complete\n");
}
```

## Hardware-Specific Patterns

### Serial Devices

```rust
pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    const DATA: u16 = 0;          // Data register
    const INT_ENABLE: u16 = 1;    // Interrupt enable
    const FIFO_CTRL: u16 = 2;     // FIFO control
    const LINE_CTRL: u16 = 3;     // Line control
    const MODEM_CTRL: u16 = 4;    // Modem control
    const LINE_STATUS: u16 = 5;   // Line status
    
    pub fn new(base: u16) -> Self {
        Self { base }
    }
    
    pub fn init(&self) {
        unsafe {
            // Disable interrupts
            outb(self.base + Self::INT_ENABLE, 0x00);
            
            // Enable DLAB (set baud rate divisor)
            outb(self.base + Self::LINE_CTRL, 0x80);
            
            // Set divisor to 3 (38400 baud)
            outb(self.base + Self::DATA, 0x03);
            outb(self.base + Self::INT_ENABLE, 0x00);
            
            // 8 bits, no parity, one stop bit
            outb(self.base + Self::LINE_CTRL, 0x03);
            
            // Enable FIFO, clear, 14-byte threshold
            outb(self.base + Self::FIFO_CTRL, 0xC7);
            
            // Enable IRQs, RTS/DSR set
            outb(self.base + Self::MODEM_CTRL, 0x0B);
        }
    }
    
    pub fn send(&self, byte: u8) {
        unsafe {
            // Wait for transmit buffer to be empty
            while (inb(self.base + Self::LINE_STATUS) & 0x20) == 0 {
                core::hint::spin_loop();
            }
            
            outb(self.base + Self::DATA, byte);
        }
    }
    
    pub fn receive(&self) -> Option<u8> {
        unsafe {
            // Check if data is available
            if (inb(self.base + Self::LINE_STATUS) & 0x01) != 0 {
                Some(inb(self.base + Self::DATA))
            } else {
                None
            }
        }
    }
}
```

### Block Devices

```rust
pub struct BlockDevice {
    block_size: usize,
    total_blocks: u64,
}

impl BlockDevice {
    /// Read a block
    pub fn read_block(&self, block: u64, buffer: &mut [u8]) -> Result<(), &'static str> {
        if buffer.len() < self.block_size {
            return Err("Buffer too small");
        }
        
        if block >= self.total_blocks {
            return Err("Block out of range");
        }
        
        // TODO: Perform actual hardware read
        
        Ok(())
    }
    
    /// Write a block
    pub fn write_block(&mut self, block: u64, buffer: &[u8]) -> Result<(), &'static str> {
        if buffer.len() < self.block_size {
            return Err("Buffer too small");
        }
        
        if block >= self.total_blocks {
            return Err("Block out of range");
        }
        
        // TODO: Perform actual hardware write
        
        Ok(())
    }
}
```

### Network Devices

```rust
pub struct NetworkDevice {
    mac_address: [u8; 6],
}

impl NetworkDevice {
    /// Send a packet
    pub fn send(&mut self, packet: &[u8]) -> Result<(), &'static str> {
        if packet.len() > 1514 {  // Max Ethernet frame
            return Err("Packet too large");
        }
        
        // TODO: Queue packet for transmission
        
        Ok(())
    }
    
    /// Receive a packet
    pub fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Read received packet
        Ok(0)
    }
    
    /// Get MAC address
    pub fn mac_address(&self) -> [u8; 6] {
        self.mac_address
    }
}
```

## Best Practices

### 1. Document Everything

```rust
//! Block device driver for ATA/IDE controllers
//!
//! Supports PIO mode 0-4 and basic DMA.
//!
//! # Safety
//!
//! This driver performs direct hardware I/O and must be used carefully.
//! Incorrect usage can corrupt data or hang the system.

/// ATA command codes
mod ata_cmd {
    pub const READ_SECTORS: u8 = 0x20;
    pub const WRITE_SECTORS: u8 = 0x30;
    pub const IDENTIFY: u8 = 0xEC;
}

/// ATA device structure
///
/// # Fields
///
/// * `base` - Base I/O port (0x1F0 or 0x170)
/// * `ctrl` - Control port (base + 0x206)
/// * `master` - True if master, false if slave
pub struct AtaDevice {
    base: u16,
    ctrl: u16,
    master: bool,
}
```

### 2. Use Type Safety

```rust
// ❌ Bad: Using raw integers
pub fn set_register(reg: u8, value: u32) { }

// ✅ Good: Using enums
#[repr(u8)]
pub enum Register {
    Control = 0x00,
    Status = 0x01,
    Data = 0x02,
}

pub fn set_register(reg: Register, value: u32) { }
```

### 3. Validate Inputs

```rust
pub fn read(&self, addr: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    // Check alignment
    if addr % 4 != 0 {
        return Err("Address must be 4-byte aligned");
    }
    
    // Check bounds
    if addr >= self.size {
        return Err("Address out of bounds");
    }
    
    // Check buffer size
    if buffer.is_empty() {
        return Err("Buffer is empty");
    }
    
    // Perform read
    Ok(())
}
```

### 4. Handle Timeouts

```rust
pub fn wait_ready(&self) -> Result<(), &'static str> {
    const TIMEOUT: usize = 100_000;
    
    for _ in 0..TIMEOUT {
        unsafe {
            if (inb(self.base + STATUS_REG) & BUSY_BIT) == 0 {
                return Ok(());
            }
        }
        core::hint::spin_loop();
    }
    
    Err("Timeout waiting for device")
}
```

### 5. Minimize Unsafe Code

```rust
// ❌ Bad: Large unsafe block
pub fn init(&mut self) {
    unsafe {
        // 50 lines of code...
    }
}

// ✅ Good: Small, focused unsafe blocks
pub fn init(&mut self) {
    self.reset();
    let status = unsafe { inb(self.base + STATUS_REG) };
    if status & ERROR_BIT != 0 {
        // Handle error
    }
    self.configure();
}
```

## Testing

### QEMU Testing

Test your driver in QEMU:

```bash
# Build kernel
make build

# Run in QEMU
make run

# Run with specific device
qemu-system-x86_64 -kernel target/.../rinux \
    -device your-device-model
```

### Debug Output

Add debug prints:

```rust
pub fn init(&mut self) -> Result<(), &'static str> {
    printk("[MyDriver] Starting initialization\n");
    
    printk("[MyDriver] Detecting hardware...\n");
    if !self.detect() {
        printk("[MyDriver] Hardware not found\n");
        return Err("Hardware not found");
    }
    printk("[MyDriver] Hardware detected\n");
    
    printk("[MyDriver] Resetting device...\n");
    self.reset();
    printk("[MyDriver] Reset complete\n");
    
    printk("[MyDriver] Initialization successful\n");
    Ok(())
}
```

### Assertions

Use assertions for invariants:

```rust
pub fn read_register(&self, reg: Register) -> u32 {
    assert!(self.initialized, "Driver not initialized");
    assert!(reg as u8 <= 0xFF, "Invalid register");
    
    unsafe {
        inl(self.base + reg as u16)
    }
}
```

## Common Pitfalls

### 1. Forgetting Volatile Access

```rust
// ❌ Bad: Regular memory access (may be optimized away)
let value = *(addr as *const u32);

// ✅ Good: Volatile access (never optimized away)
let value = unsafe { read_volatile(addr as *const u32) };
```

### 2. Race Conditions

```rust
// ❌ Bad: Race condition
static mut COUNTER: u32 = 0;

pub fn increment() {
    unsafe {
        COUNTER += 1; // Not atomic!
    }
}

// ✅ Good: Atomic operation
static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn increment() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
}
```

### 3. Infinite Loops

```rust
// ❌ Bad: Can hang forever
while !device_ready() {
    // Wait indefinitely
}

// ✅ Good: Timeout
let mut timeout = 1000;
while !device_ready() {
    if timeout == 0 {
        return Err("Timeout");
    }
    timeout -= 1;
    core::hint::spin_loop();
}
```

### 4. Buffer Overflows

```rust
// ❌ Bad: No bounds checking
pub fn read(&self, buffer: &mut [u8]) {
    for i in 0..self.size {  // May overflow buffer!
        buffer[i] = self.read_byte(i);
    }
}

// ✅ Good: Check buffer size
pub fn read(&self, buffer: &mut [u8]) -> Result<(), &'static str> {
    if buffer.len() < self.size {
        return Err("Buffer too small");
    }
    
    for i in 0..self.size {
        buffer[i] = self.read_byte(i);
    }
    Ok(())
}
```

## Example Drivers

### See existing drivers for examples:

- **VGA Driver** (`drivers/src/vga.rs`): Simple MMIO device
- **PCI Scanner** (`drivers/src/pci.rs`): Bus enumeration
- **USB xHCI** (`drivers/src/usb/xhci.rs`): Complex PCI device
- **ACPI** (`drivers/src/acpi.rs`): Firmware interface

## Resources

- [OSDev Wiki - Drivers](https://wiki.osdev.org/Category:Drivers)
- [Linux Device Drivers](https://lwn.net/Kernel/LDD3/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [Writing an OS in Rust](https://os.phil-opp.com/)

## Getting Help

- Check existing drivers in `drivers/src/`
- Read hardware datasheets
- Ask on [OSDev Forum](https://forum.osdev.org/)
- Join [Rust OS Dev Discord](https://discord.gg/rust-osdev)

## Summary

Key points for driver development:
1. Document your code thoroughly
2. Use Rust's type system for safety
3. Minimize unsafe code
4. Handle all error cases
5. Test in QEMU first
6. Use timeouts for hardware waits
7. Protect shared state with locks
8. Validate all inputs
9. Use volatile access for MMIO/ports
10. Follow existing driver patterns

Happy driver development! 🚀

# Rinux Hardware Support

## Overview

This document provides a comprehensive overview of hardware support in Rinux, including currently supported hardware, planned hardware support, and how to add support for new hardware devices.

## Current Hardware Support Status

### Architecture Support

#### ✅ Fully Supported
- **x86_64 (AMD64)**: Full support with Multiboot boot protocol

#### ⚠️ Partial Support
- **ARM64/AArch64**: Basic architecture framework in place
- **RISC-V 64**: Basic architecture framework in place

#### ❌ Not Yet Supported
- ARM32, PowerPC, MIPS, SPARC, and other architectures

### System Buses

#### PCI/PCIe ✅ Detection Only
- Configuration space access (I/O ports 0xCF8/0xCFC)
- Device enumeration and discovery
- Vendor/Device ID reading
- BAR (Base Address Register) reading
- Bus mastering enablement
- Memory/IO space enablement
- Interrupt line/pin reading

**Limitations:**
- No PCIe extended configuration space
- No Message Signaled Interrupts (MSI/MSI-X)
- No hot-plug support
- No power management

#### USB ✅ Framework Only
- Device type definitions (Low/Full/High/Super/SuperPlus speed)
- USB class codes defined
- xHCI controller detection
- EHCI/UHCI/OHCI controller detection

**Limitations:**
- No actual USB communication
- No device enumeration
- No transfer handling
- No HID/Mass Storage drivers functional

#### Other Buses ❌ Not Implemented
- I2C, SPI, ISA, SATA, NVMe

### Device Classes

#### Display/Graphics

##### VGA Text Mode ✅ Fully Functional
- 80x25 character display
- 16 color support (8 foreground, 8 background)
- Hardware cursor control
- Direct VGA buffer access (0xB8000)

**Supported Operations:**
- Character and color writing
- Screen clearing
- Cursor positioning
- Scrolling

##### GPU Support ✅ Detection Only
- Intel GPU detection (vendor ID 0x8086)
- AMD GPU detection (vendor ID 0x1002)
- NVIDIA GPU detection (vendor ID 0x10DE)

**Limitations:**
- No mode setting
- No framebuffer graphics
- No 3D acceleration
- No KMS (Kernel Mode Setting)

#### Input Devices

##### Keyboard ❌ Stub Only
- Module exists but not functional
- No PS/2 keyboard support
- No USB keyboard support

##### Mouse/Touchpad ❌ Stub Only
- Module exists but not functional
- No PS/2 mouse support
- No USB mouse support
- No touchpad gesture support

#### Storage Devices ❌ Not Implemented
- No ATA/IDE driver
- No AHCI/SATA driver
- No NVMe driver
- No USB mass storage driver
- No SCSI driver

#### Network Devices ❌ Not Implemented
- No Ethernet drivers
- No WiFi drivers
- No network stack

#### Serial/Communication

##### Serial Port (UART) ❌ Stub Only
- Module exists but not functional
- No COM port I/O
- No serial console support

#### Power Management

##### ACPI ✅ Detection Only
- RSDP (Root System Description Pointer) detection
- ACPI table header parsing
- Power management profile detection
- Laptop/Desktop/Server detection

**Limitations:**
- No AML/ASL interpreter
- No power state control (S0-S5)
- No thermal management
- No CPU frequency scaling

#### Audio ❌ Stub Only
- HD Audio (HDA) framework exists
- AC'97 not implemented
- No actual audio playback
- No mixer control

## Linux Hardware Comparison

### Coverage Estimate
Rinux currently supports approximately **0.1-1%** of hardware devices that Linux supports.

### Device Count Comparison

| Category | Linux | Rinux | Coverage |
|----------|-------|-------|----------|
| PCI Devices | 10,000+ | ~0 (detection only) | 0% |
| USB Devices | 20,000+ | 0 | 0% |
| Network Cards | 1,000+ | 0 | 0% |
| Storage Controllers | 500+ | 0 | 0% |
| Graphics Cards | 200+ | 0 (detection only) | 0% |
| Audio Devices | 500+ | 0 | 0% |
| Input Devices | 5,000+ | 0 | 0% |

### Why Such Low Coverage?

Linux has been developed by thousands of contributors over 30+ years with:
- ~20 million lines of driver code
- Extensive hardware vendor partnerships
- Massive testing infrastructure
- Binary blob support for proprietary hardware

Rinux is in early development (v0.1.0) with:
- ~1,500 lines of driver code
- Educational/experimental focus
- No vendor partnerships
- No binary driver support

## Hardware Support Architecture

### Driver Model

Rinux uses a modular driver architecture organized by device class:

```
drivers/
├── lib.rs              # Driver framework and initialization
├── vga.rs              # VGA text mode driver
├── serial.rs           # Serial port driver (stub)
├── keyboard.rs         # Keyboard driver (stub)
├── pci.rs              # PCI bus driver
├── acpi.rs             # ACPI support
├── power.rs            # Power management
├── touchpad.rs         # Touchpad driver (stub)
├── audio.rs            # Audio driver (stub)
├── usb/                # USB subsystem
│   ├── mod.rs          # USB core
│   ├── xhci.rs         # xHCI controller
│   ├── device.rs       # USB device management
│   ├── hid.rs          # HID devices (stub)
│   └── mass_storage.rs # Mass storage (stub)
└── graphics/           # Graphics subsystem
    ├── mod.rs          # Graphics core
    ├── framebuffer.rs  # Framebuffer support
    ├── intel.rs        # Intel GPU (detection)
    ├── amd.rs          # AMD GPU (detection)
    └── nvidia.rs       # NVIDIA GPU (detection)
```

### Driver Initialization

Drivers are initialized in a specific order in `drivers/src/lib.rs`:

1. **Serial**: Early debug output (when implemented)
2. **Keyboard**: Basic input
3. **VGA**: Console output
4. **ACPI**: System information and power management
5. **PCI**: Device enumeration
6. **Graphics**: GPU initialization
7. **USB**: USB controller and device initialization
8. **Audio**: Audio subsystem
9. **Touchpad**: Input device initialization
10. **Power**: Power management

## Adding Hardware Support

### Step-by-Step Guide

#### 1. Identify the Hardware

Determine:
- **Bus type**: PCI, USB, ISA, etc.
- **Device class**: Storage, network, input, etc.
- **Specifications**: Get official hardware documentation
- **Existing support**: Check if Linux/BSD drivers exist for reference

#### 2. Create Driver Module

Create a new file in `drivers/src/`:

```rust
//! My Hardware Device Driver
//!
//! Driver for [Hardware Name] [Model].

use core::fmt;
use rinux_arch_x86::io::{inb, outb}; // For I/O port access
use spin::Mutex;

/// Device registers
mod regs {
    pub const CONTROL: u16 = 0x00;
    pub const STATUS: u16 = 0x01;
    pub const DATA: u16 = 0x02;
}

/// Device state
pub struct MyDevice {
    base_addr: u16,
    initialized: bool,
}

impl MyDevice {
    pub const fn new() -> Self {
        Self {
            base_addr: 0,
            initialized: false,
        }
    }

    /// Initialize the device
    pub fn init(&mut self, base_addr: u16) -> Result<(), &'static str> {
        self.base_addr = base_addr;
        
        // Perform initialization
        unsafe {
            // Reset device
            outb(base_addr + regs::CONTROL, 0x01);
            
            // Wait for ready
            while (inb(base_addr + regs::STATUS) & 0x80) == 0 {
                core::hint::spin_loop();
            }
        }
        
        self.initialized = true;
        Ok(())
    }

    /// Read from device
    pub fn read(&self) -> Result<u8, &'static str> {
        if !self.initialized {
            return Err("Device not initialized");
        }
        
        unsafe {
            Ok(inb(self.base_addr + regs::DATA))
        }
    }

    /// Write to device
    pub fn write(&mut self, data: u8) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Device not initialized");
        }
        
        unsafe {
            outb(self.base_addr + regs::DATA, data);
        }
        
        Ok(())
    }
}

/// Global device instance
static MY_DEVICE: Mutex<MyDevice> = Mutex::new(MyDevice::new());

/// Initialize driver
pub fn init() {
    rinux_kernel::printk::printk("Initializing MyDevice driver...\n");
    
    // Detect and initialize device
    if let Ok(mut device) = MY_DEVICE.try_lock() {
        match device.init(0x3F8) { // Example base address
            Ok(()) => {
                rinux_kernel::printk::printk("MyDevice: Initialized\n");
            }
            Err(e) => {
                rinux_kernel::printk::printk("MyDevice: Failed to initialize: ");
                rinux_kernel::printk::printk(e);
                rinux_kernel::printk::printk("\n");
            }
        }
    }
}

/// Get device instance
pub fn get_device() -> &'static Mutex<MyDevice> {
    &MY_DEVICE
}
```

#### 3. Register Driver

Add to `drivers/src/lib.rs`:

```rust
pub mod mydevice;

pub fn init() {
    // ... existing initialization ...
    
    // Initialize your device
    mydevice::init();
}
```

#### 4. Add Dependencies

If your driver needs new dependencies, add them to `drivers/Cargo.toml`:

```toml
[dependencies]
spin = "0.9"
rinux-kernel = { path = "../kernel" }
rinux-arch-x86 = { path = "../arch/x86" }
# Add your dependencies here
```

#### 5. Test

Build and test your driver:

```bash
make build
make run  # Test in QEMU
```

### Hardware Detection Methods

#### PCI Device Detection

Use the PCI scanner to find your device:

```rust
use crate::pci;

pub fn init() {
    let scanner = pci::scanner();
    
    // Find device by vendor/device ID
    for i in 0..scanner.device_count() {
        if let Some(device) = scanner.get_device(i) {
            if device.vendor_id == 0x8086 && device.device_id == 0x1234 {
                rinux_kernel::printk::printk("Found my device!\n");
                // Initialize device
                init_device(device);
            }
        }
    }
}
```

#### USB Device Detection

For USB devices (when USB stack is functional):

```rust
use crate::usb;

pub fn on_usb_device_connected(device: &usb::UsbDevice) {
    if device.vendor_id == 0x046d && device.product_id == 0xc52b {
        rinux_kernel::printk::printk("Found my USB device!\n");
        // Initialize USB device
    }
}
```

#### ACPI Device Detection

For ACPI-enumerated devices:

```rust
use crate::acpi;

pub fn init() {
    let acpi_info = acpi::get_info();
    // Use ACPI tables to find device information
}
```

### Memory-Mapped I/O (MMIO)

For devices using MMIO:

```rust
use core::ptr::{read_volatile, write_volatile};

/// Read from MMIO register
unsafe fn read_mmio(base: usize, offset: usize) -> u32 {
    let addr = (base + offset) as *const u32;
    read_volatile(addr)
}

/// Write to MMIO register
unsafe fn write_mmio(base: usize, offset: usize, value: u32) {
    let addr = (base + offset) as *mut u32;
    write_volatile(addr, value);
}
```

### Direct Memory Access (DMA)

DMA support is not yet implemented in Rinux. When implemented, it will follow this pattern:

```rust
// Future DMA API (not yet available)
pub struct DmaBuffer {
    virt_addr: usize,
    phys_addr: usize,
    size: usize,
}

impl DmaBuffer {
    pub fn new(size: usize) -> Result<Self, &'static str> {
        // Allocate physically contiguous memory
        todo!("DMA not yet implemented")
    }
}
```

### Interrupt Handling

Interrupt handling framework is minimal. Future API:

```rust
// Future interrupt API (not yet available)
pub fn register_interrupt_handler(
    irq: u8,
    handler: fn(),
) -> Result<(), &'static str> {
    todo!("Interrupt registration not yet implemented")
}
```

## Hardware Support Roadmap

### Phase 1: Core Devices (v0.2.0 - Q2 2026)
- [ ] Complete serial port driver (16550 UART)
- [ ] Complete PS/2 keyboard driver
- [ ] Complete PS/2 mouse driver
- [ ] Timer driver (PIT, APIC timer)

### Phase 2: Storage (v0.3.0-0.4.0)
- [ ] ATA PIO mode driver
- [ ] AHCI driver (basic read/write)
- [ ] Virtio-blk driver

### Phase 3: Network (v0.5.0)
- [ ] E1000 network card driver
- [ ] Virtio-net driver
- [ ] RTL8139 driver

### Phase 4: Graphics (v0.6.0+)
- [ ] Linear framebuffer support
- [ ] Basic Intel GPU driver
- [ ] VBE/VESA graphics

### Phase 5: USB (v0.7.0+)
- [ ] xHCI driver (basic functionality)
- [ ] USB device enumeration
- [ ] USB HID driver (keyboard, mouse)
- [ ] USB mass storage driver

### Phase 6: Advanced (v1.0+)
- [ ] EHCI driver
- [ ] NVMe driver
- [ ] WiFi driver (basic)
- [ ] Audio driver (AC'97 or HDA)

## Hardware Support Limitations

### Current Technical Limitations

1. **No DMA**: Direct Memory Access not implemented
2. **No Interrupts**: Limited interrupt handling
3. **No Power Management**: Cannot manage device power states
4. **No Hot-plug**: Cannot detect device insertion/removal
5. **No Multi-threading**: Single-threaded driver model
6. **No User Space**: All drivers in kernel space

### Architectural Limitations

1. **x86_64 Only**: No ARM/RISC-V hardware support yet
2. **No UEFI**: Legacy BIOS only
3. **No Secure Boot**: Cannot boot on secure boot systems
4. **No IOMMU**: No memory protection for DMA
5. **No Virtualization**: Cannot use VT-x/AMD-V features

### Resource Limitations

1. **Limited Documentation**: Less hardware docs than Linux
2. **No Vendor Support**: No hardware vendor partnerships
3. **Small Team**: Limited development resources
4. **Testing**: Limited real hardware testing

## Testing Hardware Support

### QEMU Virtual Devices

QEMU provides virtual hardware for testing:

```bash
# Basic QEMU devices
qemu-system-x86_64 -kernel kernel.bin \
    -serial stdio \              # Serial port
    -vga std \                   # VGA graphics
    -soundhw hda \              # Intel HDA audio
    -device usb-ehci \          # USB EHCI controller
    -device usb-mouse \         # USB mouse
    -device e1000 \             # Intel E1000 network
    -drive file=disk.img        # IDE hard disk
```

### Real Hardware Testing

Testing on real hardware requires:

1. **Bootable Media**: Create bootable USB/CD
2. **Debug Hardware**: Serial console or debug port
3. **Test Machine**: Dedicated test computer
4. **Safety**: Backup important data

**Warning**: Testing on real hardware can:
- Cause data loss
- Damage hardware (rare but possible)
- Require BIOS modifications
- Void warranties

## Contributing Hardware Support

### Where to Start

1. **Simple Devices**: Start with well-documented, simple devices
2. **Virtual Devices**: Test with QEMU virtual hardware first
3. **Reference Drivers**: Study Linux/BSD drivers
4. **Specifications**: Read official hardware specifications

### Good First Devices

- **Serial Port** (16550 UART): Simple, well-documented
- **PS/2 Keyboard**: Standard interface, good docs
- **ATA PIO Mode**: Simple disk access
- **E1000 NIC**: Open specs, QEMU support

### Resources

#### Specifications
- [PCI Local Bus Specification](https://pcisig.com/specifications)
- [USB Specifications](https://www.usb.org/documents)
- [ACPI Specification](https://uefi.org/specifications)
- [Intel 64 and IA-32 Architectures Software Developer Manuals](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)

#### Documentation
- [OSDev Wiki](https://wiki.osdev.org/)
- [Linux Device Drivers Book](https://lwn.net/Kernel/LDD3/)
- [Writing an OS in Rust](https://os.phil-opp.com/)

#### Communities
- [OSDev Forum](https://forum.osdev.org/)
- [Rust OS Development Discord](https://discord.gg/rust-osdev)

## Frequently Asked Questions

### Q: Why not support more hardware?

**A**: Rinux is an educational/experimental kernel. Supporting all Linux hardware would require:
- Years of development time
- Thousands of contributors
- Millions of lines of code
- Hardware vendor partnerships
- Extensive testing infrastructure

Our focus is on core functionality and learning, not comprehensive hardware support.

### Q: Can I use proprietary drivers?

**A**: Currently no. Rinux does not support:
- Binary driver blobs
- Closed-source drivers
- Loadable kernel modules

All drivers must be open source and compiled into the kernel.

### Q: Will Rinux support ARM/RISC-V hardware?

**A**: Basic ARM64 and RISC-V support exists but is not functional. Full support is planned for future versions after x86_64 stabilizes.

### Q: Can Rinux run on my laptop?

**A**: Probably not reliably. Rinux lacks:
- Most device drivers (WiFi, touchpad, battery, etc.)
- Power management
- ACPI support
- Modern boot protocols (UEFI)

It may boot but will have limited functionality.

### Q: How can I add support for my hardware?

**A**: Follow the "Adding Hardware Support" guide in this document. Start with:
1. Study the hardware specifications
2. Look at existing Rinux drivers
3. Check Linux/BSD drivers for reference
4. Write a minimal driver
5. Test in QEMU first
6. Submit a pull request

### Q: Is there a list of supported hardware?

**A**: Yes, see the "Current Hardware Support Status" section. Currently supported hardware is very limited (mostly detection only).

## Summary

Rinux currently has **minimal hardware support** compared to Linux. The focus is on:
- Learning operating system development
- Exploring Rust in kernel development
- Building a solid architectural foundation

Hardware support will expand gradually as the kernel matures. Contributions are welcome!

---

**Last Updated**: 2026-02-20  
**Rinux Version**: 0.1.0  
**Maintainers**: Rinux Project Contributors

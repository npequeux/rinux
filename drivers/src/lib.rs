//! Device Drivers
//!
//! Device driver framework and drivers.

#![no_std]

pub mod keyboard;
pub mod pci;
pub mod serial;
pub mod usb;
pub mod vga;

/// Initialize all drivers
pub fn init() {
    serial::init();
    keyboard::init();
    vga::init();

    // Initialize PCI bus
    pci::init();

    // Initialize USB subsystem (depends on PCI)
    usb::init();
}

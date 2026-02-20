//! Device Drivers
//!
//! Device driver framework and drivers.

#![no_std]

pub mod serial;
pub mod keyboard;
pub mod vga;
pub mod pci;
pub mod usb;
pub mod graphics;
pub mod acpi;
pub mod touchpad;
pub mod power;
pub mod audio;

/// Initialize all drivers
pub fn init() {
    serial::init();
    keyboard::init();
    vga::init();

    // Initialize ACPI for power management and system info
    acpi::init();

    // Initialize PCI bus
    pci::init();

    // Initialize graphics subsystem
    graphics::init();

    // Initialize USB subsystem (depends on PCI)
    usb::init();

    // Initialize audio
    audio::init();

    // Initialize touchpad/input devices
    touchpad::init();

    // Initialize power management
    power::init();
}

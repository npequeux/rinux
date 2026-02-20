//! Device Drivers
//!
//! Device driver framework and drivers.

#![no_std]

pub mod acpi;
pub mod audio;
pub mod graphics;
pub mod keyboard;
pub mod pci;
pub mod power;
pub mod rtc;
pub mod serial;
pub mod timer;
pub mod touchpad;
pub mod usb;
pub mod vga;

/// Initialize all drivers
pub fn init() {
    serial::init();
    keyboard::init();
    vga::init();

    // Initialize RTC
    rtc::init();

    // Initialize timer (100 Hz = 10ms tick)
    timer::init(100);

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

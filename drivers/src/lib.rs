//! Device Drivers
//!
//! Device driver framework and drivers.

#![no_std]

pub mod serial;
pub mod keyboard;
pub mod vga;

/// Initialize all drivers
pub fn init() {
    serial::init();
    keyboard::init();
    vga::init();
}

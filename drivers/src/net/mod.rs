//! Network Device Drivers
//!
//! Network interface drivers for various hardware.

pub mod e1000;

/// Initialize all network drivers
pub fn init() {
    e1000::init();
}

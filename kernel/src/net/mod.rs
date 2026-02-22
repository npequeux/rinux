//! Network Subsystem
//!
//! TCP/IP Network stack implementation

use core::sync::atomic::{AtomicBool, Ordering};

pub mod arp;
pub mod ethernet;
pub mod ipv4;
pub mod netdev;
pub mod socket;
pub mod udp;

// TODO: Add these modules as they're implemented
// pub mod tcp;
// pub mod icmp;

/// Network subsystem initialized flag
static NET_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize network subsystem
pub fn init() {
    if NET_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    // Initialize subsystems in order
    netdev::init();
    ethernet::init();
    arp::init();
    ipv4::init();
    udp::init();
    socket::init();

    NET_INITIALIZED.store(true, Ordering::Release);
}

/// Check if network subsystem is initialized
pub fn is_initialized() -> bool {
    NET_INITIALIZED.load(Ordering::Acquire)
}

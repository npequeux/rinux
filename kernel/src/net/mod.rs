//! Network Subsystem
//!
//! TCP/IP Network stack implementation

pub mod socket;

// TODO: Add these modules as they're implemented
// pub mod ipv4;
// pub mod tcp;
// pub mod udp;
// pub mod icmp;
// pub mod ethernet;

/// Initialize network subsystem
pub fn init() {
    socket::init();
    // ipv4::init();
    // tcp::init();
    // udp::init();
}

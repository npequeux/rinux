//! Storage Subsystem
//!
//! Block devices, disk controllers, and partition support

pub mod ahci;
pub mod block;
pub mod nvme;
pub mod partition;

/// Initialize storage subsystem
pub fn init() {
    block::init();
    partition::init();
}

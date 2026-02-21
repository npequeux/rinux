//! Block Device Layer
//!
//! Provides abstraction for block devices (hard drives, SSDs, etc.)

#![no_std]

extern crate alloc;

pub mod device;
pub mod request;
pub mod partition;
pub mod ahci;
pub mod nvme;

use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
use device::BlockDevice;

/// Global list of block devices
static BLOCK_DEVICES: Mutex<Vec<Arc<dyn BlockDevice>>> = Mutex::new(Vec::new());

/// Register a block device
pub fn register_device(device: Arc<dyn BlockDevice>) -> Result<(), &'static str> {
    let mut devices = BLOCK_DEVICES.lock();
    devices.push(device);
    Ok(())
}

/// Get a block device by index
pub fn get_device(index: usize) -> Option<Arc<dyn BlockDevice>> {
    let devices = BLOCK_DEVICES.lock();
    devices.get(index).cloned()
}

/// Get number of registered block devices
pub fn device_count() -> usize {
    BLOCK_DEVICES.lock().len()
}

/// Initialize block device subsystem
pub fn init() {
    // Initialize AHCI driver
    ahci::init();
    
    // Initialize NVMe driver
    nvme::init();
    
    // Scan for partitions on all devices
    partition::scan_all();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_count() {
        assert_eq!(device_count(), 0);
    }
}

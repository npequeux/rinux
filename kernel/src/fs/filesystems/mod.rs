//! File Systems
//!
//! Different filesystem implementations

pub mod procfs;
pub mod sysfs;
pub mod tmpfs;

/// Initialize all filesystems
pub fn init() {
    tmpfs::init();
    procfs::init();
    sysfs::init();
}

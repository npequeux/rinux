//! File Systems
//!
//! Different filesystem implementations

pub mod tmpfs;
pub mod procfs;
pub mod sysfs;

/// Initialize all filesystems
pub fn init() {
    tmpfs::init();
    procfs::init();
    sysfs::init();
}

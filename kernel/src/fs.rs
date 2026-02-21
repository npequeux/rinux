//! File System Support
//!
//! Virtual File System (VFS) layer and file operations.

pub mod fd;
pub mod file;
pub mod filesystems;
pub mod vfs;

pub use fd::{FileDescriptor, FileDescriptorTable};
pub use file::{File, FileMode, FileType};
pub use vfs::{VfsNode, VfsNodeType};

use core::sync::atomic::{AtomicBool, Ordering};

static FS_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize file system subsystem
pub fn init() {
    if FS_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    fd::init();
    vfs::init();
    filesystems::init();

    FS_INITIALIZED.store(true, Ordering::Release);
    crate::printk::printk("  File system subsystem initialized\n");
}

/// Check if file system is initialized
pub fn is_initialized() -> bool {
    FS_INITIALIZED.load(Ordering::Acquire)
}

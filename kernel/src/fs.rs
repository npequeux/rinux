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

/// Open flags
pub mod flags {
    /// Open for reading only
    pub const O_RDONLY: i32 = 0x0000;
    /// Open for writing only
    pub const O_WRONLY: i32 = 0x0001;
    /// Open for reading and writing
    pub const O_RDWR: i32 = 0x0002;
    /// Create file if it doesn't exist
    pub const O_CREAT: i32 = 0x0100;
    /// Fail if file exists and O_CREAT is set
    pub const O_EXCL: i32 = 0x0200;
    /// Truncate file to zero length
    pub const O_TRUNC: i32 = 0x0400;
    /// Append to end of file
    pub const O_APPEND: i32 = 0x0800;
}

/// Open a file
pub fn open_file(_pathname: &str, flags: i32, _mode: u32) -> Result<FileDescriptor, isize> {
    use crate::syscall::errno;

    if !is_initialized() {
        return Err(errno::EIO);
    }

    // Parse flags to determine access mode
    let mode = match flags & 0x3 {
        flags::O_RDONLY => FileMode::read_only(),
        flags::O_WRONLY => FileMode::write_only(),
        flags::O_RDWR => FileMode::read_write(),
        _ => return Err(errno::EINVAL),
    };

    // For now, create a dummy file
    // TODO: Integrate with actual VFS to lookup/create files
    let file = File::new(0, FileType::Regular, mode);

    match fd::allocate_fd(file) {
        Ok(fd) => Ok(fd),
        Err(_) => Err(errno::EMFILE),
    }
}

/// Read from a file
pub fn read_file(file: &mut File, buf: *mut u8, count: usize) -> Result<usize, ()> {
    if !file.is_readable() {
        return Err(());
    }

    // For now, return 0 (EOF) for all reads
    // TODO: Integrate with actual filesystem read operations
    let _ = (buf, count);
    Ok(0)
}

/// Write to a file
pub fn write_file(file: &mut File, buf: *const u8, count: usize) -> Result<usize, ()> {
    if !file.is_writable() {
        return Err(());
    }

    // For now, pretend we wrote all bytes
    // TODO: Integrate with actual filesystem write operations
    let _ = buf;
    Ok(count)
}

/// Get a mutable reference to a file by file descriptor
pub fn get_file_mut(fd: FileDescriptor) -> Option<&'static mut File> {
    // This is a placeholder - proper implementation needs per-process FD tables
    // For now, return None
    let _ = fd;
    None
}

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

//! Filesystem Support
//!
//! Provides various filesystem implementations

#![no_std]

extern crate alloc;

pub mod tmpfs;
pub mod ext2;
pub mod vfs;

/// Filesystem error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    /// File or directory not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// Already exists
    AlreadyExists,
    /// Not a directory
    NotADirectory,
    /// Is a directory
    IsADirectory,
    /// Directory not empty
    NotEmpty,
    /// Invalid argument
    InvalidArgument,
    /// No space left on device
    NoSpaceLeft,
    /// Read-only filesystem
    ReadOnly,
    /// Invalid filesystem
    InvalidFs,
    /// I/O error
    IoError,
    /// Out of memory
    OutOfMemory,
}

/// Filesystem type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsType {
    /// Temporary filesystem (in-memory)
    TmpFs,
    /// Second Extended Filesystem
    Ext2,
    /// Fourth Extended Filesystem
    Ext4,
    /// FAT32 filesystem
    FAT32,
    /// Proc filesystem
    ProcFs,
    /// Sys filesystem
    SysFs,
    /// Dev filesystem
    DevFs,
}

/// Initialize filesystem subsystem
pub fn init() {
    tmpfs::init();
    ext2::init();
}

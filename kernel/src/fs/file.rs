//! File Structure
//!
//! Represents an open file.

use crate::types::Inode;

/// File type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Regular file
    Regular,
    /// Directory
    Directory,
    /// Character device
    CharDevice,
    /// Block device
    BlockDevice,
    /// Named pipe (FIFO)
    Fifo,
    /// Symbolic link
    Symlink,
    /// Socket
    Socket,
}

/// File access mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode {
    /// Read permission
    pub read: bool,
    /// Write permission
    pub write: bool,
    /// Execute permission
    pub execute: bool,
}

impl FileMode {
    /// Create a read-only mode
    pub const fn read_only() -> Self {
        FileMode {
            read: true,
            write: false,
            execute: false,
        }
    }

    /// Create a write-only mode
    pub const fn write_only() -> Self {
        FileMode {
            read: false,
            write: true,
            execute: false,
        }
    }

    /// Create a read-write mode
    pub const fn read_write() -> Self {
        FileMode {
            read: true,
            write: true,
            execute: false,
        }
    }
}

/// File structure
#[derive(Clone)]
pub struct File {
    /// Inode number
    pub inode: Inode,
    /// File type
    pub file_type: FileType,
    /// Access mode
    pub mode: FileMode,
    /// Current position in file
    pub position: u64,
    /// File size
    pub size: u64,
}

impl File {
    /// Create a new file
    pub fn new(inode: Inode, file_type: FileType, mode: FileMode) -> Self {
        File {
            inode,
            file_type,
            mode,
            position: 0,
            size: 0,
        }
    }

    /// Check if file is readable
    pub fn is_readable(&self) -> bool {
        self.mode.read
    }

    /// Check if file is writable
    pub fn is_writable(&self) -> bool {
        self.mode.write
    }

    /// Seek to position
    pub fn seek(&mut self, offset: u64) {
        self.position = offset;
    }

    /// Get current position
    pub fn tell(&self) -> u64 {
        self.position
    }
}

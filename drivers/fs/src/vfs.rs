//! Virtual Filesystem (VFS) Layer
//!
//! Provides a common interface for all filesystems

use crate::FsError;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::fmt;

/// File mode and permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(pub u32);

impl FileMode {
    /// Owner read permission
    pub const OWNER_READ: u32 = 0o400;
    /// Owner write permission
    pub const OWNER_WRITE: u32 = 0o200;
    /// Owner execute permission
    pub const OWNER_EXECUTE: u32 = 0o100;
    /// Group read permission
    pub const GROUP_READ: u32 = 0o040;
    /// Group write permission
    pub const GROUP_WRITE: u32 = 0o020;
    /// Group execute permission
    pub const GROUP_EXECUTE: u32 = 0o010;
    /// Other read permission
    pub const OTHER_READ: u32 = 0o004;
    /// Other write permission
    pub const OTHER_WRITE: u32 = 0o002;
    /// Other execute permission
    pub const OTHER_EXECUTE: u32 = 0o001;

    pub fn new(mode: u32) -> Self {
        FileMode(mode)
    }

    pub fn is_readable(&self) -> bool {
        (self.0 & Self::OWNER_READ) != 0
    }

    pub fn is_writable(&self) -> bool {
        (self.0 & Self::OWNER_WRITE) != 0
    }

    pub fn is_executable(&self) -> bool {
        (self.0 & Self::OWNER_EXECUTE) != 0
    }
}

/// File type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Regular file
    Regular,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
    /// Character device
    CharDevice,
    /// Block device
    BlockDevice,
    /// FIFO (named pipe)
    Fifo,
    /// Socket
    Socket,
}

/// File attributes
#[derive(Debug, Clone)]
pub struct FileAttr {
    /// File type
    pub file_type: FileType,
    /// File mode and permissions
    pub mode: FileMode,
    /// File size in bytes
    pub size: u64,
    /// Number of hard links
    pub nlink: u32,
    /// User ID
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// Inode number
    pub ino: u64,
    /// Number of 512-byte blocks allocated
    pub blocks: u64,
    /// Access time
    pub atime: u64,
    /// Modification time
    pub mtime: u64,
    /// Change time
    pub ctime: u64,
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Inode number
    pub ino: u64,
    /// File type
    pub file_type: FileType,
    /// File name
    pub name: String,
}

/// VNode (Virtual Node) - represents a file or directory
pub trait VNode: Send + Sync {
    /// Read from file
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, FsError>;

    /// Write to file
    fn write(&self, offset: u64, buffer: &[u8]) -> Result<usize, FsError>;

    /// Get file attributes
    fn getattr(&self) -> Result<FileAttr, FsError>;

    /// Set file attributes
    fn setattr(&self, attr: &FileAttr) -> Result<(), FsError>;

    /// Read directory entries
    fn readdir(&self) -> Result<Vec<DirEntry>, FsError>;

    /// Look up a child entry by name
    fn lookup(&self, name: &str) -> Result<Arc<dyn VNode>, FsError>;

    /// Create a new file
    fn create(&self, name: &str, mode: FileMode) -> Result<Arc<dyn VNode>, FsError>;

    /// Create a new directory
    fn mkdir(&self, name: &str, mode: FileMode) -> Result<Arc<dyn VNode>, FsError>;

    /// Remove a file
    fn unlink(&self, name: &str) -> Result<(), FsError>;

    /// Remove a directory
    fn rmdir(&self, name: &str) -> Result<(), FsError>;

    /// Rename a file or directory
    fn rename(&self, old_name: &str, new_parent: Arc<dyn VNode>, new_name: &str) -> Result<(), FsError>;

    /// Create a symbolic link
    fn symlink(&self, name: &str, target: &str) -> Result<Arc<dyn VNode>, FsError>;

    /// Read symbolic link target
    fn readlink(&self) -> Result<String, FsError>;

    /// Truncate file to specified size
    fn truncate(&self, size: u64) -> Result<(), FsError>;

    /// Sync file data to storage
    fn fsync(&self) -> Result<(), FsError>;
}

/// Filesystem operations
pub trait Filesystem: Send + Sync {
    /// Get filesystem type
    fn fs_type(&self) -> crate::FsType;

    /// Get root VNode
    fn root(&self) -> Arc<dyn VNode>;

    /// Sync all filesystem data to storage
    fn sync(&self) -> Result<(), FsError>;

    /// Get filesystem statistics
    fn statfs(&self) -> Result<StatFs, FsError>;

    /// Unmount filesystem
    fn unmount(&self) -> Result<(), FsError>;
}

/// Filesystem statistics
#[derive(Debug, Clone)]
pub struct StatFs {
    /// Filesystem type
    pub fs_type: u64,
    /// Optimal transfer block size
    pub block_size: u64,
    /// Total data blocks in filesystem
    pub blocks: u64,
    /// Free blocks in filesystem
    pub blocks_free: u64,
    /// Free blocks available to non-superuser
    pub blocks_available: u64,
    /// Total file nodes in filesystem
    pub files: u64,
    /// Free file nodes in filesystem
    pub files_free: u64,
    /// Maximum length of filenames
    pub name_max: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_mode() {
        let mode = FileMode::new(0o644);
        assert!(mode.is_readable());
        assert!(mode.is_writable());
        assert!(!mode.is_executable());
    }
}

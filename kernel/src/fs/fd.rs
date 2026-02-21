//! File Descriptor Management
//!
//! File descriptor table and allocation.

use super::file::File;
use alloc::vec::Vec;
use spin::Mutex;

/// File descriptor type
pub type FileDescriptor = i32;

/// Standard file descriptors
pub const STDIN_FILENO: FileDescriptor = 0;
pub const STDOUT_FILENO: FileDescriptor = 1;
pub const STDERR_FILENO: FileDescriptor = 2;

/// File descriptor table entry
pub enum FdEntry {
    /// Empty slot
    Empty,
    /// Open file
    File(File),
}

/// File descriptor table
pub struct FileDescriptorTable {
    entries: Vec<FdEntry>,
}

impl FileDescriptorTable {
    /// Create a new file descriptor table
    pub fn new() -> Self {
        let mut entries = Vec::new();
        // Reserve standard file descriptors
        for _ in 0..3 {
            entries.push(FdEntry::Empty);
        }
        FileDescriptorTable { entries }
    }

    /// Allocate a new file descriptor
    pub fn allocate_fd(&mut self, file: File) -> Result<FileDescriptor, ()> {
        // Try to find an empty slot
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if matches!(entry, FdEntry::Empty) {
                *entry = FdEntry::File(file);
                return Ok(i as FileDescriptor);
            }
        }

        // No empty slot, add a new one
        let fd = self.entries.len() as FileDescriptor;
        self.entries.push(FdEntry::File(file));
        Ok(fd)
    }

    /// Free a file descriptor
    pub fn free_fd(&mut self, fd: FileDescriptor) -> Result<(), ()> {
        if fd < 0 || fd as usize >= self.entries.len() {
            return Err(());
        }

        self.entries[fd as usize] = FdEntry::Empty;
        Ok(())
    }

    /// Get a file by descriptor
    pub fn get_file(&self, fd: FileDescriptor) -> Option<&File> {
        if fd < 0 || fd as usize >= self.entries.len() {
            return None;
        }

        match &self.entries[fd as usize] {
            FdEntry::File(file) => Some(file),
            FdEntry::Empty => None,
        }
    }

    /// Get a mutable file by descriptor
    pub fn get_file_mut(&mut self, fd: FileDescriptor) -> Option<&mut File> {
        if fd < 0 || fd as usize >= self.entries.len() {
            return None;
        }

        match &mut self.entries[fd as usize] {
            FdEntry::File(file) => Some(file),
            FdEntry::Empty => None,
        }
    }
}

/// Global file descriptor table (for kernel)
static GLOBAL_FD_TABLE: Mutex<Option<FileDescriptorTable>> = Mutex::new(None);

/// Initialize file descriptor subsystem
pub fn init() {
    let mut table = GLOBAL_FD_TABLE.lock();
    *table = Some(FileDescriptorTable::new());
}

/// Allocate a file descriptor globally
pub fn allocate_fd(file: File) -> Result<FileDescriptor, ()> {
    let mut table = GLOBAL_FD_TABLE.lock();
    if let Some(ref mut t) = *table {
        t.allocate_fd(file)
    } else {
        Err(())
    }
}

/// Free a file descriptor globally
pub fn free_fd(fd: FileDescriptor) -> Result<(), ()> {
    let mut table = GLOBAL_FD_TABLE.lock();
    if let Some(ref mut t) = *table {
        t.free_fd(fd)
    } else {
        Err(())
    }
}

/// Get a file by descriptor globally
pub fn get_file(fd: FileDescriptor) -> Option<File> {
    let table = GLOBAL_FD_TABLE.lock();
    if let Some(ref t) = *table {
        t.get_file(fd).cloned()
    } else {
        None
    }
}

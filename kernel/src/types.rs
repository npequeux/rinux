//! Kernel Types
//!
//! Common type definitions used throughout the kernel.

/// Process ID
pub type Pid = i32;

/// User ID
pub type Uid = u32;

/// Group ID
pub type Gid = u32;

/// File descriptor
pub type Fd = i32;

/// Error number
pub type Errno = i32;

/// Physical address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

/// Virtual address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl PhysAddr {
    /// Create a new physical address
    pub const fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    /// Get the address as a u64
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl VirtAddr {
    /// Create a new virtual address
    pub const fn new(addr: u64) -> Self {
        VirtAddr(addr)
    }

    /// Get the address as a u64
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

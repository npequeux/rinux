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

/// Inode number
pub type Inode = u64;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phys_addr_new() {
        let addr = PhysAddr::new(0x1000);
        assert_eq!(addr.0, 0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
    }

    #[test]
    fn test_phys_addr_as_u64() {
        let addr = PhysAddr::new(0xDEADBEEF);
        assert_eq!(addr.as_u64(), 0xDEADBEEF);
    }

    #[test]
    fn test_phys_addr_zero() {
        let addr = PhysAddr::new(0);
        assert_eq!(addr.as_u64(), 0);
    }

    #[test]
    fn test_phys_addr_max() {
        let addr = PhysAddr::new(u64::MAX);
        assert_eq!(addr.as_u64(), u64::MAX);
    }

    #[test]
    fn test_phys_addr_equality() {
        let addr1 = PhysAddr::new(0x1000);
        let addr2 = PhysAddr::new(0x1000);
        let addr3 = PhysAddr::new(0x2000);

        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn test_phys_addr_ordering() {
        let addr1 = PhysAddr::new(0x1000);
        let addr2 = PhysAddr::new(0x2000);
        let addr3 = PhysAddr::new(0x3000);

        assert!(addr1 < addr2);
        assert!(addr2 < addr3);
        assert!(addr1 < addr3);
        assert!(addr2 > addr1);
        assert!(addr3 > addr2);
    }

    #[test]
    fn test_phys_addr_clone() {
        let addr1 = PhysAddr::new(0x1000);
        let addr2 = addr1.clone();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_phys_addr_copy() {
        let addr1 = PhysAddr::new(0x1000);
        let addr2 = addr1;
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_virt_addr_new() {
        let addr = VirtAddr::new(0x1000);
        assert_eq!(addr.0, 0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
    }

    #[test]
    fn test_virt_addr_as_u64() {
        let addr = VirtAddr::new(0xCAFEBABE);
        assert_eq!(addr.as_u64(), 0xCAFEBABE);
    }

    #[test]
    fn test_virt_addr_zero() {
        let addr = VirtAddr::new(0);
        assert_eq!(addr.as_u64(), 0);
    }

    #[test]
    fn test_virt_addr_max() {
        let addr = VirtAddr::new(u64::MAX);
        assert_eq!(addr.as_u64(), u64::MAX);
    }

    #[test]
    fn test_virt_addr_equality() {
        let addr1 = VirtAddr::new(0x1000);
        let addr2 = VirtAddr::new(0x1000);
        let addr3 = VirtAddr::new(0x2000);

        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn test_virt_addr_ordering() {
        let addr1 = VirtAddr::new(0x1000);
        let addr2 = VirtAddr::new(0x2000);
        let addr3 = VirtAddr::new(0x3000);

        assert!(addr1 < addr2);
        assert!(addr2 < addr3);
        assert!(addr1 < addr3);
        assert!(addr2 > addr1);
        assert!(addr3 > addr2);
    }

    #[test]
    fn test_virt_addr_clone() {
        let addr1 = VirtAddr::new(0x1000);
        let addr2 = addr1.clone();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_virt_addr_copy() {
        let addr1 = VirtAddr::new(0x1000);
        let addr2 = addr1;
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_phys_virt_addr_different_types() {
        let phys = PhysAddr::new(0x1000);
        let virt = VirtAddr::new(0x1000);

        // These should be different types
        assert_eq!(phys.as_u64(), virt.as_u64());
    }

    #[test]
    fn test_addr_const_fn() {
        const PHYS: PhysAddr = PhysAddr::new(0x1000);
        const VIRT: VirtAddr = VirtAddr::new(0x2000);

        assert_eq!(PHYS.as_u64(), 0x1000);
        assert_eq!(VIRT.as_u64(), 0x2000);
    }

    #[test]
    fn test_type_aliases() {
        let _pid: Pid = 1;
        let _uid: Uid = 1000;
        let _gid: Gid = 1000;
        let _fd: Fd = 3;
        let _errno: Errno = -1;

        // Just verify type aliases work
        assert_eq!(_pid, 1);
        assert_eq!(_uid, 1000);
        assert_eq!(_gid, 1000);
        assert_eq!(_fd, 3);
        assert_eq!(_errno, -1);
    }
}

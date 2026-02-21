//! Paging Support
//!
//! Higher-level paging operations on top of architecture-specific code.

use crate::frame::{Frame, allocate_frame, deallocate_frame};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// TLB shootdown support
pub mod tlb {
    use super::*;

    /// TLB flush request
    pub struct TlbFlushRequest {
        pub virt_addr: u64,
        pub flush_all: bool,
    }

    static TLB_FLUSH_PENDING: AtomicBool = AtomicBool::new(false);
    static TLB_FLUSH_REQUEST: Mutex<Option<TlbFlushRequest>> = Mutex::new(None);

    /// Initiate a TLB shootdown for all CPUs
    pub fn shootdown_all(virt_addr: u64) {
        let mut request = TLB_FLUSH_REQUEST.lock();
        *request = Some(TlbFlushRequest {
            virt_addr,
            flush_all: false,
        });
        TLB_FLUSH_PENDING.store(true, Ordering::Release);

        // TODO: Send IPI to all other CPUs to flush their TLBs
        // For now, just flush local TLB
        flush_local(virt_addr);

        TLB_FLUSH_PENDING.store(false, Ordering::Release);
        *request = None;
    }

    /// Flush entire TLB on all CPUs
    pub fn shootdown_full() {
        let mut request = TLB_FLUSH_REQUEST.lock();
        *request = Some(TlbFlushRequest {
            virt_addr: 0,
            flush_all: true,
        });
        TLB_FLUSH_PENDING.store(true, Ordering::Release);

        // TODO: Send IPI to all other CPUs
        flush_local_all();

        TLB_FLUSH_PENDING.store(false, Ordering::Release);
        *request = None;
    }

    /// Flush local CPU's TLB entry
    fn flush_local(addr: u64) {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack));
            }
        }
    }

    /// Flush local CPU's entire TLB
    fn flush_local_all() {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                let cr3: u64;
                core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
                core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
            }
        }
    }

    /// Handle TLB flush IPI (called from interrupt handler)
    pub fn handle_flush_ipi() {
        if TLB_FLUSH_PENDING.load(Ordering::Acquire) {
            if let Some(request) = TLB_FLUSH_REQUEST.lock().as_ref() {
                if request.flush_all {
                    flush_local_all();
                } else {
                    flush_local(request.virt_addr);
                }
            }
        }
    }
}

/// Memory zones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryZone {
    /// DMA zone (0-16MB) for legacy DMA devices
    Dma,
    /// Normal zone (16MB-896MB) for regular allocations
    Normal,
    /// High memory zone (>896MB) on 32-bit systems
    High,
}

/// Virtual address wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        VirtAddr(addr)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn align_down(&self, align: u64) -> Self {
        VirtAddr(self.0 & !(align - 1))
    }

    pub fn align_up(&self, align: u64) -> Self {
        VirtAddr((self.0 + align - 1) & !(align - 1))
    }

    pub fn is_aligned(&self, align: u64) -> bool {
        self.0 % align == 0
    }

    pub fn page_offset(&self) -> u64 {
        self.0 & 0xFFF
    }

    pub fn page_number(&self) -> u64 {
        self.0 >> 12
    }
}

/// Physical address wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn align_down(&self, align: u64) -> Self {
        PhysAddr(self.0 & !(align - 1))
    }

    pub fn align_up(&self, align: u64) -> Self {
        PhysAddr((self.0 + align - 1) & !(align - 1))
    }

    pub fn is_aligned(&self, align: u64) -> bool {
        self.0 % align == 0
    }

    pub fn zone(&self) -> MemoryZone {
        match self.0 {
            0..=0xFF_FFFF => MemoryZone::Dma,           // 0-16MB
            0x100_0000..=0x37FF_FFFF => MemoryZone::Normal, // 16MB-896MB
            _ => MemoryZone::High,                      // >896MB
        }
    }
}

/// Huge page sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugePageSize {
    /// 2MB huge pages
    Size2MB,
    /// 1GB huge pages
    Size1GB,
}

impl HugePageSize {
    pub fn size(&self) -> u64 {
        match self {
            HugePageSize::Size2MB => 2 * 1024 * 1024,
            HugePageSize::Size1GB => 1024 * 1024 * 1024,
        }
    }

    pub fn alignment(&self) -> u64 {
        self.size()
    }
}

/// Page mapper for managing virtual to physical mappings
pub struct PageMapper {
    // Page table root (CR3 value)
    root: PhysAddr,
}

impl PageMapper {
    /// Create a new page mapper with the current CR3
    ///
    /// # Safety
    ///
    /// Caller must ensure the current CR3 points to a valid page table.
    pub unsafe fn new() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            let cr3: u64;
            core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
            PageMapper {
                root: PhysAddr::new(cr3),
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            PageMapper {
                root: PhysAddr::new(0),
            }
        }
    }

    /// Map a virtual page to a physical frame
    pub fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        writable: bool,
        user: bool,
    ) -> Result<(), &'static str> {
        // TODO: Walk page tables and create mapping
        // For now, this is a stub
        let _ = (virt, phys, writable, user);
        Ok(())
    }

    /// Unmap a virtual page
    pub fn unmap_page(&mut self, virt: VirtAddr) -> Result<Frame, &'static str> {
        // TODO: Walk page tables and remove mapping
        // For now, this is a stub
        let _ = virt;
        Err("Not implemented")
    }

    /// Translate a virtual address to physical
    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        // TODO: Walk page tables to find physical address
        // For now, return None
        let _ = virt;
        None
    }

    /// Map a huge page
    pub fn map_huge_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        size: HugePageSize,
        writable: bool,
        user: bool,
    ) -> Result<(), &'static str> {
        // Verify alignment
        if !virt.is_aligned(size.alignment()) || !phys.is_aligned(size.alignment()) {
            return Err("Addresses not aligned for huge page");
        }

        // TODO: Set huge page bit in page table entry
        let _ = (virt, phys, size, writable, user);
        Ok(())
    }
}

/// NUMA node information
#[derive(Debug, Clone, Copy)]
pub struct NumaNode {
    pub id: u32,
    pub memory_start: PhysAddr,
    pub memory_end: PhysAddr,
}

static NUMA_NODES: Mutex<Option<alloc::vec::Vec<NumaNode>>> = Mutex::new(None);

/// Initialize NUMA support
pub fn init_numa() {
    let mut nodes = NUMA_NODES.lock();
    *nodes = Some(alloc::vec::Vec::new());

    // TODO: Detect NUMA configuration from ACPI SRAT table
    // For now, assume single node
    if let Some(ref mut nodes) = *nodes {
        nodes.push(NumaNode {
            id: 0,
            memory_start: PhysAddr::new(0x100000),  // 1MB
            memory_end: PhysAddr::new(0x8000_0000), // 2GB
        });
    }
}

/// Get NUMA node count
pub fn numa_node_count() -> usize {
    NUMA_NODES
        .lock()
        .as_ref()
        .map(|n| n.len())
        .unwrap_or(1)
}

/// Get NUMA node for a physical address
pub fn get_numa_node(addr: PhysAddr) -> Option<u32> {
    let nodes = NUMA_NODES.lock();
    if let Some(ref nodes) = *nodes {
        for node in nodes {
            if addr >= node.memory_start && addr < node.memory_end {
                return Some(node.id);
            }
        }
    }
    None
}

/// Initialize paging subsystem
pub fn init() {
    init_numa();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virt_addr_alignment() {
        let addr = VirtAddr::new(0x1234);
        assert_eq!(addr.align_down(0x1000).as_u64(), 0x1000);
        assert_eq!(addr.align_up(0x1000).as_u64(), 0x2000);
    }

    #[test]
    fn test_phys_addr_zone() {
        assert_eq!(PhysAddr::new(0x8000).zone(), MemoryZone::Dma);
        assert_eq!(PhysAddr::new(0x200_0000).zone(), MemoryZone::Normal);
        assert_eq!(PhysAddr::new(0x4000_0000).zone(), MemoryZone::High);
    }

    #[test]
    fn test_huge_page_sizes() {
        assert_eq!(HugePageSize::Size2MB.size(), 2 * 1024 * 1024);
        assert_eq!(HugePageSize::Size1GB.size(), 1024 * 1024 * 1024);
    }
}

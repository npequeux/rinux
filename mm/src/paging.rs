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

/// Page table entry with flags
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const PRESENT: u64 = 1 << 0;
    const WRITABLE: u64 = 1 << 1;
    const USER: u64 = 1 << 2;
    const WRITE_THROUGH: u64 = 1 << 3;
    const NO_CACHE: u64 = 1 << 4;
    const ACCESSED: u64 = 1 << 5;
    const DIRTY: u64 = 1 << 6;
    const HUGE: u64 = 1 << 7;
    const GLOBAL: u64 = 1 << 8;
    const NO_EXECUTE: u64 = 1 << 63;
    const ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

    pub const fn new() -> Self {
        PageTableEntry(0)
    }

    pub fn is_present(&self) -> bool {
        (self.0 & Self::PRESENT) != 0
    }

    pub fn is_writable(&self) -> bool {
        (self.0 & Self::WRITABLE) != 0
    }

    pub fn is_user(&self) -> bool {
        (self.0 & Self::USER) != 0
    }

    pub fn is_huge(&self) -> bool {
        (self.0 & Self::HUGE) != 0
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & Self::ADDR_MASK)
    }

    pub fn set(&mut self, addr: PhysAddr, writable: bool, user: bool) {
        let mut flags = Self::PRESENT;
        if writable {
            flags |= Self::WRITABLE;
        }
        if user {
            flags |= Self::USER;
        }
        self.0 = (addr.as_u64() & Self::ADDR_MASK) | flags;
    }

    pub fn set_huge(&mut self, addr: PhysAddr, writable: bool, user: bool) {
        let mut flags = Self::PRESENT | Self::HUGE;
        if writable {
            flags |= Self::WRITABLE;
        }
        if user {
            flags |= Self::USER;
        }
        self.0 = (addr.as_u64() & Self::ADDR_MASK) | flags;
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Page table with 512 entries
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); 512],
        }
    }

    pub fn get_entry(&self, index: usize) -> Option<&PageTableEntry> {
        self.entries.get(index)
    }

    pub fn get_entry_mut(&mut self, index: usize) -> Option<&mut PageTableEntry> {
        self.entries.get_mut(index)
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
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

    /// Get page table indices from a virtual address
    fn page_indices(virt: VirtAddr) -> [usize; 4] {
        let addr = virt.as_u64();
        [
            ((addr >> 39) & 0x1FF) as usize, // PML4
            ((addr >> 30) & 0x1FF) as usize, // PDP
            ((addr >> 21) & 0x1FF) as usize, // PD
            ((addr >> 12) & 0x1FF) as usize, // PT
        ]
    }
    
    /// Safely access a page table by physical address
    ///
    /// # Safety
    ///
    /// This assumes identity mapping for page tables. In a production kernel,
    /// this should use a dedicated page table mapping region or recursive mapping.
    /// The caller must ensure the physical address points to a valid page table.
    unsafe fn access_page_table(phys: PhysAddr) -> &'static mut PageTable {
        // TODO: In a complete implementation, map page tables to a known virtual
        // address range instead of assuming identity mapping
        &mut *(phys.as_u64() as *mut PageTable)
    }

    /// Map a virtual page to a physical frame
    ///
    /// This walks the page table hierarchy and creates page tables as needed.
    pub fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        writable: bool,
        user: bool,
    ) -> Result<(), &'static str> {
        #[cfg(target_arch = "x86_64")]
        {
            let indices = Self::page_indices(virt);
            
            // Walk page tables, creating them if needed
            let mut current_table_phys = self.root;
            
            // For each level (except the last), ensure the next level exists
            for level in 0..3 {
                // SAFETY: We assume identity mapping for page tables. This is a limitation
                // of the current implementation and should be improved with proper mapping.
                let table = unsafe { Self::access_page_table(current_table_phys) };
                
                let entry = table.get_entry_mut(indices[level])
                    .ok_or("Invalid page table index")?;
                
                if !entry.is_present() {
                    // Allocate a new page table
                    let new_frame = allocate_frame()
                        .ok_or("Out of memory")?;
                    
                    // Zero the new page table
                    let new_table_ptr = new_frame.start_address() as *mut PageTable;
                    unsafe {
                        core::ptr::write_bytes(new_table_ptr, 0, 1);
                    }
                    
                    // Set the entry to point to the new table
                    entry.set(PhysAddr::new(new_frame.start_address()), true, user);
                }
                
                current_table_phys = entry.addr();
            }
            
            // Now map the final page
            let table_ptr = current_table_phys.as_u64() as *mut PageTable;
            let table = unsafe { &mut *table_ptr };
            
            let entry = table.get_entry_mut(indices[3])
                .ok_or("Invalid page table index")?;
            
            if entry.is_present() {
                return Err("Page already mapped");
            }
            
            entry.set(phys, writable, user);
            
            // Flush TLB for this address
            tlb::shootdown_all(virt.as_u64());
            
            Ok(())
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (virt, phys, writable, user);
            Err("Paging not supported on this architecture")
        }
    }

    /// Unmap a virtual page
    pub fn unmap_page(&mut self, virt: VirtAddr) -> Result<Frame, &'static str> {
        #[cfg(target_arch = "x86_64")]
        {
            let indices = Self::page_indices(virt);
            
            // Walk to the final page table
            let mut current_table_phys = self.root;
            
            for level in 0..3 {
                let table_ptr = current_table_phys.as_u64() as *const PageTable;
                let table = unsafe { &*table_ptr };
                
                let entry = table.get_entry(indices[level])
                    .ok_or("Invalid page table index")?;
                
                if !entry.is_present() {
                    return Err("Page not mapped");
                }
                
                current_table_phys = entry.addr();
            }
            
            // Unmap the page
            let table_ptr = current_table_phys.as_u64() as *mut PageTable;
            let table = unsafe { &mut *table_ptr };
            
            let entry = table.get_entry_mut(indices[3])
                .ok_or("Invalid page table index")?;
            
            if !entry.is_present() {
                return Err("Page not mapped");
            }
            
            let phys_addr = entry.addr();
            let frame = Frame::containing_address(phys_addr.as_u64());
            
            entry.clear();
            
            // Flush TLB
            tlb::shootdown_all(virt.as_u64());
            
            Ok(frame)
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = virt;
            Err("Paging not supported on this architecture")
        }
    }

    /// Translate a virtual address to physical
    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        #[cfg(target_arch = "x86_64")]
        {
            let indices = Self::page_indices(virt);
            let mut current_table_phys = self.root;
            
            // Walk the page tables
            for level in 0..4 {
                let table_ptr = current_table_phys.as_u64() as *const PageTable;
                let table = unsafe { &*table_ptr };
                
                let entry = table.get_entry(indices[level])?;
                
                if !entry.is_present() {
                    return None;
                }
                
                // Check for huge pages at level 2 (1GB) or level 3 (2MB)
                if level >= 2 && entry.is_huge() {
                    let page_offset = virt.as_u64() & ((1 << (12 + 9 * (3 - level))) - 1);
                    return Some(PhysAddr::new(entry.addr().as_u64() + page_offset));
                }
                
                if level == 3 {
                    // Final level - add page offset
                    let page_offset = virt.page_offset();
                    return Some(PhysAddr::new(entry.addr().as_u64() + page_offset));
                }
                
                current_table_phys = entry.addr();
            }
            
            None
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = virt;
            None
        }
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

        #[cfg(target_arch = "x86_64")]
        {
            let indices = Self::page_indices(virt);
            let mut current_table_phys = self.root;
            
            // Determine how many levels to walk (2 for 1GB, 3 for 2MB)
            let target_level = match size {
                HugePageSize::Size1GB => 2,
                HugePageSize::Size2MB => 3,
            };
            
            // Walk to the target level
            for level in 0..target_level {
                let table_ptr = current_table_phys.as_u64() as *mut PageTable;
                let table = unsafe { &mut *table_ptr };
                
                let entry = table.get_entry_mut(indices[level])
                    .ok_or("Invalid page table index")?;
                
                if !entry.is_present() {
                    let new_frame = allocate_frame()
                        .ok_or("Out of memory")?;
                    
                    let new_table_ptr = new_frame.start_address() as *mut PageTable;
                    unsafe {
                        core::ptr::write_bytes(new_table_ptr, 0, 1);
                    }
                    
                    entry.set(PhysAddr::new(new_frame.start_address()), true, user);
                }
                
                current_table_phys = entry.addr();
            }
            
            // Set huge page entry
            let table_ptr = current_table_phys.as_u64() as *mut PageTable;
            let table = unsafe { &mut *table_ptr };
            
            let entry = table.get_entry_mut(indices[target_level])
                .ok_or("Invalid page table index")?;
            
            if entry.is_present() {
                return Err("Page already mapped");
            }
            
            entry.set_huge(phys, writable, user);
            
            // Flush TLB
            tlb::shootdown_all(virt.as_u64());
            
            Ok(())
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (virt, phys, size, writable, user);
            Err("Paging not supported on this architecture")
        }
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

//! Paging
//!
//! Page table management and virtual memory.

use core::arch::asm;

/// Page table entry flags
bitflags::bitflags! {
    pub struct PageTableFlags: u64 {
        const PRESENT        = 1 << 0;
        const WRITABLE       = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH  = 1 << 3;
        const NO_CACHE       = 1 << 4;
        const ACCESSED       = 1 << 5;
        const DIRTY          = 1 << 6;
        const HUGE_PAGE      = 1 << 7;
        const GLOBAL         = 1 << 8;
        const NO_EXECUTE     = 1 << 63;
    }
}

/// Page table entry
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn new() -> Self {
        PageTableEntry(0)
    }
    
    pub fn is_present(&self) -> bool {
        self.flags().contains(PageTableFlags::PRESENT)
    }
    
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }
    
    pub fn addr(&self) -> u64 {
        self.0 & 0x000F_FFFF_FFFF_F000
    }
    
    pub fn set(&mut self, addr: u64, flags: PageTableFlags) {
        self.0 = (addr & 0x000F_FFFF_FFFF_F000) | flags.bits();
    }
}

/// Page table
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
    
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = PageTableEntry::new();
        }
    }
}

/// Get CR3 (page table register)
pub fn read_cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) value, options(nomem, nostack));
    }
    value
}

/// Set CR3 (page table register)
pub unsafe fn write_cr3(value: u64) {
    asm!("mov cr3, {}", in(reg) value, options(nostack));
}

/// Flush TLB for a specific page
pub fn flush_tlb(addr: u64) {
    unsafe {
        asm!("invlpg [{}]", in(reg) addr, options(nostack));
    }
}

/// Flush entire TLB
pub fn flush_tlb_all() {
    unsafe {
        let cr3 = read_cr3();
        write_cr3(cr3);
    }
}

/// Initialize paging
pub fn init() {
    // Page tables should already be set up by bootloader
    // Here we can verify and enhance them if needed
}

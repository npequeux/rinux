//! Paging
//!
//! Page table management and virtual memory.

use core::arch::asm;

bitflags::bitflags! {
    /// Page table entry flags
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

impl Default for PageTableEntry {
    fn default() -> Self {
        Self::new()
    }
}

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

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
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
///
/// # Safety
///
/// The caller must ensure that the value is a valid physical address of a page table.
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
///
/// Verifies paging is enabled and checks CPU features.
/// The bootloader should have already set up initial page tables.
pub fn init() {
    // Verify paging is enabled (CR0.PG bit)
    let cr0 = read_cr0();
    if (cr0 & (1 << 31)) == 0 {
        panic!("Paging is not enabled!");
    }
    
    // Verify we're in 64-bit long mode (EFER.LMA bit)
    let efer = read_efer();
    if (efer & (1 << 10)) == 0 {
        panic!("Not in 64-bit long mode!");
    }
    
    // Check for NX bit support (EFER.NXE bit)
    let nx_enabled = (efer & (1 << 11)) != 0;
    
    // Get current page table
    let cr3 = read_cr3();
    
    // In early boot logging would use early_printk:
    // early_printk!("Paging initialized: CR3={:#x}, NX={}\n", cr3, nx_enabled);
    let _ = (cr3, nx_enabled); // Suppress unused warnings
}

/// Read CR0 register
fn read_cr0() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {}, cr0", out(reg) value, options(nomem, nostack));
    }
    value
}

/// Read EFER (Extended Feature Enable Register)
fn read_efer() -> u64 {
    let value: u64;
    unsafe {
        asm!(
            "mov ecx, 0xC0000080", // EFER MSR
            "rdmsr",
            "shl rdx, 32",
            "or rax, rdx",
            out("rax") value,
            out("rdx") _,
            out("ecx") _,
            options(nomem, nostack)
        );
    }
    value
}

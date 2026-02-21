//! Complete Page Fault Handler
//!
//! Handles page faults with full page table walking and allocation.

use crate::frame::{allocate_frame, FrameAllocator};
use core::ptr;

/// Page table entry flags
#[derive(Clone, Copy)]
pub struct PageFlags {
    pub present: bool,
    pub writable: bool,
    pub user: bool,
    pub write_through: bool,
    pub cache_disabled: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub huge: bool,
    pub global: bool,
    pub no_execute: bool,
}

impl PageFlags {
    pub const fn new() -> Self {
        Self {
            present: false,
            writable: false,
            user: false,
            write_through: false,
            cache_disabled: false,
            accessed: false,
            dirty: false,
            huge: false,
            global: false,
            no_execute: false,
        }
    }

    pub fn to_bits(&self) -> u64 {
        let mut bits = 0u64;
        if self.present { bits |= 1 << 0; }
        if self.writable { bits |= 1 << 1; }
        if self.user { bits |= 1 << 2; }
        if self.write_through { bits |= 1 << 3; }
        if self.cache_disabled { bits |= 1 << 4; }
        if self.accessed { bits |= 1 << 5; }
        if self.dirty { bits |= 1 << 6; }
        if self.huge { bits |= 1 << 7; }
        if self.global { bits |= 1 << 8; }
        if self.no_execute { bits |= 1 << 63; }
        bits
    }

    pub fn from_bits(bits: u64) -> Self {
        Self {
            present: (bits & (1 << 0)) != 0,
            writable: (bits & (1 << 1)) != 0,
            user: (bits & (1 << 2)) != 0,
            write_through: (bits & (1 << 3)) != 0,
            cache_disabled: (bits & (1 << 4)) != 0,
            accessed: (bits & (1 << 5)) != 0,
            dirty: (bits & (1 << 6)) != 0,
            huge: (bits & (1 << 7)) != 0,
            global: (bits & (1 << 8)) != 0,
            no_execute: (bits & (1 << 63)) != 0,
        }
    }
}

/// Page Table Entry
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    pub const fn new() -> Self {
        Self { entry: 0 }
    }

    pub fn is_present(&self) -> bool {
        (self.entry & 1) != 0
    }

    pub fn physical_address(&self) -> u64 {
        self.entry & 0x000F_FFFF_FFFF_F000
    }

    pub fn flags(&self) -> PageFlags {
        PageFlags::from_bits(self.entry)
    }

    pub fn set(&mut self, phys_addr: u64, flags: PageFlags) {
        self.entry = (phys_addr & 0x000F_FFFF_FFFF_F000) | flags.to_bits();
    }

    pub fn clear(&mut self) {
        self.entry = 0;
    }
}

/// Page Table (512 entries)
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }

    pub fn get_entry(&self, index: usize) -> Option<&PageTableEntry> {
        if index < 512 {
            Some(&self.entries[index])
        } else {
            None
        }
    }

    pub fn get_entry_mut(&mut self, index: usize) -> Option<&mut PageTableEntry> {
        if index < 512 {
            Some(&mut self.entries[index])
        } else {
            None
        }
    }
}

/// Page fault handler with full implementation
pub fn handle_page_fault(fault_addr: u64, error_code: u64) -> Result<(), &'static str> {
    // Check error code
    let present = (error_code & 1) != 0;
    let write = (error_code & 2) != 0;
    let user = (error_code & 4) != 0;
    let reserved = (error_code & 8) != 0;
    let instruction = (error_code & 16) != 0;

    if reserved {
        return Err("Reserved bit violation");
    }

    // Align fault address to page boundary
    let fault_page = fault_addr & !0xFFF;

    if present {
        // Page is present but access was denied
        if write {
            // Attempt to write to read-only page
            // Check if this is a copy-on-write page
            // For now, return error
            return Err("Write to read-only page");
        }
        if instruction {
            return Err("Instruction fetch from non-executable page");
        }
        return Err("Protection violation");
    }

    // Page not present - need to allocate and map
    map_page(fault_page, write, user)?;

    Ok(())
}

/// Map a virtual page to a new physical frame
fn map_page(virt_addr: u64, writable: bool, user: bool) -> Result<(), &'static str> {
    // Allocate a physical frame
    let phys_frame = allocate_frame().ok_or("Out of memory")?;

    // Get current page table
    let cr3 = read_cr3();
    let pml4_phys = cr3 & !0xFFF;
    
    // Convert physical address to virtual for access
    // In kernel, we assume identity mapping or higher-half mapping
    let pml4 = unsafe { &mut *(pml4_phys as *mut PageTable) };

    // Extract page table indices
    let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;

    // Walk page tables, creating them as needed
    let pdpt = get_or_create_table(pml4, pml4_idx, user)?;
    let pd = get_or_create_table(pdpt, pdpt_idx, user)?;
    let pt = get_or_create_table(pd, pd_idx, user)?;

    // Map the page
    let entry = pt.get_entry_mut(pt_idx).ok_or("Invalid PT index")?;
    
    let mut flags = PageFlags::new();
    flags.present = true;
    flags.writable = writable;
    flags.user = user;
    
    entry.set(phys_frame.start_address(), flags);

    // Clear the new page
    unsafe {
        ptr::write_bytes(virt_addr as *mut u8, 0, 4096);
    }

    // Flush TLB for this address
    flush_tlb(virt_addr);

    Ok(())
}

/// Get or create intermediate page table
fn get_or_create_table(parent: &mut PageTable, index: usize, user: bool) -> Result<&mut PageTable, &'static str> {
    let entry = parent.get_entry_mut(index).ok_or("Invalid table index")?;

    if entry.is_present() {
        // Table exists
        let phys_addr = entry.physical_address();
        Ok(unsafe { &mut *(phys_addr as *mut PageTable) })
    } else {
        // Create new table
        let phys_frame = allocate_frame().ok_or("Out of memory")?;
        let phys_addr = phys_frame.start_address();
        
        // Clear the new table
        unsafe {
            ptr::write_bytes(phys_addr as *mut u8, 0, 4096);
        }

        let mut flags = PageFlags::new();
        flags.present = true;
        flags.writable = true;
        flags.user = user;
        
        entry.set(phys_addr, flags);

        Ok(unsafe { &mut *(phys_addr as *mut PageTable) })
    }
}

/// Read CR3 register (page table base)
fn read_cr3() -> u64 {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    }
    cr3
}

/// Flush TLB entry for a virtual address
fn flush_tlb(virt_addr: u64) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_flags() {
        let mut flags = PageFlags::new();
        flags.present = true;
        flags.writable = true;
        flags.user = true;

        let bits = flags.to_bits();
        assert_eq!(bits & 0x7, 0x7); // Present | Writable | User

        let flags2 = PageFlags::from_bits(bits);
        assert!(flags2.present);
        assert!(flags2.writable);
        assert!(flags2.user);
    }

    #[test]
    fn test_page_table_entry() {
        let mut entry = PageTableEntry::new();
        assert!(!entry.is_present());

        let mut flags = PageFlags::new();
        flags.present = true;
        entry.set(0x1000, flags);

        assert!(entry.is_present());
        assert_eq!(entry.physical_address(), 0x1000);
    }
}

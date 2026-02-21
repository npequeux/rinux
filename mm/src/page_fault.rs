//! Page Fault Handler
//!
//! Handles page faults and manages virtual memory.

use crate::arch::x86::paging::{PageTableEntry, PageTableFlags, PageTable};
use crate::mm::frame;

/// Page fault error code bits
pub mod error_code {
    pub const PRESENT: u64 = 1 << 0;       // 0 = not present, 1 = protection fault
    pub const WRITE: u64 = 1 << 1;         // 0 = read, 1 = write
    pub const USER: u64 = 1 << 2;          // 0 = kernel, 1 = user
    pub const RESERVED: u64 = 1 << 3;      // 1 = reserved bits set
    pub const INSTRUCTION: u64 = 1 << 4;   // 1 = instruction fetch
}

/// Page fault handler
///
/// # Arguments
///
/// * `fault_addr` - Virtual address that caused the fault
/// * `error_code` - Error code from CPU
pub fn handle_page_fault(fault_addr: u64, error_code: u64) -> Result<(), PageFaultError> {
    // Check what kind of fault this is
    let is_present = (error_code & error_code::PRESENT) != 0;
    let is_write = (error_code & error_code::WRITE) != 0;
    let is_user = (error_code & error_code::USER) != 0;
    let is_reserved = (error_code & error_code::RESERVED) != 0;
    let is_instruction = (error_code & error_code::INSTRUCTION) != 0;

    // Log the fault (in a real kernel, this would use printk)
    // kernel::printk::printk(&format!("Page fault at {:#x}, error code: {:#x}\n", fault_addr, error_code));

    if is_reserved {
        return Err(PageFaultError::ReservedBit);
    }

    if !is_present {
        // Page not present - might need to allocate
        return handle_not_present(fault_addr, is_write, is_user);
    }

    if is_write {
        // Write to read-only page
        return Err(PageFaultError::WriteToReadOnly);
    }

    if is_instruction {
        // Instruction fetch from non-executable page
        return Err(PageFaultError::InstructionFetch);
    }

    // Protection violation
    Err(PageFaultError::ProtectionViolation)
}

/// Handle a page not present fault
fn handle_not_present(fault_addr: u64, is_write: bool, is_user: bool) -> Result<(), PageFaultError> {
    // Align address to page boundary
    let page_addr = fault_addr & !0xFFF;

    // Check if this is in a valid memory region
    // For now, we'll allow kernel heap addresses
    if page_addr >= 0xFFFF_FF00_0000_0000 && page_addr < 0xFFFF_FF80_0000_0000 {
        // Allocate a physical frame
        let frame = frame::allocate_frame()
            .ok_or(PageFaultError::OutOfMemory)?;

        // Map the page
        // Note: This is simplified - a real implementation would walk the page tables
        map_page(page_addr, frame.start_address(), is_write, is_user)?;

        Ok(())
    } else if page_addr < 0x0000_8000_0000_0000 {
        // User space address - would need to check VMA (Virtual Memory Areas)
        // For now, return an error
        Err(PageFaultError::InvalidAddress)
    } else {
        // Invalid address
        Err(PageFaultError::InvalidAddress)
    }
}

/// Map a virtual page to a physical frame
fn map_page(virt_addr: u64, phys_addr: u64, writable: bool, user: bool) -> Result<(), PageFaultError> {
    // Get current page table from CR3
    let cr3 = crate::arch::x86::paging::read_cr3();
    let pml4_addr = cr3 & !0xFFF;

    // Extract page table indices
    let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
    let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
    let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;

    // This is a simplified version - a real implementation would:
    // 1. Walk the page tables, creating intermediate tables as needed
    // 2. Set up proper permissions
    // 3. Handle TLB flushing
    // For now, we'll just return success
    // TODO: Implement full page table walking and mapping

    // Flush TLB for this page
    crate::arch::x86::paging::flush_tlb(virt_addr);

    Ok(())
}

/// Page fault errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageFaultError {
    /// Reserved bit was set in page table
    ReservedBit,
    /// Write to read-only page
    WriteToReadOnly,
    /// Instruction fetch from non-executable page
    InstructionFetch,
    /// Protection violation
    ProtectionViolation,
    /// Invalid address
    InvalidAddress,
    /// Out of memory
    OutOfMemory,
    /// Page table error
    PageTableError,
}

/// Initialize page fault handling
pub fn init() {
    // Register page fault handler in IDT (Interrupt Descriptor Table)
    // This is done by the interrupt subsystem
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_parsing() {
        let error_code = error_code::PRESENT | error_code::WRITE | error_code::USER;
        assert_ne!(error_code & error_code::PRESENT, 0);
        assert_ne!(error_code & error_code::WRITE, 0);
        assert_ne!(error_code & error_code::USER, 0);
        assert_eq!(error_code & error_code::RESERVED, 0);
    }
}

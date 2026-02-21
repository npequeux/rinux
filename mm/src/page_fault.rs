//! Page Fault Handler
//!
//! Handles page faults and manages virtual memory.

use crate::frame;

/// Page fault error code bits
pub mod error_code {
    pub const PRESENT: u64 = 1 << 0; // 0 = not present, 1 = protection fault
    pub const WRITE: u64 = 1 << 1; // 0 = read, 1 = write
    pub const USER: u64 = 1 << 2; // 0 = kernel, 1 = user
    pub const RESERVED: u64 = 1 << 3; // 1 = reserved bits set
    pub const INSTRUCTION: u64 = 1 << 4; // 1 = instruction fetch
}

/// Virtual Memory Area - represents a contiguous range of virtual memory
pub struct VMA {
    start: u64,
    end: u64,
    flags: VMAFlags,
}

bitflags::bitflags! {
    /// VMA permission flags
    pub struct VMAFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC = 1 << 2;
        const SHARED = 1 << 3;
    }
}

impl VMA {
    pub fn new(start: u64, end: u64, flags: VMAFlags) -> Self {
        VMA { start, end, flags }
    }

    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }

    pub fn is_writable(&self) -> bool {
        self.flags.contains(VMAFlags::WRITE)
    }

    pub fn is_executable(&self) -> bool {
        self.flags.contains(VMAFlags::EXEC)
    }
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

    if is_reserved {
        return Err(PageFaultError::ReservedBit);
    }

    if !is_present {
        // Page not present - implement demand paging
        return handle_demand_paging(fault_addr, is_write, is_user, is_instruction);
    }

    if is_write {
        // Write to read-only page - might be copy-on-write
        return handle_write_protection(fault_addr, is_user);
    }

    if is_instruction {
        // Instruction fetch from non-executable page
        return Err(PageFaultError::InstructionFetch);
    }

    // Protection violation
    Err(PageFaultError::ProtectionViolation)
}

/// Handle demand paging - allocate page on first access
fn handle_demand_paging(
    fault_addr: u64,
    is_write: bool,
    is_user: bool,
    is_instruction: bool,
) -> Result<(), PageFaultError> {
    // Align address to page boundary
    let page_addr = fault_addr & !0xFFF;

    // Check if this is in a valid memory region
    let vma = find_vma(page_addr)?;

    // Verify permissions match the access type
    if is_write && !vma.is_writable() {
        return Err(PageFaultError::WriteToReadOnly);
    }

    if is_instruction && !vma.is_executable() {
        return Err(PageFaultError::InstructionFetch);
    }

    // Allocate a physical frame
    let frame = frame::allocate_frame().ok_or(PageFaultError::OutOfMemory)?;

    // Zero the frame for security
    unsafe {
        let ptr = frame.start_address() as *mut u8;
        core::ptr::write_bytes(ptr, 0, 4096);
    }

    // Map the page with appropriate permissions
    map_page(page_addr, frame.start_address(), is_write, is_user)?;

    Ok(())
}

/// Handle copy-on-write page fault
fn handle_write_protection(fault_addr: u64, is_user: bool) -> Result<(), PageFaultError> {
    let page_addr = fault_addr & !0xFFF;

    // Check if this is a copy-on-write page
    if is_copy_on_write(page_addr) {
        // Allocate a new frame
        let new_frame = frame::allocate_frame().ok_or(PageFaultError::OutOfMemory)?;

        // Copy the old page content to the new frame
        copy_page_content(page_addr, new_frame.start_address())?;

        // Remap the page to the new frame with write permissions
        remap_page(page_addr, new_frame.start_address(), true, is_user)?;

        Ok(())
    } else {
        Err(PageFaultError::WriteToReadOnly)
    }
}

/// Find the VMA containing the given address
fn find_vma(addr: u64) -> Result<VMA, PageFaultError> {
    // Simplified: define some basic VMAs
    // In a real kernel, these would be tracked per-process

    // Kernel heap
    if addr >= 0xFFFF_FF00_0000_0000 && addr < 0xFFFF_FF80_0000_0000 {
        return Ok(VMA::new(
            0xFFFF_FF00_0000_0000,
            0xFFFF_FF80_0000_0000,
            VMAFlags::READ | VMAFlags::WRITE,
        ));
    }

    // User space
    if addr < 0x0000_8000_0000_0000 {
        return Ok(VMA::new(
            0x0000_0000_0000_0000,
            0x0000_8000_0000_0000,
            VMAFlags::READ | VMAFlags::WRITE | VMAFlags::EXEC,
        ));
    }

    Err(PageFaultError::InvalidAddress)
}

/// Check if a page is marked copy-on-write
fn is_copy_on_write(_page_addr: u64) -> bool {
    // TODO: Track COW pages (could use page table software bits)
    false
}

/// Copy content from one page to another
fn copy_page_content(src_virt: u64, dst_phys: u64) -> Result<(), PageFaultError> {
    unsafe {
        let src = src_virt as *const u8;
        let dst = dst_phys as *mut u8;
        core::ptr::copy_nonoverlapping(src, dst, 4096);
    }
    Ok(())
}

/// Map a virtual page to a physical frame
fn map_page(
    virt_addr: u64,
    phys_addr: u64,
    writable: bool,
    user: bool,
) -> Result<(), PageFaultError> {
    // This would use the architecture-specific paging module
    // For now, this is a stub that will be integrated with arch/x86/paging.rs
    let _ = (virt_addr, phys_addr, writable, user);

    // TODO: Integrate with rinux_arch_x86::paging::PageMapper
    // let mut mapper = unsafe { PageMapper::new() };
    // let virt = VirtAddr::new(virt_addr);
    // let phys = PhysAddr::new(phys_addr);
    // let mut flags = PageTableFlags::PRESENT;
    // if writable { flags |= PageTableFlags::WRITABLE; }
    // if user { flags |= PageTableFlags::USER_ACCESSIBLE; }
    // mapper.map_page(virt, phys, flags)?;

    Ok(())
}

/// Remap an existing page to a new physical frame
fn remap_page(
    virt_addr: u64,
    phys_addr: u64,
    writable: bool,
    user: bool,
) -> Result<(), PageFaultError> {
    // Unmap old page
    // TODO: unmap_page(virt_addr)?;

    // Map to new frame
    map_page(virt_addr, phys_addr, writable, user)
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

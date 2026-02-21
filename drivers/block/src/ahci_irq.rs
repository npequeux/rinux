//! Interrupt-driven AHCI I/O
//!
//! Interrupt handling for AHCI storage devices

use spin::Mutex;
use alloc::vec::Vec;
use alloc::sync::Arc;

/// IRQ number for AHCI (typically 11 on PCI)
pub const AHCI_IRQ: u8 = 11;

/// Interrupt handler callback
type InterruptCallback = fn(irq: u8);

/// Registered interrupt handlers
static INTERRUPT_HANDLERS: Mutex<Vec<(u8, InterruptCallback)>> = Mutex::new(Vec::new());

/// Pending I/O completions
static PENDING_IO: Mutex<Vec<IoCompletion>> = Mutex::new(Vec::new());

/// I/O completion structure
#[derive(Clone)]
pub struct IoCompletion {
    /// Port number
    pub port: usize,
    /// Command slot
    pub slot: usize,
    /// Status code
    pub status: u32,
    /// Completed flag
    pub completed: bool,
}

impl IoCompletion {
    pub fn new(port: usize, slot: usize) -> Self {
        Self {
            port,
            slot,
            status: 0,
            completed: false,
        }
    }

    pub fn complete(&mut self, status: u32) {
        self.status = status;
        self.completed = true;
    }
}

/// Register an IRQ handler
pub fn register_irq_handler(irq: u8, handler: InterruptCallback) {
    let mut handlers = INTERRUPT_HANDLERS.lock();
    handlers.push((irq, handler));
}

/// Call interrupt handlers for the given IRQ
pub fn dispatch_irq(irq: u8) {
    let handlers = INTERRUPT_HANDLERS.lock();
    for (registered_irq, handler) in handlers.iter() {
        if *registered_irq == irq {
            handler(irq);
        }
    }
}

/// AHCI interrupt handler
fn ahci_interrupt_handler(_irq: u8) {
    // Read AHCI interrupt status
    // For each port with interrupt pending:
    //   - Read port interrupt status
    //   - Clear interrupt
    //   - Mark I/O completion as done
    
    let mut pending = PENDING_IO.lock();
    for completion in pending.iter_mut() {
        if !completion.completed {
            // In real implementation, check hardware status
            completion.complete(0); // Success
        }
    }
}

/// Add an I/O operation to track
pub fn add_pending_io(port: usize, slot: usize) -> Arc<Mutex<IoCompletion>> {
    let completion = IoCompletion::new(port, slot);
    let arc = Arc::new(Mutex::new(completion.clone()));
    
    let mut pending = PENDING_IO.lock();
    pending.push(completion);
    
    arc
}

/// Wait for I/O completion with timeout
pub fn wait_for_completion(
    completion: &Arc<Mutex<IoCompletion>>,
    timeout_ms: u32,
) -> Result<u32, &'static str> {
    // In real implementation, this would:
    // 1. Sleep/yield while waiting
    // 2. Check completion status periodically
    // 3. Return error on timeout
    
    let mut elapsed = 0;
    while elapsed < timeout_ms {
        let comp = completion.lock();
        if comp.completed {
            return Ok(comp.status);
        }
        drop(comp);
        
        // Yield to other tasks
        core::hint::spin_loop();
        elapsed += 1;
    }
    
    Err("I/O timeout")
}

/// Initialize interrupt-driven I/O
pub fn init() {
    // Register AHCI interrupt handler
    register_irq_handler(AHCI_IRQ, ahci_interrupt_handler);
    
    // Enable AHCI interrupts in hardware
    // This would be done in the AHCI driver initialization
}

/// Enable interrupts for a specific AHCI port
pub fn enable_port_interrupts(port_regs: *mut u8, port: usize) {
    unsafe {
        // Calculate interrupt enable register offset
        let ie_offset = 0x100 + (port * 0x80) + 0x14;
        let ie_reg = port_regs.add(ie_offset) as *mut u32;
        
        // Enable relevant interrupts:
        // - Device to Host Register FIS Interrupt (DHR)
        // - PIO Setup FIS Interrupt (PSI)
        // - DMA Setup FIS Interrupt (DSI)
        // - Set Device Bits Interrupt (SDB)
        let interrupt_mask = 0x00000001  // DHRE
                           | 0x00000002  // PSE
                           | 0x00000004  // DSE
                           | 0x00000008; // SDBE
        
        ie_reg.write_volatile(interrupt_mask);
    }
}

/// Disable interrupts for a specific AHCI port
pub fn disable_port_interrupts(port_regs: *mut u8, port: usize) {
    unsafe {
        let ie_offset = 0x100 + (port * 0x80) + 0x14;
        let ie_reg = port_regs.add(ie_offset) as *mut u32;
        ie_reg.write_volatile(0);
    }
}

/// Clear port interrupt status
pub fn clear_port_interrupts(port_regs: *mut u8, port: usize) {
    unsafe {
        // Read interrupt status
        let is_offset = 0x100 + (port * 0x80) + 0x10;
        let is_reg = port_regs.add(is_offset) as *mut u32;
        let status = is_reg.read_volatile();
        
        // Write back to clear (write-1-to-clear)
        is_reg.write_volatile(status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_completion() {
        let mut completion = IoCompletion::new(0, 0);
        assert!(!completion.completed);
        
        completion.complete(0);
        assert!(completion.completed);
        assert_eq!(completion.status, 0);
    }
}

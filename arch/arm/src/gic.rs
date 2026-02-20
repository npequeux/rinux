//! ARM64 Generic Interrupt Controller (GIC) Support
//!
//! Support for GICv2 and GICv3.

use core::ptr::{read_volatile, write_volatile};

/// GIC Distributor registers (GICv2)
pub mod gicd {
    pub const CTLR: usize = 0x000;
    pub const TYPER: usize = 0x004;
    pub const IIDR: usize = 0x008;
    pub const IGROUPR: usize = 0x080;
    pub const ISENABLER: usize = 0x100;
    pub const ICENABLER: usize = 0x180;
    pub const ISPENDR: usize = 0x200;
    pub const ICPENDR: usize = 0x280;
    pub const ISACTIVER: usize = 0x300;
    pub const ICACTIVER: usize = 0x380;
    pub const IPRIORITYR: usize = 0x400;
    pub const ITARGETSR: usize = 0x800;
    pub const ICFGR: usize = 0xC00;
    pub const SGIR: usize = 0xF00;
}

/// GIC CPU Interface registers (GICv2)
pub mod gicc {
    pub const CTLR: usize = 0x000;
    pub const PMR: usize = 0x004;
    pub const BPR: usize = 0x008;
    pub const IAR: usize = 0x00C;
    pub const EOIR: usize = 0x010;
    pub const RPR: usize = 0x014;
    pub const HPPIR: usize = 0x018;
}

static mut GICD_BASE: Option<u64> = None;
static mut GICC_BASE: Option<u64> = None;

/// Initialize GIC
pub fn init() {
    kernel::printk!("[ARM64] Initializing Generic Interrupt Controller...\n");
    
    // TODO: Detect GIC base addresses from device tree or hardcoded values
    // Common addresses for QEMU virt:
    // GICD: 0x08000000
    // GICC: 0x08010000
    
    kernel::printk!("[ARM64] GIC initialization (stub)\n");
}

/// Enable an interrupt
pub fn enable_interrupt(irq: u32) {
    // TODO: Enable interrupt in GICD_ISENABLER
    kernel::printk!("[ARM64] Enable interrupt {} (stub)\n", irq);
}

/// Disable an interrupt
pub fn disable_interrupt(irq: u32) {
    // TODO: Disable interrupt in GICD_ICENABLER
    kernel::printk!("[ARM64] Disable interrupt {} (stub)\n", irq);
}

/// Send End of Interrupt
pub fn send_eoi(irq: u32) {
    // TODO: Write to GICC_EOIR
}

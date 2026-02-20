//! Advanced Programmable Interrupt Controller (APIC)
//!
//! Support for local APIC and x2APIC modes.

use crate::long_mode::{rdmsr, wrmsr};
use core::ptr::{read_volatile, write_volatile};

/// APIC register offsets
pub mod reg {
    pub const ID: usize = 0x20;
    pub const VERSION: usize = 0x30;
    pub const TPR: usize = 0x80; // Task Priority Register
    pub const APR: usize = 0x90; // Arbitration Priority Register
    pub const PPR: usize = 0xA0; // Processor Priority Register
    pub const EOI: usize = 0xB0; // End of Interrupt
    pub const RRD: usize = 0xC0; // Remote Read Register
    pub const LDR: usize = 0xD0; // Logical Destination Register
    pub const DFR: usize = 0xE0; // Destination Format Register
    pub const SPURIOUS: usize = 0xF0; // Spurious Interrupt Vector Register
    pub const ESR: usize = 0x280; // Error Status Register
    pub const ICR_LOW: usize = 0x300; // Interrupt Command Register (low)
    pub const ICR_HIGH: usize = 0x310; // Interrupt Command Register (high)
    pub const LVT_TIMER: usize = 0x320; // LVT Timer Register
    pub const LVT_THERMAL: usize = 0x330; // LVT Thermal Sensor Register
    pub const LVT_PERF: usize = 0x340; // LVT Performance Monitoring
    pub const LVT_LINT0: usize = 0x350; // LVT LINT0 Register
    pub const LVT_LINT1: usize = 0x360; // LVT LINT1 Register
    pub const LVT_ERROR: usize = 0x370; // LVT Error Register
    pub const TIMER_INIT: usize = 0x380; // Timer Initial Count
    pub const TIMER_CURRENT: usize = 0x390; // Timer Current Count
    pub const TIMER_DIV: usize = 0x3E0; // Timer Divide Configuration
}

/// x2APIC MSR addresses
pub mod x2apic_msr {
    pub const BASE: u32 = 0x800;
    pub const ID: u32 = 0x802;
    pub const VERSION: u32 = 0x803;
    pub const TPR: u32 = 0x808;
    pub const PPR: u32 = 0x80A;
    pub const EOI: u32 = 0x80B;
    pub const LDR: u32 = 0x80D;
    pub const SPURIOUS: u32 = 0x80F;
    pub const ESR: u32 = 0x828;
    pub const ICR: u32 = 0x830;
    pub const LVT_TIMER: u32 = 0x832;
    pub const LVT_THERMAL: u32 = 0x833;
    pub const LVT_PERF: u32 = 0x834;
    pub const LVT_LINT0: u32 = 0x835;
    pub const LVT_LINT1: u32 = 0x836;
    pub const LVT_ERROR: u32 = 0x837;
    pub const TIMER_INIT: u32 = 0x838;
    pub const TIMER_CURRENT: u32 = 0x839;
    pub const TIMER_DIV: u32 = 0x83E;
}

/// APIC Base MSR
const IA32_APIC_BASE: u32 = 0x1B;

/// APIC Base MSR flags
const APIC_BASE_ENABLE: u64 = 1 << 11;
const APIC_BASE_X2APIC: u64 = 1 << 10;
const APIC_BASE_BSP: u64 = 1 << 8;

/// APIC mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApicMode {
    Disabled,
    XApic,
    X2Apic,
}

/// APIC state
static mut APIC_MODE: ApicMode = ApicMode::Disabled;
static mut APIC_BASE_ADDR: u64 = 0;

/// Get APIC base address from MSR
fn get_apic_base() -> (u64, bool, bool) {
    let base_msr = rdmsr(IA32_APIC_BASE);
    let addr = base_msr & 0xFFFF_FFFF_FFFF_F000;
    let enabled = (base_msr & APIC_BASE_ENABLE) != 0;
    let x2apic = (base_msr & APIC_BASE_X2APIC) != 0;
    (addr, enabled, x2apic)
}

/// Set APIC base address and mode
fn set_apic_base(addr: u64, enable: bool, x2apic: bool) {
    let mut value = addr & 0xFFFF_FFFF_FFFF_F000;
    if enable {
        value |= APIC_BASE_ENABLE;
    }
    if x2apic {
        value |= APIC_BASE_X2APIC;
    }
    wrmsr(IA32_APIC_BASE, value);
}

/// Read APIC register (xAPIC mode)
unsafe fn read_apic_reg(offset: usize) -> u32 {
    let addr = (APIC_BASE_ADDR + offset as u64) as *const u32;
    read_volatile(addr)
}

/// Write APIC register (xAPIC mode)
unsafe fn write_apic_reg(offset: usize, value: u32) {
    let addr = (APIC_BASE_ADDR + offset as u64) as *mut u32;
    write_volatile(addr, value);
}

/// Read APIC register (works for both xAPIC and x2APIC)
pub fn read_register(offset: usize) -> u32 {
    unsafe {
        match APIC_MODE {
            ApicMode::XApic => read_apic_reg(offset),
            ApicMode::X2Apic => {
                // Convert offset to MSR number
                let msr = x2apic_msr::BASE + (offset >> 4) as u32;
                rdmsr(msr) as u32
            }
            ApicMode::Disabled => 0,
        }
    }
}

/// Write APIC register (works for both xAPIC and x2APIC)
pub fn write_register(offset: usize, value: u32) {
    unsafe {
        match APIC_MODE {
            ApicMode::XApic => write_apic_reg(offset, value),
            ApicMode::X2Apic => {
                // Convert offset to MSR number
                let msr = x2apic_msr::BASE + (offset >> 4) as u32;
                wrmsr(msr, value as u64);
            }
            ApicMode::Disabled => {}
        }
    }
}

/// Check if APIC is supported
pub fn is_apic_supported() -> bool {
    use crate::cpu::cpuid;
    let (_, _, _, edx) = cpuid(1);
    (edx & (1 << 9)) != 0 // APIC bit
}

/// Check if x2APIC is supported
pub fn is_x2apic_supported() -> bool {
    use crate::cpu::cpuid;
    let (_, _, ecx, _) = cpuid(1);
    (ecx & (1 << 21)) != 0 // x2APIC bit
}

/// Initialize APIC in xAPIC mode
pub fn init_xapic() -> bool {
    if !is_apic_supported() {
        return false;
    }

    let (base, _, _) = get_apic_base();
    unsafe {
        APIC_BASE_ADDR = base;
        APIC_MODE = ApicMode::XApic;
    }

    // Enable APIC
    set_apic_base(base, true, false);

    // Enable APIC via spurious interrupt vector register
    let spurious = read_register(reg::SPURIOUS);
    write_register(reg::SPURIOUS, spurious | 0x100); // Enable bit

    rinux_kernel::printk!("[APIC] Initialized in xAPIC mode at {:#x}\n", base);
    true
}

/// Initialize APIC in x2APIC mode
pub fn init_x2apic() -> bool {
    if !is_apic_supported() || !is_x2apic_supported() {
        return false;
    }

    let (base, _, _) = get_apic_base();
    unsafe {
        APIC_MODE = ApicMode::X2Apic;
    }

    // Enable x2APIC
    set_apic_base(base, true, true);

    // Enable APIC via spurious interrupt vector register
    let spurious = rdmsr(x2apic_msr::SPURIOUS) as u32;
    wrmsr(x2apic_msr::SPURIOUS, (spurious | 0x100) as u64);

    rinux_kernel::printk!("[APIC] Initialized in x2APIC mode\n");
    true
}

/// Initialize APIC (tries x2APIC first, falls back to xAPIC)
pub fn init() {
    if init_x2apic() {
        return;
    }

    if init_xapic() {
        return;
    }

    rinux_kernel::printk!("[APIC] No APIC support available, using legacy PIC\n");
}

/// Send End of Interrupt signal
pub fn send_eoi() {
    write_register(reg::EOI, 0);
}

/// Get local APIC ID
pub fn get_id() -> u32 {
    match unsafe { APIC_MODE } {
        ApicMode::XApic => read_register(reg::ID) >> 24,
        ApicMode::X2Apic => rdmsr(x2apic_msr::ID) as u32,
        ApicMode::Disabled => 0,
    }
}

/// Get APIC version
pub fn get_version() -> u32 {
    read_register(reg::VERSION) & 0xFF
}

/// Get max LVT entry
pub fn get_max_lvt() -> u32 {
    (read_register(reg::VERSION) >> 16) & 0xFF
}

/// Get current APIC mode
pub fn get_mode() -> ApicMode {
    unsafe { APIC_MODE }
}

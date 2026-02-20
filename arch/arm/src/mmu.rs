//! ARM64 MMU (Memory Management Unit)
//!
//! Page table management and virtual memory for ARM64.

use core::arch::asm;

/// Read TCR_EL1 (Translation Control Register)
#[inline]
pub fn read_tcr() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, tcr_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write TCR_EL1
#[inline]
pub fn write_tcr(val: u64) {
    unsafe {
        asm!("msr tcr_el1, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read TTBR0_EL1 (Translation Table Base Register 0)
#[inline]
pub fn read_ttbr0() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, ttbr0_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write TTBR0_EL1
#[inline]
pub fn write_ttbr0(val: u64) {
    unsafe {
        asm!("msr ttbr0_el1, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read TTBR1_EL1 (Translation Table Base Register 1)
#[inline]
pub fn read_ttbr1() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, ttbr1_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write TTBR1_EL1
#[inline]
pub fn write_ttbr1(val: u64) {
    unsafe {
        asm!("msr ttbr1_el1, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read SCTLR_EL1 (System Control Register)
#[inline]
pub fn read_sctlr() -> u64 {
    let val: u64;
    unsafe {
        asm!("mrs {}, sctlr_el1", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write SCTLR_EL1
#[inline]
pub fn write_sctlr(val: u64) {
    unsafe {
        asm!("msr sctlr_el1, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Enable MMU
pub fn enable_mmu() {
    let mut sctlr = read_sctlr();
    sctlr |= 1; // Set M bit (MMU enable)
    write_sctlr(sctlr);
    crate::instruction_sync_barrier();
}

/// Disable MMU
pub fn disable_mmu() {
    let mut sctlr = read_sctlr();
    sctlr &= !1; // Clear M bit (MMU disable)
    write_sctlr(sctlr);
    crate::instruction_sync_barrier();
}

/// Check if MMU is enabled
pub fn is_mmu_enabled() -> bool {
    (read_sctlr() & 1) != 0
}

/// Initialize MMU
pub fn init() {
    kernel::printk!("[ARM64] Initializing MMU...\n");
    kernel::printk!("  MMU enabled: {}\n", is_mmu_enabled());
    kernel::printk!("  TTBR0: {:#018x}\n", read_ttbr0());
    kernel::printk!("  TTBR1: {:#018x}\n", read_ttbr1());
    // TODO: Setup page tables
    kernel::printk!("[ARM64] MMU initialized\n");
}

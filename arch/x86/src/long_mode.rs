//! Long Mode Setup and Transition
//!
//! Handles setting up and transitioning to x86_64 long mode.

use core::arch::asm;

/// CR0 control register flags
pub mod cr0 {
    pub const PE: u64 = 1 << 0;  // Protection Enable
    pub const MP: u64 = 1 << 1;  // Monitor Coprocessor
    pub const EM: u64 = 1 << 2;  // Emulation
    pub const TS: u64 = 1 << 3;  // Task Switched
    pub const ET: u64 = 1 << 4;  // Extension Type
    pub const NE: u64 = 1 << 5;  // Numeric Error
    pub const WP: u64 = 1 << 16; // Write Protect
    pub const AM: u64 = 1 << 18; // Alignment Mask
    pub const NW: u64 = 1 << 29; // Not Write-through
    pub const CD: u64 = 1 << 30; // Cache Disable
    pub const PG: u64 = 1 << 31; // Paging
}

/// CR4 control register flags
pub mod cr4 {
    pub const PAE: u64 = 1 << 5;   // Physical Address Extension
    pub const PGE: u64 = 1 << 7;   // Page Global Enable
    pub const OSFXSR: u64 = 1 << 9; // OS FXSAVE/FXRSTOR support
    pub const OSXMMEXCPT: u64 = 1 << 10; // OS XMM exception support
    pub const OSXSAVE: u64 = 1 << 18; // OS XSAVE support
}

/// EFER (Extended Feature Enable Register) flags
pub mod efer {
    pub const SCE: u64 = 1 << 0;  // System Call Extensions
    pub const LME: u64 = 1 << 8;  // Long Mode Enable
    pub const LMA: u64 = 1 << 10; // Long Mode Active
    pub const NXE: u64 = 1 << 11; // No-Execute Enable
}

/// MSR (Model Specific Register) addresses
pub mod msr {
    pub const EFER: u32 = 0xC0000080;
    pub const STAR: u32 = 0xC0000081;
    pub const LSTAR: u32 = 0xC0000082;
    pub const CSTAR: u32 = 0xC0000083;
    pub const SFMASK: u32 = 0xC0000084;
}

/// Read CR0 register
#[inline]
pub fn read_cr0() -> u64 {
    let val: u64;
    unsafe {
        asm!("mov {}, cr0", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write CR0 register
#[inline]
pub fn write_cr0(val: u64) {
    unsafe {
        asm!("mov cr0, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read CR3 register (page table base)
#[inline]
pub fn read_cr3() -> u64 {
    let val: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write CR3 register (page table base)
#[inline]
pub fn write_cr3(val: u64) {
    unsafe {
        asm!("mov cr3, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read CR2 register (page fault linear address)
#[inline]
pub fn read_cr2() -> u64 {
    let val: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Read CR4 register
#[inline]
pub fn read_cr4() -> u64 {
    let val: u64;
    unsafe {
        asm!("mov {}, cr4", out(reg) val, options(nomem, nostack));
    }
    val
}

/// Write CR4 register
#[inline]
pub fn write_cr4(val: u64) {
    unsafe {
        asm!("mov cr4, {}", in(reg) val, options(nomem, nostack));
    }
}

/// Read MSR (Model Specific Register)
#[inline]
pub fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack)
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Write MSR (Model Specific Register)
#[inline]
pub fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack)
        );
    }
}

/// Check if long mode is already enabled
pub fn is_long_mode_enabled() -> bool {
    let efer = rdmsr(msr::EFER);
    (efer & efer::LMA) != 0
}

/// Enable long mode
///
/// This function sets up the necessary control registers and EFER flags
/// to enable x86_64 long mode.
pub fn enable_long_mode() {
    // Enable PAE (Physical Address Extension) in CR4
    let mut cr4 = read_cr4();
    cr4 |= cr4::PAE;
    write_cr4(cr4);

    // Enable Long Mode in EFER
    let mut efer = rdmsr(msr::EFER);
    efer |= efer::LME | efer::NXE; // Long Mode Enable + No-Execute Enable
    wrmsr(msr::EFER, efer);

    // Enable paging in CR0 to activate long mode
    let mut cr0 = read_cr0();
    cr0 |= cr0::PG | cr0::PE | cr0::WP; // Paging + Protection + Write Protect
    write_cr0(cr0);
}

/// Initialize long mode support
pub fn init() {
    if is_long_mode_enabled() {
        kernel::printk!("[LONG_MODE] Already in long mode\n");
    } else {
        kernel::printk!("[LONG_MODE] Enabling long mode...\n");
        enable_long_mode();
        kernel::printk!("[LONG_MODE] Long mode enabled\n");
    }

    // Enable additional CPU features
    let mut cr4 = read_cr4();
    cr4 |= cr4::OSFXSR | cr4::OSXMMEXCPT; // Enable FXSAVE/FXRSTOR and XMM exceptions
    write_cr4(cr4);

    kernel::printk!("[LONG_MODE] CPU features enabled\n");
}

/// Get current protection level
pub fn get_protection_level() -> u8 {
    let cs: u16;
    unsafe {
        asm!("mov {:x}, cs", out(reg) cs, options(nomem, nostack));
    }
    (cs & 0x3) as u8
}

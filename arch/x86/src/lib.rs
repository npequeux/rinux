//! x86_64 Architecture Support
//!
//! Architecture-specific code for x86_64.

#![no_std]
#![feature(abi_x86_interrupt)]

pub mod boot;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod paging;

/// Initialize x86_64 architecture
pub fn init() {
    // Setup GDT
    gdt::init();

    // Setup IDT
    idt::init();

    // Initialize interrupts
    interrupts::init();

    // Setup paging
    paging::init();
}

/// Halt the CPU
#[inline(always)]
pub fn halt() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

/// Enable interrupts
#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

/// Disable interrupts
#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}

/// Check if interrupts are enabled
#[inline(always)]
pub fn interrupts_enabled() -> bool {
    let flags: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem));
    }
    (flags & 0x200) != 0
}

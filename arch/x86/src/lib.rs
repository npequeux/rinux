//! x86_64 Architecture Support
//!
//! Architecture-specific code for x86_64.

#![no_std]
#![feature(abi_x86_interrupt)]

pub mod apic;
pub mod boot;
pub mod cpu;
pub mod exceptions;
pub mod fpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod io;
pub mod long_mode;
pub mod memory;
pub mod paging;
pub mod smp;
pub mod timers;

/// Initialize x86_64 architecture
pub fn init() {
    // Setup long mode (if not already)
    long_mode::init();

    // Setup GDT
    gdt::init();

    // Setup IDT
    idt::init();

    // Initialize APIC (or fall back to PIC)
    apic::init();
    
    // Initialize interrupts
    interrupts::init();

    // Initialize FPU/SSE
    fpu::init();

    // Setup paging
    paging::init();

    // Initialize timers (TSC, HPET)
    timers::init();

    // Initialize SMP
    smp::init();
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

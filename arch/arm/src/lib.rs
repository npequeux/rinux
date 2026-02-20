//! ARM64 (AArch64) Architecture Support
//!
//! Architecture-specific code for ARM64.

#![no_std]

pub mod boot;
pub mod cpu;
pub mod exceptions;
pub mod gic;
pub mod interrupts;
pub mod memory;
pub mod mmu;
pub mod timers;

/// Initialize ARM64 architecture
pub fn init() {
    kernel::printk!("[ARM64] Initializing architecture...\n");
    
    // Initialize CPU features
    cpu::init();
    
    // Initialize exceptions
    exceptions::init();
    
    // Initialize GIC (Generic Interrupt Controller)
    gic::init();
    
    // Initialize MMU
    mmu::init();
    
    // Initialize timers
    timers::init();
    
    kernel::printk!("[ARM64] Initialization complete\n");
}

/// Halt the CPU
#[inline(always)]
pub fn halt() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}

/// Enable interrupts
#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("msr daifclr, #2", options(nomem, nostack));
    }
}

/// Disable interrupts
#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("msr daifset, #2", options(nomem, nostack));
    }
}

/// Check if interrupts are enabled
#[inline(always)]
pub fn interrupts_enabled() -> bool {
    let daif: u64;
    unsafe {
        core::arch::asm!("mrs {}, daif", out(reg) daif, options(nomem, nostack));
    }
    (daif & (1 << 7)) == 0 // IRQ mask bit
}

/// Memory barrier
#[inline(always)]
pub fn memory_barrier() {
    unsafe {
        core::arch::asm!("dmb sy", options(nomem, nostack));
    }
}

/// Data synchronization barrier
#[inline(always)]
pub fn data_sync_barrier() {
    unsafe {
        core::arch::asm!("dsb sy", options(nomem, nostack));
    }
}

/// Instruction synchronization barrier
#[inline(always)]
pub fn instruction_sync_barrier() {
    unsafe {
        core::arch::asm!("isb", options(nomem, nostack));
    }
}

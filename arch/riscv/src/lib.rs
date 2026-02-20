//! RISC-V 64 Architecture Support
//!
//! Architecture-specific code for RISC-V 64-bit.

#![no_std]

pub mod boot;
pub mod cpu;
pub mod csr;
pub mod exceptions;
pub mod interrupts;
pub mod memory;
pub mod plic;
pub mod sbi;
pub mod timers;

/// Initialize RISC-V architecture
pub fn init() {
    kernel::printk!("[RISCV] Initializing architecture...\n");
    
    // Initialize CPU features
    cpu::init();
    
    // Initialize SBI (Supervisor Binary Interface)
    sbi::init();
    
    // Initialize exceptions and interrupts
    exceptions::init();
    interrupts::init();
    
    // Initialize PLIC (Platform-Level Interrupt Controller)
    plic::init();
    
    // Initialize timers
    timers::init();
    
    kernel::printk!("[RISCV] Initialization complete\n");
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
        // Set SIE bit in sstatus
        core::arch::asm!(
            "csrsi sstatus, 0x2",
            options(nomem, nostack)
        );
    }
}

/// Disable interrupts
#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        // Clear SIE bit in sstatus
        core::arch::asm!(
            "csrci sstatus, 0x2",
            options(nomem, nostack)
        );
    }
}

/// Check if interrupts are enabled
#[inline(always)]
pub fn interrupts_enabled() -> bool {
    let sstatus: usize;
    unsafe {
        core::arch::asm!(
            "csrr {}, sstatus",
            out(reg) sstatus,
            options(nomem, nostack)
        );
    }
    (sstatus & 0x2) != 0 // SIE bit
}

/// Memory fence
#[inline(always)]
pub fn fence() {
    unsafe {
        core::arch::asm!("fence", options(nomem, nostack));
    }
}

/// Fence with specific ordering
#[inline(always)]
pub fn fence_rw_rw() {
    unsafe {
        core::arch::asm!("fence rw, rw", options(nomem, nostack));
    }
}

/// Instruction fence
#[inline(always)]
pub fn fence_i() {
    unsafe {
        core::arch::asm!("fence.i", options(nomem, nostack));
    }
}

/// Supervisor fence for virtual memory
#[inline(always)]
pub fn sfence_vma() {
    unsafe {
        core::arch::asm!("sfence.vma", options(nomem, nostack));
    }
}

/// Supervisor fence for specific address
#[inline(always)]
pub fn sfence_vma_addr(addr: usize) {
    unsafe {
        core::arch::asm!(
            "sfence.vma {}, zero",
            in(reg) addr,
            options(nomem, nostack)
        );
    }
}

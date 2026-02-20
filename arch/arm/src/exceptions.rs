//! ARM64 Exception Handlers
//!
//! Exception and interrupt handling for ARM64.

use core::arch::asm;

/// Exception vectors must be 2KB aligned
#[repr(C, align(2048))]
pub struct ExceptionVectors {
    // Current EL with SP0
    curr_el_sp0_sync: [u8; 0x80],
    curr_el_sp0_irq: [u8; 0x80],
    curr_el_sp0_fiq: [u8; 0x80],
    curr_el_sp0_serror: [u8; 0x80],
    
    // Current EL with SPx
    curr_el_spx_sync: [u8; 0x80],
    curr_el_spx_irq: [u8; 0x80],
    curr_el_spx_fiq: [u8; 0x80],
    curr_el_spx_serror: [u8; 0x80],
    
    // Lower EL using AArch64
    lower_el_aarch64_sync: [u8; 0x80],
    lower_el_aarch64_irq: [u8; 0x80],
    lower_el_aarch64_fiq: [u8; 0x80],
    lower_el_aarch64_serror: [u8; 0x80],
    
    // Lower EL using AArch32
    lower_el_aarch32_sync: [u8; 0x80],
    lower_el_aarch32_irq: [u8; 0x80],
    lower_el_aarch32_fiq: [u8; 0x80],
    lower_el_aarch32_serror: [u8; 0x80],
}

/// Set exception vector base address
pub fn set_vbar(addr: u64) {
    unsafe {
        asm!("msr vbar_el1, {}", in(reg) addr, options(nomem, nostack));
    }
}

/// Get exception vector base address
pub fn get_vbar() -> u64 {
    let addr: u64;
    unsafe {
        asm!("mrs {}, vbar_el1", out(reg) addr, options(nomem, nostack));
    }
    addr
}

/// Initialize exception handling
pub fn init() {
    kernel::printk!("[ARM64] Initializing exception handling...\n");
    // TODO: Setup exception vectors
    kernel::printk!("[ARM64] Exception handling initialized\n");
}

//! ARM64 Boot Code
//!
//! Early boot initialization for ARM64.

use core::arch::asm;

/// Boot entry point
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Clear BSS
    extern "C" {
        static mut __bss_start: u8;
        static mut __bss_end: u8;
    }
    
    let bss_start = &mut __bss_start as *mut u8;
    let bss_end = &mut __bss_end as *mut u8;
    let bss_size = bss_end as usize - bss_start as usize;
    
    core::ptr::write_bytes(bss_start, 0, bss_size);
    
    // Initialize architecture
    crate::init();
    
    // Jump to kernel main
    extern "C" {
        fn kernel_main() -> !;
    }
    kernel_main();
}

/// Exception level
#[derive(Debug, Clone, Copy)]
pub enum ExceptionLevel {
    EL0,
    EL1,
    EL2,
    EL3,
}

/// Get current exception level
pub fn current_exception_level() -> ExceptionLevel {
    let el: u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) el, options(nomem, nostack));
    }
    match (el >> 2) & 0x3 {
        0 => ExceptionLevel::EL0,
        1 => ExceptionLevel::EL1,
        2 => ExceptionLevel::EL2,
        3 => ExceptionLevel::EL3,
        _ => unreachable!(),
    }
}

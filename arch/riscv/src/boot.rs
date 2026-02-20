//! RISC-V Boot Code

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

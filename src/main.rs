//! Rinux Kernel Main Entry Point
//!
//! This is the main entry point for the Rinux kernel.

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

use core::panic::PanicInfo;

/// Kernel entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize early printk
    rinux_kernel::printk::init();
    
    rinux_kernel::printk::printk("Rinux kernel starting...\n");
    rinux_kernel::printk::printk("Version: 0.1.0\n");
    
    // Initialize architecture-specific components
    rinux_arch_x86::init();
    
    // Initialize memory management
    rinux_mm::init();
    
    // Initialize kernel subsystems
    rinux_kernel::init();
    
    rinux_kernel::printk::printk("Rinux kernel initialization complete!\n");
    
    // Enter main kernel loop
    loop {
        rinux_arch_x86::halt();
    }
}

/// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use rinux_kernel::printk::printk;
    
    printk("\n\n!!! KERNEL PANIC !!!\n");
    
    if let Some(location) = info.location() {
        printk("Location: ");
        printk(location.file());
        printk(":");
        // TODO: Convert line number to string
        printk("\n");
    }
    
    if let Some(message) = info.message() {
        printk("Message: ");
        // TODO: Format message
        printk("\n");
    }
    
    loop {
        rinux_arch_x86::halt();
    }
}

//! Rinux Kernel Core
//!
//! Core kernel functionality and initialization.

#![no_std]

pub mod printk;
pub mod init;
pub mod panic;
pub mod types;
pub mod process;

use core::sync::atomic::{AtomicBool, Ordering};

static KERNEL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the kernel
pub fn init() {
    if KERNEL_INITIALIZED.load(Ordering::Acquire) {
        printk::printk("Warning: Kernel already initialized\n");
        return;
    }
    
    printk::printk("Initializing kernel subsystems...\n");
    
    // Initialize subsystems
    init::early_init();
    
    KERNEL_INITIALIZED.store(true, Ordering::Release);
    printk::printk("Kernel subsystems initialized\n");
}

/// Check if kernel is initialized
pub fn is_initialized() -> bool {
    KERNEL_INITIALIZED.load(Ordering::Acquire)
}

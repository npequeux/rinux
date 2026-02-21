//! Rinux Kernel Core
//!
//! Core kernel functionality and initialization.

#![no_std]

extern crate alloc;
extern crate rinux_mm as mm;

pub mod fs;
pub mod init;
pub mod ipc;
pub mod panic;
pub mod printk;
pub mod process;
pub mod signal;
pub mod syscall;
pub mod tests;
pub mod time;
pub mod types;

use core::sync::atomic::{AtomicBool, Ordering};

static KERNEL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Kernel panic macro - wraps core::panic!
#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => {
        core::panic!($($arg)*)
    };
}

/// Initialize the kernel
pub fn init() {
    if KERNEL_INITIALIZED.load(Ordering::Acquire) {
        printk::printk("Warning: Kernel already initialized\n");
        return;
    }

    printk::printk("Initializing kernel subsystems...\n");

    // Initialize subsystems
    init::early_init();

    // Initialize time subsystem
    time::init();

    // Initialize file system
    fs::init();

    // Initialize signal handling
    signal::init();

    // Initialize IPC
    ipc::init();

    // Initialize scheduler
    process::sched::init();

    // Initialize syscall interface
    syscall::init();

    KERNEL_INITIALIZED.store(true, Ordering::Release);
    printk::printk("Kernel subsystems initialized\n");
}

/// Check if kernel is initialized
pub fn is_initialized() -> bool {
    KERNEL_INITIALIZED.load(Ordering::Acquire)
}

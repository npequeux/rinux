//! Memory Management
//!
//! Physical and virtual memory management.

#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod allocator;
pub mod frame;
pub mod heap;
pub mod vmalloc;
pub mod slab;
pub mod page_fault;

use core::sync::atomic::{AtomicBool, Ordering};

static MM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize memory management
pub fn init() {
    if MM_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    // Initialize frame allocator
    frame::init();

    // Initialize heap allocator
    heap::init();

    MM_INITIALIZED.store(true, Ordering::Release);
}

/// Check if memory management is initialized
pub fn is_initialized() -> bool {
    MM_INITIALIZED.load(Ordering::Acquire)
}

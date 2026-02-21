//! Memory Management
//!
//! Physical and virtual memory management.

#![no_std]
#![cfg_attr(not(test), feature(alloc_error_handler))]

extern crate alloc;

pub mod allocator;
pub mod frame;
pub mod heap;
pub mod mmap;
pub mod oom;
pub mod page_fault;
pub mod page_handler;
pub mod paging;
pub mod slab;
pub mod swap;
pub mod vmalloc;

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

    // Initialize vmalloc
    vmalloc::init();

    // Initialize paging support
    paging::init();

    // Initialize page fault handler
    page_fault::init();

    // Initialize OOM killer
    oom::init();

    // Initialize swap support
    swap::init();

    MM_INITIALIZED.store(true, Ordering::Release);
}

/// Check if memory management is initialized
pub fn is_initialized() -> bool {
    MM_INITIALIZED.load(Ordering::Acquire)
}

//! Signal Handling
//!
//! POSIX signal support for processes.

pub mod handler;
mod sig_types;

pub use handler::{SignalHandler, SignalHandlerFn};
pub use sig_types::{Signal, SignalSet};

use core::sync::atomic::{AtomicBool, Ordering};

static SIGNAL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize signal subsystem
pub fn init() {
    if SIGNAL_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    handler::init();

    SIGNAL_INITIALIZED.store(true, Ordering::Release);
    crate::printk::printk("  Signal subsystem initialized\n");
}

/// Check if signal subsystem is initialized
pub fn is_initialized() -> bool {
    SIGNAL_INITIALIZED.load(Ordering::Acquire)
}

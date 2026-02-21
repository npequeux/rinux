//! Inter-Process Communication
//!
//! IPC mechanisms including pipes, message queues, and shared memory.

pub mod pipe;
pub mod shm;

pub use pipe::{Pipe, PipeEnd};
pub use shm::{SharedMemorySegment, ShmId};

use core::sync::atomic::{AtomicBool, Ordering};

static IPC_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize IPC subsystem
pub fn init() {
    if IPC_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    pipe::init();
    shm::init();

    IPC_INITIALIZED.store(true, Ordering::Release);
    crate::printk::printk("  IPC subsystem initialized\n");
}

/// Check if IPC is initialized
pub fn is_initialized() -> bool {
    IPC_INITIALIZED.load(Ordering::Acquire)
}

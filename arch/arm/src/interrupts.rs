//! ARM64 Interrupt Management

/// Initialize interrupts
pub fn init() {
    kernel::printk!("[ARM64] Initializing interrupts...\n");
    crate::gic::init();
    crate::enable_interrupts();
    kernel::printk!("[ARM64] Interrupts initialized\n");
}

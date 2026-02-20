//! Kernel Panic Handler
//!
//! Handles kernel panics and crashes.

use crate::printk::printk;

/// Handle a kernel panic
pub fn handle_panic(info: &str, file: &str, _line: u32) -> ! {
    printk("\n\n");
    printk("=====================================\n");
    printk("!!!      KERNEL PANIC      !!!\n");
    printk("=====================================\n");
    printk("\nPanic in: ");
    printk(file);
    printk("\n");
    printk("Info: ");
    printk(info);
    printk("\n");
    printk("=====================================\n");

    // Halt all CPUs
    loop {
        unsafe {
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}

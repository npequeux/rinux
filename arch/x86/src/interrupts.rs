//! Interrupt Management
//!
//! Interrupt handling and management.

/// Initialize interrupt controllers
pub fn init() {
    // Initialize PIC
    init_pic();
}

/// Initialize the 8259 PIC
fn init_pic() {
    use crate::io::{inb, outb};

    unsafe {
        // Save masks
        let mask1 = inb(0x21);
        let mask2 = inb(0xA1);

        // Start initialization sequence
        outb(0x20, 0x11);
        outb(0xA0, 0x11);

        // Set vector offsets
        outb(0x21, 0x20); // Master PIC vector offset
        outb(0xA1, 0x28); // Slave PIC vector offset

        // Configure chaining
        outb(0x21, 0x04);
        outb(0xA1, 0x02);

        // Set mode
        outb(0x21, 0x01);
        outb(0xA1, 0x01);

        // Restore masks
        outb(0x21, mask1);
        outb(0xA1, mask2);
    }
}

/// Enable an IRQ
pub fn enable_irq(irq: u8) {
    use crate::io::{inb, outb};

    unsafe {
        let port = if irq < 8 { 0x21 } else { 0xA1 };
        let mask = inb(port);
        outb(port, mask & !(1 << (irq % 8)));
    }
}

/// Disable an IRQ
pub fn disable_irq(irq: u8) {
    use crate::io::{inb, outb};

    unsafe {
        let port = if irq < 8 { 0x21 } else { 0xA1 };
        let mask = inb(port);
        outb(port, mask | (1 << (irq % 8)));
    }
}

/// Send EOI (End of Interrupt)
pub fn send_eoi(irq: u8) {
    use crate::io::outb;

    unsafe {
        if irq >= 8 {
            outb(0xA0, 0x20);
        }
        outb(0x20, 0x20);
    }
}

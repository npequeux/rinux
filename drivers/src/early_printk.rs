//! Early printk - Debug output before console is initialized
//!
//! Provides debug output capability during early boot using serial port.

use core::fmt;

/// Early print function - outputs to serial port
///
/// This function can be used before the kernel console is fully initialized.
/// It writes directly to COM1 serial port (0x3F8) for debugging purposes.
///
/// # Examples
///
/// ```
/// early_printk("Boot stage 1\n");
/// ```
pub fn early_printk(s: &str) {
    // Write directly to COM1 port at 0x3F8
    for byte in s.bytes() {
        unsafe {
            // Wait for transmit buffer empty
            while (read_port(0x3FD) & 0x20) == 0 {}
            
            // Write byte
            write_port(0x3F8, byte);
        }
    }
}

/// Read from I/O port
#[inline]
unsafe fn read_port(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack)
    );
    value
}

/// Write to I/O port
#[inline]
unsafe fn write_port(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("al") value,
        in("dx") port,
        options(nomem, nostack)
    );
}

/// Early formatted print
///
/// # Examples
///
/// ```
/// early_printk!("Memory: {} MB", mem_size);
/// ```
#[macro_export]
macro_rules! early_printk {
    ($($arg:tt)*) => {
        $crate::early_printk::_print(format_args!($($arg)*))
    };
}

/// Early print with newline
#[macro_export]
macro_rules! early_printkln {
    () => ($crate::early_printk!("\n"));
    ($($arg:tt)*) => {
        $crate::early_printk::_print(format_args!($($arg)*));
        $crate::early_printk::_print(format_args!("\n"));
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    
    struct SerialWriter;
    
    impl fmt::Write for SerialWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            early_printk(s);
            Ok(())
        }
    }
    
    SerialWriter.write_fmt(args).unwrap();
}

/// Initialize early printk
///
/// Sets up serial port for early debugging output
pub fn init() {
    unsafe {
        // Initialize COM1 (0x3F8)
        // Disable interrupts
        write_port(0x3F8 + 1, 0x00);
        // Enable DLAB (set baud rate divisor)
        write_port(0x3F8 + 3, 0x80);
        // Set divisor to 3 (38400 baud)
        write_port(0x3F8 + 0, 0x03);
        write_port(0x3F8 + 1, 0x00);
        // 8 bits, no parity, one stop bit
        write_port(0x3F8 + 3, 0x03);
        // Enable FIFO, clear them, with 14-byte threshold
        write_port(0x3F8 + 2, 0xC7);
        // IRQs enabled, RTS/DSR set
        write_port(0x3F8 + 4, 0x0B);
    }
}


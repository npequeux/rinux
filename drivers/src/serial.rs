//! Serial Port Driver
//!
//! Driver for serial port communication (16550 UART).

use rinux_arch_x86::io::{inb, outb};
use spin::Mutex;

/// COM1 base port
const COM1: u16 = 0x3F8;

/// Serial port registers
const DATA: u16 = 0;
const INT_ENABLE: u16 = 1;
const FIFO_CTRL: u16 = 2;
const LINE_CTRL: u16 = 3;
const MODEM_CTRL: u16 = 4;
const LINE_STATUS: u16 = 5;

/// Global serial port lock
static SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort {
    base: COM1,
    initialized: false,
});

/// Serial port structure
pub struct SerialPort {
    base: u16,
    initialized: bool,
}

impl SerialPort {
    /// Initialize the serial port
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn init(&mut self) {
        if self.initialized {
            return;
        }

        // Disable interrupts
        outb(self.base + INT_ENABLE, 0x00);

        // Enable DLAB (set baud rate divisor)
        outb(self.base + LINE_CTRL, 0x80);

        // Set divisor to 3 (38400 baud)
        outb(self.base + DATA, 0x03);
        outb(self.base + INT_ENABLE, 0x00);

        // 8 bits, no parity, one stop bit
        outb(self.base + LINE_CTRL, 0x03);

        // Enable FIFO, clear them, with 14-byte threshold
        outb(self.base + FIFO_CTRL, 0xC7);

        // Enable interrupts, RTS/DSR set
        outb(self.base + MODEM_CTRL, 0x0B);

        // Enable interrupts
        outb(self.base + INT_ENABLE, 0x01);

        self.initialized = true;
    }

    /// Check if transmit buffer is empty
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn is_transmit_empty(&self) -> bool {
        inb(self.base + LINE_STATUS) & 0x20 != 0
    }

    /// Check if data is available to read
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn is_data_available(&self) -> bool {
        inb(self.base + LINE_STATUS) & 0x01 != 0
    }

    /// Write a byte to serial port
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn write_byte(&self, byte: u8) {
        // Wait for transmit buffer to be empty
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        outb(self.base + DATA, byte);
    }

    /// Read a byte from serial port
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn read_byte(&self) -> Option<u8> {
        if self.is_data_available() {
            Some(inb(self.base + DATA))
        } else {
            None
        }
    }
}

/// Initialize serial port
pub fn init() {
    let mut serial = SERIAL.lock();
    unsafe {
        serial.init();
    }
}

/// Write a byte to serial port
pub fn write_byte(byte: u8) {
    let serial = SERIAL.lock();
    if serial.initialized {
        unsafe {
            serial.write_byte(byte);
        }
    }
}

/// Write a string to serial port
pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

/// Read a byte from serial port
pub fn read_byte() -> Option<u8> {
    let serial = SERIAL.lock();
    if serial.initialized {
        unsafe { serial.read_byte() }
    } else {
        None
    }
}

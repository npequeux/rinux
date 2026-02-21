//! Serial Port Driver
//!
//! Driver for serial port communication (16550 UART).

use rinux_arch_x86::io::{inb, outb};
use spin::Mutex;

/// COM port base addresses
const COM1: u16 = 0x3F8;
const COM2: u16 = 0x2F8;
const COM3: u16 = 0x3E8;
const COM4: u16 = 0x2E8;

/// Serial port registers (offsets from base)
const DATA: u16 = 0;
const INT_ENABLE: u16 = 1;
const FIFO_CTRL: u16 = 2;
const LINE_CTRL: u16 = 3;
const MODEM_CTRL: u16 = 4;
const LINE_STATUS: u16 = 5;
const MODEM_STATUS: u16 = 6;

/// Baud rate divisors
pub enum BaudRate {
    Baud115200 = 1,
    Baud57600 = 2,
    Baud38400 = 3,
    Baud19200 = 6,
    Baud9600 = 12,
    Baud4800 = 24,
    Baud2400 = 48,
}

/// Data bits configuration
pub enum DataBits {
    Bits5 = 0x00,
    Bits6 = 0x01,
    Bits7 = 0x02,
    Bits8 = 0x03,
}

/// Stop bits configuration
pub enum StopBits {
    One = 0x00,
    Two = 0x04,
}

/// Parity configuration
pub enum Parity {
    None = 0x00,
    Odd = 0x08,
    Even = 0x18,
    Mark = 0x28,
    Space = 0x38,
}

/// Global serial ports
static COM1_PORT: Mutex<SerialPort> = Mutex::new(SerialPort {
    base: COM1,
    initialized: false,
});
static COM2_PORT: Mutex<SerialPort> = Mutex::new(SerialPort {
    base: COM2,
    initialized: false,
});
static COM3_PORT: Mutex<SerialPort> = Mutex::new(SerialPort {
    base: COM3,
    initialized: false,
});
static COM4_PORT: Mutex<SerialPort> = Mutex::new(SerialPort {
    base: COM4,
    initialized: false,
});

/// COM port identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComPort {
    COM1,
    COM2,
    COM3,
    COM4,
}

impl ComPort {
    fn get_port(&self) -> &'static Mutex<SerialPort> {
        match self {
            ComPort::COM1 => &COM1_PORT,
            ComPort::COM2 => &COM2_PORT,
            ComPort::COM3 => &COM3_PORT,
            ComPort::COM4 => &COM4_PORT,
        }
    }
}

/// Serial port structure
pub struct SerialPort {
    base: u16,
    initialized: bool,
}

impl SerialPort {
    /// Initialize the serial port with custom settings
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn init_with_config(
        &mut self,
        baud_rate: BaudRate,
        data_bits: DataBits,
        stop_bits: StopBits,
        parity: Parity,
    ) {
        if self.initialized {
            return;
        }

        // Disable interrupts
        outb(self.base + INT_ENABLE, 0x00);

        // Enable DLAB (set baud rate divisor)
        outb(self.base + LINE_CTRL, 0x80);

        // Set baud rate divisor
        let divisor = baud_rate as u16;
        outb(self.base + DATA, (divisor & 0xFF) as u8);
        outb(self.base + INT_ENABLE, ((divisor >> 8) & 0xFF) as u8);

        // Set line control: data bits, stop bits, parity
        let line_ctrl = data_bits as u8 | stop_bits as u8 | parity as u8;
        outb(self.base + LINE_CTRL, line_ctrl);

        // Enable FIFO, clear them, with 14-byte threshold
        outb(self.base + FIFO_CTRL, 0xC7);

        // Enable interrupts, RTS/DSR set
        outb(self.base + MODEM_CTRL, 0x0B);

        // Enable interrupts
        outb(self.base + INT_ENABLE, 0x01);

        self.initialized = true;
    }

    /// Initialize the serial port with default settings (38400 baud, 8N1)
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn init(&mut self) {
        self.init_with_config(
            BaudRate::Baud38400,
            DataBits::Bits8,
            StopBits::One,
            Parity::None,
        );
    }

    /// Configure baud rate
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn set_baud_rate(&mut self, baud_rate: BaudRate) {
        if !self.initialized {
            return;
        }

        // Save current line control
        let line_ctrl = inb(self.base + LINE_CTRL);

        // Enable DLAB
        outb(self.base + LINE_CTRL, line_ctrl | 0x80);

        // Set baud rate divisor
        let divisor = baud_rate as u16;
        outb(self.base + DATA, (divisor & 0xFF) as u8);
        outb(self.base + INT_ENABLE, ((divisor >> 8) & 0xFF) as u8);

        // Restore line control
        outb(self.base + LINE_CTRL, line_ctrl);
    }

    /// Check if carrier detect is active
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn is_carrier_detected(&self) -> bool {
        inb(self.base + MODEM_STATUS) & 0x80 != 0
    }

    /// Check if ring indicator is active
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    #[allow(dead_code)]
    unsafe fn is_ring_indicator(&self) -> bool {
        inb(self.base + MODEM_STATUS) & 0x40 != 0
    }

    /// Check if data set ready
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn is_data_set_ready(&self) -> bool {
        inb(self.base + MODEM_STATUS) & 0x20 != 0
    }

    /// Check if clear to send
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn is_clear_to_send(&self) -> bool {
        inb(self.base + MODEM_STATUS) & 0x10 != 0
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

/// Initialize serial port (COM1 by default)
pub fn init() {
    init_port(ComPort::COM1);
}

/// Initialize a specific COM port
pub fn init_port(port: ComPort) {
    let mut serial = port.get_port().lock();
    unsafe {
        serial.init();
    }
}

/// Initialize a COM port with custom configuration
pub fn init_port_with_config(
    port: ComPort,
    baud_rate: BaudRate,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: Parity,
) {
    let mut serial = port.get_port().lock();
    unsafe {
        serial.init_with_config(baud_rate, data_bits, stop_bits, parity);
    }
}

/// Configure baud rate for a port
pub fn set_baud_rate(port: ComPort, baud_rate: BaudRate) {
    let mut serial = port.get_port().lock();
    unsafe {
        serial.set_baud_rate(baud_rate);
    }
}

/// Write a byte to serial port
pub fn write_byte(byte: u8) {
    write_byte_to(ComPort::COM1, byte);
}

/// Write a byte to a specific COM port
pub fn write_byte_to(port: ComPort, byte: u8) {
    let serial = port.get_port().lock();
    if serial.initialized {
        unsafe {
            serial.write_byte(byte);
        }
    }
}

/// Write a string to serial port
pub fn write_str(s: &str) {
    write_str_to(ComPort::COM1, s);
}

/// Write a string to a specific COM port
pub fn write_str_to(port: ComPort, s: &str) {
    for byte in s.bytes() {
        write_byte_to(port, byte);
    }
}

/// Read a byte from serial port
pub fn read_byte() -> Option<u8> {
    read_byte_from(ComPort::COM1)
}

/// Read a byte from a specific COM port
pub fn read_byte_from(port: ComPort) -> Option<u8> {
    let serial = port.get_port().lock();
    if serial.initialized {
        unsafe { serial.read_byte() }
    } else {
        None
    }
}

/// Read multiple bytes into a buffer (non-blocking)
pub fn read_bytes(buffer: &mut [u8]) -> usize {
    read_bytes_from(ComPort::COM1, buffer)
}

/// Read multiple bytes from a specific COM port into a buffer (non-blocking)
pub fn read_bytes_from(port: ComPort, buffer: &mut [u8]) -> usize {
    let mut count = 0;
    for byte_slot in buffer.iter_mut() {
        if let Some(byte) = read_byte_from(port) {
            *byte_slot = byte;
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Check if data is available on a port
pub fn is_data_available(port: ComPort) -> bool {
    let serial = port.get_port().lock();
    if serial.initialized {
        unsafe { serial.is_data_available() }
    } else {
        false
    }
}

/// Check modem status - carrier detect
pub fn is_carrier_detected(port: ComPort) -> bool {
    let serial = port.get_port().lock();
    if serial.initialized {
        unsafe { serial.is_carrier_detected() }
    } else {
        false
    }
}

/// Check modem status - data set ready
pub fn is_data_set_ready(port: ComPort) -> bool {
    let serial = port.get_port().lock();
    if serial.initialized {
        unsafe { serial.is_data_set_ready() }
    } else {
        false
    }
}

/// Check modem status - clear to send
pub fn is_clear_to_send(port: ComPort) -> bool {
    let serial = port.get_port().lock();
    if serial.initialized {
        unsafe { serial.is_clear_to_send() }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baud_rate_divisors() {
        assert_eq!(BaudRate::Baud115200 as u16, 1);
        assert_eq!(BaudRate::Baud57600 as u16, 2);
        assert_eq!(BaudRate::Baud38400 as u16, 3);
        assert_eq!(BaudRate::Baud19200 as u16, 6);
        assert_eq!(BaudRate::Baud9600 as u16, 12);
    }

    #[test]
    fn test_com_port_variants() {
        // Test that each COM port returns a different static reference
        let com1 = ComPort::COM1.get_port() as *const _;
        let com2 = ComPort::COM2.get_port() as *const _;
        let com3 = ComPort::COM3.get_port() as *const _;
        let com4 = ComPort::COM4.get_port() as *const _;

        assert_ne!(com1, com2);
        assert_ne!(com1, com3);
        assert_ne!(com1, com4);
        assert_ne!(com2, com3);
    }

    #[test]
    fn test_data_bits_values() {
        assert_eq!(DataBits::Bits5 as u8, 0x00);
        assert_eq!(DataBits::Bits6 as u8, 0x01);
        assert_eq!(DataBits::Bits7 as u8, 0x02);
        assert_eq!(DataBits::Bits8 as u8, 0x03);
    }

    #[test]
    fn test_stop_bits_values() {
        assert_eq!(StopBits::One as u8, 0x00);
        assert_eq!(StopBits::Two as u8, 0x04);
    }

    #[test]
    fn test_parity_values() {
        assert_eq!(Parity::None as u8, 0x00);
        assert_eq!(Parity::Odd as u8, 0x08);
        assert_eq!(Parity::Even as u8, 0x18);
        assert_eq!(Parity::Mark as u8, 0x28);
        assert_eq!(Parity::Space as u8, 0x38);
    }
}

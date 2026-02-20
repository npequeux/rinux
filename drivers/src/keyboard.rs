//! Keyboard Driver
//!
//! PS/2 keyboard driver (8042 controller).

use rinux_arch_x86::io::{inb, outb};
use spin::Mutex;

/// PS/2 data port
const DATA_PORT: u16 = 0x60;
/// PS/2 status/command port
const STATUS_PORT: u16 = 0x64;
const COMMAND_PORT: u16 = 0x64;

/// Status register bits
const STATUS_OUTPUT_FULL: u8 = 0x01;
const STATUS_INPUT_FULL: u8 = 0x02;

/// Global keyboard state
static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard { initialized: false });

/// Keyboard structure
pub struct Keyboard {
    initialized: bool,
}

impl Keyboard {
    /// Initialize the keyboard
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn init(&mut self) {
        if self.initialized {
            return;
        }

        // Wait for input buffer to be empty
        self.wait_input_empty();

        // Disable devices
        outb(COMMAND_PORT, 0xAD); // Disable first port
        outb(COMMAND_PORT, 0xA7); // Disable second port

        // Flush output buffer
        let _ = inb(DATA_PORT);

        // Set controller configuration
        outb(COMMAND_PORT, 0x20); // Read configuration
        self.wait_output_full();
        let mut config = inb(DATA_PORT);

        // Enable interrupts and translation
        config |= 0x01; // Enable first port interrupt
        config &= !0x10; // Clear first port clock disable

        self.wait_input_empty();
        outb(COMMAND_PORT, 0x60); // Write configuration
        self.wait_input_empty();
        outb(DATA_PORT, config);

        // Enable first port
        self.wait_input_empty();
        outb(COMMAND_PORT, 0xAE);

        // Enable scanning
        self.wait_input_empty();
        outb(DATA_PORT, 0xF4);

        self.initialized = true;
    }

    /// Wait for input buffer to be empty
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn wait_input_empty(&self) {
        for _ in 0..1000 {
            if inb(STATUS_PORT) & STATUS_INPUT_FULL == 0 {
                return;
            }
            core::hint::spin_loop();
        }
    }

    /// Wait for output buffer to be full
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn wait_output_full(&self) {
        for _ in 0..1000 {
            if inb(STATUS_PORT) & STATUS_OUTPUT_FULL != 0 {
                return;
            }
            core::hint::spin_loop();
        }
    }

    /// Read a scancode from keyboard
    ///
    /// # Safety
    ///
    /// Performs I/O port read.
    unsafe fn read_scancode(&self) -> Option<u8> {
        if inb(STATUS_PORT) & STATUS_OUTPUT_FULL != 0 {
            Some(inb(DATA_PORT))
        } else {
            None
        }
    }
}

/// Initialize keyboard
pub fn init() {
    let mut kb = KEYBOARD.lock();
    unsafe {
        kb.init();
    }
}

/// Read a scancode from keyboard
///
/// Returns raw scancode if available.
pub fn read_scancode() -> Option<u8> {
    let kb = KEYBOARD.lock();
    if kb.initialized {
        unsafe { kb.read_scancode() }
    } else {
        None
    }
}

/// Read a key from keyboard
///
/// Returns ASCII character if available (simplified mapping).
pub fn read_key() -> Option<u8> {
    read_scancode().and_then(|scancode| {
        // Simplified scancode to ASCII mapping for Set 1
        // Only handle key presses (not releases)
        if scancode & 0x80 != 0 {
            return None; // Key release
        }

        match scancode {
            0x02 => Some(b'1'),
            0x03 => Some(b'2'),
            0x04 => Some(b'3'),
            0x05 => Some(b'4'),
            0x06 => Some(b'5'),
            0x07 => Some(b'6'),
            0x08 => Some(b'7'),
            0x09 => Some(b'8'),
            0x0A => Some(b'9'),
            0x0B => Some(b'0'),
            0x10 => Some(b'q'),
            0x11 => Some(b'w'),
            0x12 => Some(b'e'),
            0x13 => Some(b'r'),
            0x14 => Some(b't'),
            0x15 => Some(b'y'),
            0x16 => Some(b'u'),
            0x17 => Some(b'i'),
            0x18 => Some(b'o'),
            0x19 => Some(b'p'),
            0x1E => Some(b'a'),
            0x1F => Some(b's'),
            0x20 => Some(b'd'),
            0x21 => Some(b'f'),
            0x22 => Some(b'g'),
            0x23 => Some(b'h'),
            0x24 => Some(b'j'),
            0x25 => Some(b'k'),
            0x26 => Some(b'l'),
            0x2C => Some(b'z'),
            0x2D => Some(b'x'),
            0x2E => Some(b'c'),
            0x2F => Some(b'v'),
            0x30 => Some(b'b'),
            0x31 => Some(b'n'),
            0x32 => Some(b'm'),
            0x39 => Some(b' '),  // Space
            0x1C => Some(b'\n'), // Enter
            _ => None,
        }
    })
}

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

/// Timeout iterations for waiting operations
const TIMEOUT_ITERATIONS: u32 = 1000;

/// Keyboard commands
const KB_CMD_SET_LEDS: u8 = 0xED;
const KB_CMD_ECHO: u8 = 0xEE;
const KB_CMD_SCAN_CODE_SET: u8 = 0xF0;
const KB_CMD_IDENTIFY: u8 = 0xF2;
const KB_CMD_TYPEMATIC: u8 = 0xF3;
const KB_CMD_ENABLE_SCAN: u8 = 0xF4;
const KB_CMD_DISABLE_SCAN: u8 = 0xF5;
const KB_CMD_RESET: u8 = 0xFF;

/// Keyboard LEDs
const LED_SCROLL_LOCK: u8 = 0x01;
const LED_NUM_LOCK: u8 = 0x02;
const LED_CAPS_LOCK: u8 = 0x04;

/// Keyboard state flags
#[derive(Debug, Clone, Copy)]
pub struct KeyboardState {
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
        }
    }

    fn toggle_caps_lock(&mut self) {
        self.caps_lock = !self.caps_lock;
    }

    fn toggle_num_lock(&mut self) {
        self.num_lock = !self.num_lock;
    }

    fn toggle_scroll_lock(&mut self) {
        self.scroll_lock = !self.scroll_lock;
    }
}

/// Global keyboard state
static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {
    initialized: false,
    state: KeyboardState::new(),
});

/// Keyboard structure
pub struct Keyboard {
    initialized: bool,
    state: KeyboardState,
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
        for _ in 0..TIMEOUT_ITERATIONS {
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
        for _ in 0..TIMEOUT_ITERATIONS {
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

    /// Send command to keyboard
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn send_command(&self, cmd: u8) -> bool {
        self.wait_input_empty();
        outb(DATA_PORT, cmd);

        // Wait for ACK (0xFA)
        for _ in 0..TIMEOUT_ITERATIONS {
            if let Some(response) = self.read_scancode() {
                return response == 0xFA;
            }
            core::hint::spin_loop();
        }
        false
    }

    /// Set keyboard LEDs
    ///
    /// # Safety
    ///
    /// Performs I/O port operations.
    unsafe fn set_leds(&self, leds: u8) {
        if self.send_command(KB_CMD_SET_LEDS) {
            self.wait_input_empty();
            outb(DATA_PORT, leds);
            // Wait for ACK
            for _ in 0..TIMEOUT_ITERATIONS {
                if let Some(response) = self.read_scancode() {
                    if response == 0xFA {
                        break;
                    }
                }
                core::hint::spin_loop();
            }
        }
    }

    /// Update LED state based on keyboard state
    unsafe fn update_leds(&self) {
        let mut leds = 0u8;
        if self.state.scroll_lock {
            leds |= LED_SCROLL_LOCK;
        }
        if self.state.num_lock {
            leds |= LED_NUM_LOCK;
        }
        if self.state.caps_lock {
            leds |= LED_CAPS_LOCK;
        }
        self.set_leds(leds);
    }

    /// Process a scancode and update keyboard state
    unsafe fn process_scancode(&mut self, scancode: u8) {
        // Handle key releases (high bit set)
        let released = (scancode & 0x80) != 0;
        let scancode = scancode & 0x7F;

        match scancode {
            0x2A | 0x36 => self.state.shift_pressed = !released, // Left/Right Shift
            0x1D => self.state.ctrl_pressed = !released,         // Ctrl
            0x38 => self.state.alt_pressed = !released,          // Alt
            0x3A if !released => {
                // Caps Lock (toggle on press)
                self.state.toggle_caps_lock();
                self.update_leds();
            }
            0x45 if !released => {
                // Num Lock (toggle on press)
                self.state.toggle_num_lock();
                self.update_leds();
            }
            0x46 if !released => {
                // Scroll Lock (toggle on press)
                self.state.toggle_scroll_lock();
                self.update_leds();
            }
            _ => {}
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
    let mut kb = KEYBOARD.lock();
    if kb.initialized {
        let scancode = unsafe { kb.read_scancode() };
        if let Some(sc) = scancode {
            unsafe {
                kb.process_scancode(sc);
            }
        }
        scancode
    } else {
        None
    }
}

/// Get current keyboard state
pub fn get_state() -> KeyboardState {
    let kb = KEYBOARD.lock();
    kb.state
}

/// Set keyboard LED state
pub fn set_leds(scroll_lock: bool, num_lock: bool, caps_lock: bool) {
    let mut kb = KEYBOARD.lock();
    if kb.initialized {
        kb.state.scroll_lock = scroll_lock;
        kb.state.num_lock = num_lock;
        kb.state.caps_lock = caps_lock;
        unsafe {
            kb.update_leds();
        }
    }
}

/// Read a key from keyboard with modifier support
///
/// Returns ASCII character if available with shift/caps lock support.
pub fn read_key() -> Option<u8> {
    let mut kb = KEYBOARD.lock();
    if !kb.initialized {
        return None;
    }

    let scancode = unsafe { kb.read_scancode() };
    if let Some(sc) = scancode {
        unsafe {
            kb.process_scancode(sc);
        }

        // Only handle key presses (not releases)
        if sc & 0x80 != 0 {
            return None;
        }

        let is_shifted = kb.state.shift_pressed;
        let is_caps = kb.state.caps_lock;

        // Scancode to ASCII mapping with shift support
        scancode_to_ascii(sc, is_shifted, is_caps)
    } else {
        None
    }
}

/// Convert scancode to ASCII with shift and caps lock support
fn scancode_to_ascii(scancode: u8, shift: bool, caps: bool) -> Option<u8> {
    match scancode {
        // Number row
        0x02 if shift => Some(b'!'),
        0x02 => Some(b'1'),
        0x03 if shift => Some(b'@'),
        0x03 => Some(b'2'),
        0x04 if shift => Some(b'#'),
        0x04 => Some(b'3'),
        0x05 if shift => Some(b'$'),
        0x05 => Some(b'4'),
        0x06 if shift => Some(b'%'),
        0x06 => Some(b'5'),
        0x07 if shift => Some(b'^'),
        0x07 => Some(b'6'),
        0x08 if shift => Some(b'&'),
        0x08 => Some(b'7'),
        0x09 if shift => Some(b'*'),
        0x09 => Some(b'8'),
        0x0A if shift => Some(b'('),
        0x0A => Some(b'9'),
        0x0B if shift => Some(b')'),
        0x0B => Some(b'0'),
        0x0C if shift => Some(b'_'),
        0x0C => Some(b'-'),
        0x0D if shift => Some(b'+'),
        0x0D => Some(b'='),

        // Letters - Q row
        0x10 => Some(if shift ^ caps { b'Q' } else { b'q' }),
        0x11 => Some(if shift ^ caps { b'W' } else { b'w' }),
        0x12 => Some(if shift ^ caps { b'E' } else { b'e' }),
        0x13 => Some(if shift ^ caps { b'R' } else { b'r' }),
        0x14 => Some(if shift ^ caps { b'T' } else { b't' }),
        0x15 => Some(if shift ^ caps { b'Y' } else { b'y' }),
        0x16 => Some(if shift ^ caps { b'U' } else { b'u' }),
        0x17 => Some(if shift ^ caps { b'I' } else { b'i' }),
        0x18 => Some(if shift ^ caps { b'O' } else { b'o' }),
        0x19 => Some(if shift ^ caps { b'P' } else { b'p' }),
        0x1A if shift => Some(b'{'),
        0x1A => Some(b'['),
        0x1B if shift => Some(b'}'),
        0x1B => Some(b']'),

        // Letters - A row
        0x1E => Some(if shift ^ caps { b'A' } else { b'a' }),
        0x1F => Some(if shift ^ caps { b'S' } else { b's' }),
        0x20 => Some(if shift ^ caps { b'D' } else { b'd' }),
        0x21 => Some(if shift ^ caps { b'F' } else { b'f' }),
        0x22 => Some(if shift ^ caps { b'G' } else { b'g' }),
        0x23 => Some(if shift ^ caps { b'H' } else { b'h' }),
        0x24 => Some(if shift ^ caps { b'J' } else { b'j' }),
        0x25 => Some(if shift ^ caps { b'K' } else { b'k' }),
        0x26 => Some(if shift ^ caps { b'L' } else { b'l' }),
        0x27 if shift => Some(b':'),
        0x27 => Some(b';'),
        0x28 if shift => Some(b'"'),
        0x28 => Some(b'\''),
        0x29 if shift => Some(b'~'),
        0x29 => Some(b'`'),

        // Letters - Z row
        0x2B if shift => Some(b'|'),
        0x2B => Some(b'\\'),
        0x2C => Some(if shift ^ caps { b'Z' } else { b'z' }),
        0x2D => Some(if shift ^ caps { b'X' } else { b'x' }),
        0x2E => Some(if shift ^ caps { b'C' } else { b'c' }),
        0x2F => Some(if shift ^ caps { b'V' } else { b'v' }),
        0x30 => Some(if shift ^ caps { b'B' } else { b'b' }),
        0x31 => Some(if shift ^ caps { b'N' } else { b'n' }),
        0x32 => Some(if shift ^ caps { b'M' } else { b'm' }),
        0x33 if shift => Some(b'<'),
        0x33 => Some(b','),
        0x34 if shift => Some(b'>'),
        0x34 => Some(b'.'),
        0x35 if shift => Some(b'?'),
        0x35 => Some(b'/'),

        // Special keys
        0x39 => Some(b' '),    // Space
        0x1C => Some(b'\n'),   // Enter
        0x0E => Some(b'\x08'), // Backspace
        0x0F => Some(b'\t'),   // Tab

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state_new() {
        let state = KeyboardState::new();
        assert!(!state.shift_pressed);
        assert!(!state.ctrl_pressed);
        assert!(!state.alt_pressed);
        assert!(!state.caps_lock);
        assert!(!state.num_lock);
        assert!(!state.scroll_lock);
    }

    #[test]
    fn test_keyboard_state_toggles() {
        let mut state = KeyboardState::new();

        state.toggle_caps_lock();
        assert!(state.caps_lock);
        state.toggle_caps_lock();
        assert!(!state.caps_lock);

        state.toggle_num_lock();
        assert!(state.num_lock);

        state.toggle_scroll_lock();
        assert!(state.scroll_lock);
    }

    #[test]
    fn test_scancode_to_ascii_numbers() {
        // Test number keys without shift
        assert_eq!(scancode_to_ascii(0x02, false, false), Some(b'1'));
        assert_eq!(scancode_to_ascii(0x03, false, false), Some(b'2'));
        assert_eq!(scancode_to_ascii(0x0B, false, false), Some(b'0'));

        // Test number keys with shift
        assert_eq!(scancode_to_ascii(0x02, true, false), Some(b'!'));
        assert_eq!(scancode_to_ascii(0x03, true, false), Some(b'@'));
        assert_eq!(scancode_to_ascii(0x0B, true, false), Some(b')'));
    }

    #[test]
    fn test_scancode_to_ascii_letters() {
        // Test lowercase letters
        assert_eq!(scancode_to_ascii(0x1E, false, false), Some(b'a'));
        assert_eq!(scancode_to_ascii(0x30, false, false), Some(b'b'));
        assert_eq!(scancode_to_ascii(0x2C, false, false), Some(b'z'));

        // Test uppercase with shift
        assert_eq!(scancode_to_ascii(0x1E, true, false), Some(b'A'));
        assert_eq!(scancode_to_ascii(0x30, true, false), Some(b'B'));

        // Test uppercase with caps lock
        assert_eq!(scancode_to_ascii(0x1E, false, true), Some(b'A'));

        // Test shift+caps (should be lowercase)
        assert_eq!(scancode_to_ascii(0x1E, true, true), Some(b'a'));
    }

    #[test]
    fn test_scancode_to_ascii_special() {
        assert_eq!(scancode_to_ascii(0x39, false, false), Some(b' ')); // Space
        assert_eq!(scancode_to_ascii(0x1C, false, false), Some(b'\n')); // Enter
        assert_eq!(scancode_to_ascii(0x0E, false, false), Some(b'\x08')); // Backspace
        assert_eq!(scancode_to_ascii(0x0F, false, false), Some(b'\t')); // Tab
    }

    #[test]
    fn test_scancode_to_ascii_invalid() {
        // Test invalid scancode
        assert_eq!(scancode_to_ascii(0xFF, false, false), None);
    }
}

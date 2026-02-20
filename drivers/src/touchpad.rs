//! Touchpad/Trackpad Driver
//!
//! Support for PS/2 and I2C touchpads.

use rinux_arch_x86::io::{inb, outb};

/// PS/2 controller ports
const PS2_DATA: u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const PS2_COMMAND: u16 = 0x64;

/// PS/2 status register bits
const PS2_STATUS_OUTPUT_FULL: u8 = 0x01;
const PS2_STATUS_INPUT_FULL: u8 = 0x02;

/// PS/2 commands
const PS2_CMD_READ_CONFIG: u8 = 0x20;
const PS2_CMD_WRITE_CONFIG: u8 = 0x60;
const PS2_CMD_DISABLE_AUX: u8 = 0xA7;
const PS2_CMD_ENABLE_AUX: u8 = 0xA8;
const PS2_CMD_AUX_SEND: u8 = 0xD4;

/// Mouse commands
const MOUSE_CMD_SET_DEFAULTS: u8 = 0xF6;
const MOUSE_CMD_ENABLE_DATA: u8 = 0xF4;
const MOUSE_CMD_SET_SAMPLE_RATE: u8 = 0xF3;
const MOUSE_CMD_GET_DEVICE_ID: u8 = 0xF2;

/// Touchpad event
#[derive(Debug, Clone, Copy)]
pub struct TouchpadEvent {
    pub x: i16,
    pub y: i16,
    pub buttons: u8,
    pub z: i8, // Pressure
}

/// Touchpad device
pub struct Touchpad {
    device_id: u8,
    is_intellimouse: bool,
    packet_state: u8,
    packet_buffer: [u8; 4],
}

impl Touchpad {
    pub const fn new() -> Self {
        Self {
            device_id: 0,
            is_intellimouse: false,
            packet_state: 0,
            packet_buffer: [0; 4],
        }
    }

    /// Wait for PS/2 controller to be ready for input
    unsafe fn wait_input(&self) {
        for _ in 0..1000 {
            if (inb(PS2_STATUS) & PS2_STATUS_INPUT_FULL) == 0 {
                return;
            }
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }
    }

    /// Wait for PS/2 controller to have output
    unsafe fn wait_output(&self) {
        for _ in 0..1000 {
            if (inb(PS2_STATUS) & PS2_STATUS_OUTPUT_FULL) != 0 {
                return;
            }
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }
    }

    /// Send command to mouse/touchpad
    unsafe fn send_mouse_command(&self, cmd: u8) -> bool {
        self.wait_input();
        outb(PS2_COMMAND, PS2_CMD_AUX_SEND);

        self.wait_input();
        outb(PS2_DATA, cmd);

        self.wait_output();
        let response = inb(PS2_DATA);

        response == 0xFA // ACK
    }

    /// Initialize the touchpad
    pub unsafe fn init(&mut self) -> Result<(), &'static str> {
        // Enable auxiliary device (mouse/touchpad)
        self.wait_input();
        outb(PS2_COMMAND, PS2_CMD_ENABLE_AUX);

        // Set defaults
        if !self.send_mouse_command(MOUSE_CMD_SET_DEFAULTS) {
            return Err("Failed to set defaults");
        }

        // Try to enable IntelliMouse mode (scrollwheel)
        self.send_mouse_command(MOUSE_CMD_SET_SAMPLE_RATE);
        self.wait_input();
        outb(PS2_DATA, 200);

        self.send_mouse_command(MOUSE_CMD_SET_SAMPLE_RATE);
        self.wait_input();
        outb(PS2_DATA, 100);

        self.send_mouse_command(MOUSE_CMD_SET_SAMPLE_RATE);
        self.wait_input();
        outb(PS2_DATA, 80);

        // Get device ID
        self.send_mouse_command(MOUSE_CMD_GET_DEVICE_ID);
        self.wait_output();
        self.device_id = inb(PS2_DATA);

        self.is_intellimouse = self.device_id == 3 || self.device_id == 4;

        // Enable data reporting
        if !self.send_mouse_command(MOUSE_CMD_ENABLE_DATA) {
            return Err("Failed to enable data reporting");
        }

        Ok(())
    }

    /// Process a received byte
    pub fn process_byte(&mut self, byte: u8) -> Option<TouchpadEvent> {
        self.packet_buffer[self.packet_state as usize] = byte;
        self.packet_state += 1;

        let packet_size = if self.is_intellimouse { 4 } else { 3 };

        if self.packet_state >= packet_size {
            self.packet_state = 0;

            let buttons = self.packet_buffer[0] & 0x07;
            let x_sign = (self.packet_buffer[0] & 0x10) != 0;
            let y_sign = (self.packet_buffer[0] & 0x20) != 0;

            let mut x = self.packet_buffer[1] as i16;
            let mut y = self.packet_buffer[2] as i16;

            if x_sign {
                x = x.wrapping_sub(256);
            }
            if y_sign {
                y = y.wrapping_sub(256);
            }

            let z = if self.is_intellimouse {
                self.packet_buffer[3] as i8
            } else {
                0
            };

            return Some(TouchpadEvent {
                x,
                y: -y, // Invert Y for natural scrolling
                buttons,
                z,
            });
        }

        None
    }
}

/// Global touchpad instance
static mut TOUCHPAD: Touchpad = Touchpad::new();

/// Initialize touchpad
pub fn init() {
    rinux_kernel::printk::printk("  Initializing touchpad...\n");

    unsafe {
        match TOUCHPAD.init() {
            Ok(_) => {
                rinux_kernel::printk::printk("    Touchpad initialized (Device ID: ");
                if TOUCHPAD.is_intellimouse {
                    rinux_kernel::printk::printk("IntelliMouse)\n");
                } else {
                    rinux_kernel::printk::printk("Standard)\n");
                }
            }
            Err(e) => {
                rinux_kernel::printk::printk("    Touchpad init failed: ");
                rinux_kernel::printk::printk(e);
                rinux_kernel::printk::printk("\n");
            }
        }
    }
}

/// Get touchpad instance
pub fn get() -> &'static mut Touchpad {
    unsafe { &mut TOUCHPAD }
}

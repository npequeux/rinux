//! HID (Human Interface Device) Driver
//!
//! This module provides support for HID devices like keyboards, mice, and game controllers.

use super::UsbClass;

/// HID protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HidProtocol {
    None = 0x00,
    Keyboard = 0x01,
    Mouse = 0x02,
}

impl From<u8> for HidProtocol {
    fn from(value: u8) -> Self {
        match value {
            0x01 => HidProtocol::Keyboard,
            0x02 => HidProtocol::Mouse,
            _ => HidProtocol::None,
        }
    }
}

/// HID device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidDeviceType {
    Keyboard,
    Mouse,
    Gamepad,
    Joystick,
    Other,
}

/// HID device information
#[derive(Debug, Clone, Copy)]
pub struct HidDevice {
    pub device_address: u8,
    pub device_type: HidDeviceType,
    pub protocol: HidProtocol,
    pub endpoint: u8,
    pub max_packet_size: u16,
}

impl HidDevice {
    pub const fn new(device_address: u8, protocol: HidProtocol) -> Self {
        let device_type = match protocol {
            HidProtocol::Keyboard => HidDeviceType::Keyboard,
            HidProtocol::Mouse => HidDeviceType::Mouse,
            _ => HidDeviceType::Other,
        };

        Self {
            device_address,
            device_type,
            protocol,
            endpoint: 0,
            max_packet_size: 8,
        }
    }
}

/// Check if a device is an HID device
pub fn is_hid_device(class: u8, subclass: u8, protocol: u8) -> bool {
    class == UsbClass::Hid as u8 || (class == 0 && subclass == 0 && protocol > 0)
}

/// Initialize HID driver
pub fn init() {
    rinux_kernel::printk::printk("  HID: Initializing HID driver\n");
}

/// Register an HID device
pub fn register_hid_device(device_address: u8, protocol: u8) -> Result<(), &'static str> {
    let hid_protocol = HidProtocol::from(protocol);
    let hid_device = HidDevice::new(device_address, hid_protocol);

    rinux_kernel::printk::printk("  HID: Registered ");
    match hid_device.device_type {
        HidDeviceType::Keyboard => rinux_kernel::printk::printk("keyboard"),
        HidDeviceType::Mouse => rinux_kernel::printk::printk("mouse"),
        HidDeviceType::Gamepad => rinux_kernel::printk::printk("gamepad"),
        HidDeviceType::Joystick => rinux_kernel::printk::printk("joystick"),
        HidDeviceType::Other => rinux_kernel::printk::printk("HID device"),
    }
    rinux_kernel::printk::printk(" device\n");

    Ok(())
}

/// HID boot protocol keyboard report
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HidKeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub keycode: [u8; 6],
}

/// HID boot protocol mouse report
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HidMouseReport {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
    pub wheel: i8,
}

/// HID descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HidDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub hid_version: u16,
    pub country_code: u8,
    pub num_descriptors: u8,
    pub report_descriptor_type: u8,
    pub report_descriptor_length: u16,
}

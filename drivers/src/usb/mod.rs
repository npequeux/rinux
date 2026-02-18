//! USB (Universal Serial Bus) Support
//!
//! This module provides USB host controller and device support.

pub mod xhci;

use core::fmt;

/// USB device speed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Low,    // 1.5 Mbps (USB 1.0)
    Full,   // 12 Mbps (USB 1.1)
    High,   // 480 Mbps (USB 2.0)
    Super,  // 5 Gbps (USB 3.0)
    SuperPlus, // 10 Gbps (USB 3.1)
}

impl fmt::Display for UsbSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsbSpeed::Low => write!(f, "Low Speed (1.5 Mbps)"),
            UsbSpeed::Full => write!(f, "Full Speed (12 Mbps)"),
            UsbSpeed::High => write!(f, "High Speed (480 Mbps)"),
            UsbSpeed::Super => write!(f, "Super Speed (5 Gbps)"),
            UsbSpeed::SuperPlus => write!(f, "Super Speed+ (10 Gbps)"),
        }
    }
}

/// USB device class codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UsbClass {
    Interface = 0x00,
    Audio = 0x01,
    Comm = 0x02,
    Hid = 0x03,
    Physical = 0x05,
    Image = 0x06,
    Printer = 0x07,
    MassStorage = 0x08,
    Hub = 0x09,
    CdcData = 0x0A,
    SmartCard = 0x0B,
    ContentSecurity = 0x0D,
    Video = 0x0E,
    PersonalHealthcare = 0x0F,
    AudioVideo = 0x10,
    Diagnostic = 0xDC,
    Wireless = 0xE0,
    Miscellaneous = 0xEF,
    ApplicationSpecific = 0xFE,
    VendorSpecific = 0xFF,
}

/// USB transfer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferType {
    Control,
    Isochronous,
    Bulk,
    Interrupt,
}

/// USB endpoint direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDirection {
    Out = 0,
    In = 1,
}

/// USB device descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub usb_version: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub manufacturer_index: u8,
    pub product_index: u8,
    pub serial_number_index: u8,
    pub num_configurations: u8,
}

/// USB configuration descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbConfigDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub config_value: u8,
    pub config_index: u8,
    pub attributes: u8,
    pub max_power: u8,
}

/// USB interface descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbInterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_index: u8,
}

/// USB endpoint descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbEndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

/// USB device information
#[derive(Debug, Clone, Copy)]
pub struct UsbDevice {
    pub address: u8,
    pub speed: UsbSpeed,
    pub vendor_id: u16,
    pub product_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
}

impl UsbDevice {
    pub const fn new() -> Self {
        Self {
            address: 0,
            speed: UsbSpeed::Full,
            vendor_id: 0,
            product_id: 0,
            class: 0,
            subclass: 0,
            protocol: 0,
        }
    }
}

/// USB host controller trait
pub trait UsbHostController {
    /// Initialize the controller
    fn init(&mut self) -> Result<(), &'static str>;
    
    /// Reset the controller
    fn reset(&mut self) -> Result<(), &'static str>;
    
    /// Get the number of ports
    fn port_count(&self) -> u8;
    
    /// Check if a port has a device connected
    fn port_connected(&self, port: u8) -> bool;
    
    /// Reset a port
    fn reset_port(&mut self, port: u8) -> Result<(), &'static str>;
    
    /// Enumerate devices on all ports
    fn enumerate_devices(&mut self) -> usize;
}

/// Initialize USB subsystem
pub fn init() {
    rinux_kernel::printk::printk("Initializing USB subsystem...\n");
    
    // Find all USB controllers via PCI
    let scanner = crate::pci::scanner();
    
    for device in scanner.find_usb_controllers() {
        if let Some(ctrl_type) = device.usb_controller_type() {
            rinux_kernel::printk::printk("  Found ");
            match ctrl_type {
                crate::pci::UsbControllerType::XHCI => {
                    rinux_kernel::printk::printk("xHCI controller\n");
                    // Initialize xHCI
                    if let Err(e) = xhci::init_controller(device) {
                        rinux_kernel::printk::printk("    Failed to initialize: ");
                        rinux_kernel::printk::printk(e);
                        rinux_kernel::printk::printk("\n");
                    }
                }
                crate::pci::UsbControllerType::EHCI => {
                    rinux_kernel::printk::printk("EHCI controller (not supported yet)\n");
                }
                crate::pci::UsbControllerType::UHCI => {
                    rinux_kernel::printk::printk("UHCI controller (not supported yet)\n");
                }
                crate::pci::UsbControllerType::OHCI => {
                    rinux_kernel::printk::printk("OHCI controller (not supported yet)\n");
                }
            }
        }
    }
}

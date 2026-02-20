//! PCI (Peripheral Component Interconnect) Bus Driver
//!
//! This module provides support for PCI device enumeration and configuration.

use core::fmt;
use rinux_arch_x86::io::{inl, outl};

/// PCI configuration space address port
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
/// PCI configuration space data port
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// PCI device class codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PciClass {
    Unclassified = 0x00,
    MassStorageController = 0x01,
    NetworkController = 0x02,
    DisplayController = 0x03,
    MultimediaController = 0x04,
    MemoryController = 0x05,
    BridgeDevice = 0x06,
    SimpleCommunicationController = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDevice = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBusController = 0x0C, // USB controllers are here
    WirelessController = 0x0D,
    IntelligentController = 0x0E,
    SatelliteController = 0x0F,
    EncryptionController = 0x10,
    SignalProcessingController = 0x11,
    Other = 0xFF,
}

impl From<u8> for PciClass {
    fn from(value: u8) -> Self {
        match value {
            0x00 => PciClass::Unclassified,
            0x01 => PciClass::MassStorageController,
            0x02 => PciClass::NetworkController,
            0x03 => PciClass::DisplayController,
            0x04 => PciClass::MultimediaController,
            0x05 => PciClass::MemoryController,
            0x06 => PciClass::BridgeDevice,
            0x07 => PciClass::SimpleCommunicationController,
            0x08 => PciClass::BaseSystemPeripheral,
            0x09 => PciClass::InputDevice,
            0x0A => PciClass::DockingStation,
            0x0B => PciClass::Processor,
            0x0C => PciClass::SerialBusController,
            0x0D => PciClass::WirelessController,
            0x0E => PciClass::IntelligentController,
            0x0F => PciClass::SatelliteController,
            0x10 => PciClass::EncryptionController,
            0x11 => PciClass::SignalProcessingController,
            _ => PciClass::Other,
        }
    }
}

/// PCI device address
#[derive(Debug, Clone, Copy)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciAddress {
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }

    /// Build the address for PCI configuration space access
    fn config_address(&self, offset: u8) -> u32 {
        let bus = self.bus as u32;
        let device = self.device as u32;
        let function = self.function as u32;
        let offset = (offset & 0xFC) as u32;

        0x8000_0000 | (bus << 16) | (device << 11) | (function << 8) | offset
    }
}

impl fmt::Display for PciAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}:{:02x}.{}", self.bus, self.device, self.function)
    }
}

/// PCI device information
#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: PciClass,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub bars: [u32; 6],
}

impl PciDevice {
    /// Read a 32-bit value from the device's configuration space
    pub fn read_config(&self, offset: u8) -> u32 {
        pci_config_read(self.address, offset)
    }

    /// Write a 32-bit value to the device's configuration space
    pub fn write_config(&self, offset: u8, value: u32) {
        pci_config_write(self.address, offset, value);
    }

    /// Read a 16-bit value from the device's configuration space
    pub fn read_config_u16(&self, offset: u8) -> u16 {
        let dword = self.read_config(offset & 0xFC);
        ((dword >> ((offset & 2) * 8)) & 0xFFFF) as u16
    }

    /// Read an 8-bit value from the device's configuration space
    pub fn read_config_u8(&self, offset: u8) -> u8 {
        let dword = self.read_config(offset & 0xFC);
        ((dword >> ((offset & 3) * 8)) & 0xFF) as u8
    }

    /// Enable bus mastering for this device
    pub fn enable_bus_mastering(&self) {
        let mut command = self.read_config_u16(0x04);
        command |= 0x04; // Bus Master bit
        self.write_config(0x04, command as u32);
    }

    /// Enable memory space access for this device
    pub fn enable_memory_space(&self) {
        let mut command = self.read_config_u16(0x04);
        command |= 0x02; // Memory Space bit
        self.write_config(0x04, command as u32);
    }

    /// Enable I/O space access for this device
    pub fn enable_io_space(&self) {
        let mut command = self.read_config_u16(0x04);
        command |= 0x01; // I/O Space bit
        self.write_config(0x04, command as u32);
    }

    /// Get the interrupt line assigned to this device
    pub fn interrupt_line(&self) -> u8 {
        self.read_config_u8(0x3C)
    }

    /// Get the interrupt pin used by this device
    pub fn interrupt_pin(&self) -> u8 {
        self.read_config_u8(0x3D)
    }

    /// Check if this is a USB controller
    pub fn is_usb_controller(&self) -> bool {
        self.class == PciClass::SerialBusController && self.subclass == 0x03
    }

    /// Get USB controller type
    pub fn usb_controller_type(&self) -> Option<UsbControllerType> {
        if !self.is_usb_controller() {
            return None;
        }

        match self.prog_if {
            0x00 => Some(UsbControllerType::UHCI),
            0x10 => Some(UsbControllerType::OHCI),
            0x20 => Some(UsbControllerType::EHCI),
            0x30 => Some(UsbControllerType::XHCI),
            _ => None,
        }
    }
}

impl fmt::Display for PciDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{:04x}:{:04x}] {:?} (subclass: {:02x}, prog_if: {:02x})",
            self.address, self.vendor_id, self.device_id, self.class, self.subclass, self.prog_if
        )
    }
}

/// USB controller types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbControllerType {
    UHCI, // USB 1.1
    OHCI, // USB 1.1
    EHCI, // USB 2.0
    XHCI, // USB 3.0+
}

impl fmt::Display for UsbControllerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsbControllerType::UHCI => write!(f, "UHCI (USB 1.1)"),
            UsbControllerType::OHCI => write!(f, "OHCI (USB 1.1)"),
            UsbControllerType::EHCI => write!(f, "EHCI (USB 2.0)"),
            UsbControllerType::XHCI => write!(f, "XHCI (USB 3.0+)"),
        }
    }
}

/// Read a 32-bit value from PCI configuration space
pub fn pci_config_read(address: PciAddress, offset: u8) -> u32 {
    unsafe {
        outl(PCI_CONFIG_ADDRESS, address.config_address(offset));
        inl(PCI_CONFIG_DATA)
    }
}

/// Write a 32-bit value to PCI configuration space
pub fn pci_config_write(address: PciAddress, offset: u8, value: u32) {
    unsafe {
        outl(PCI_CONFIG_ADDRESS, address.config_address(offset));
        outl(PCI_CONFIG_DATA, value);
    }
}

/// Check if a device exists at the given address
pub fn device_exists(address: PciAddress) -> bool {
    let vendor_id = pci_config_read(address, 0) & 0xFFFF;
    vendor_id != 0xFFFF
}

/// Read device information
pub fn read_device_info(address: PciAddress) -> Option<PciDevice> {
    if !device_exists(address) {
        return None;
    }

    let dword0 = pci_config_read(address, 0x00);
    let vendor_id = (dword0 & 0xFFFF) as u16;
    let device_id = ((dword0 >> 16) & 0xFFFF) as u16;

    let dword2 = pci_config_read(address, 0x08);
    let revision = (dword2 & 0xFF) as u8;
    let prog_if = ((dword2 >> 8) & 0xFF) as u8;
    let subclass = ((dword2 >> 16) & 0xFF) as u8;
    let class = PciClass::from(((dword2 >> 24) & 0xFF) as u8);

    let dword3 = pci_config_read(address, 0x0C);
    let header_type = ((dword3 >> 16) & 0xFF) as u8;

    // Read BARs
    let mut bars = [0u32; 6];
    for (i, bar) in bars.iter_mut().enumerate() {
        *bar = pci_config_read(address, 0x10 + (i as u8 * 4));
    }

    Some(PciDevice {
        address,
        vendor_id,
        device_id,
        class,
        subclass,
        prog_if,
        revision,
        header_type,
        bars,
    })
}

/// PCI bus scanner
pub struct PciScanner {
    devices: [Option<PciDevice>; 256],
    count: usize,
}

impl Default for PciScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PciScanner {
    pub const fn new() -> Self {
        Self {
            devices: [None; 256],
            count: 0,
        }
    }

    /// Scan all PCI buses for devices
    pub fn scan(&mut self) {
        self.count = 0;

        // Scan all possible bus/device/function combinations
        for bus in 0..=255u8 {
            for device in 0..32u8 {
                for function in 0..8u8 {
                    let address = PciAddress::new(bus, device, function);

                    if let Some(dev_info) = read_device_info(address) {
                        if self.count < 256 {
                            self.devices[self.count] = Some(dev_info);
                            self.count += 1;
                        }

                        // If function 0 doesn't exist or isn't multi-function, skip other functions
                        if function == 0 && (dev_info.header_type & 0x80) == 0 {
                            break;
                        }
                    } else if function == 0 {
                        // If function 0 doesn't exist, no need to check other functions
                        break;
                    }
                }
            }
        }
    }

    /// Get the number of detected devices
    pub fn device_count(&self) -> usize {
        self.count
    }

    /// Get device by index
    pub fn get_device(&self, index: usize) -> Option<&PciDevice> {
        if index < self.count {
            self.devices[index].as_ref()
        } else {
            None
        }
    }

    /// Find all USB controllers
    pub fn find_usb_controllers(&self) -> impl Iterator<Item = &PciDevice> {
        self.devices[..self.count]
            .iter()
            .filter_map(|d| d.as_ref())
            .filter(|d| d.is_usb_controller())
    }

    /// Find all devices of a specific class
    pub fn find_by_class(&self, class: PciClass) -> impl Iterator<Item = &PciDevice> {
        self.devices[..self.count]
            .iter()
            .filter_map(|d| d.as_ref())
            .filter(move |d| d.class == class)
    }
}

/// Global PCI scanner instance
static mut PCI_SCANNER: PciScanner = PciScanner::new();

/// Initialize PCI subsystem
pub fn init() {
    rinux_kernel::printk::printk("Initializing PCI subsystem...\n");

    #[allow(static_mut_refs)]
    unsafe {
        PCI_SCANNER.scan();

        rinux_kernel::printk::printk("PCI: Found ");
        // TODO: Print number
        rinux_kernel::printk::printk(" devices\n");

        // Print USB controllers
        for device in PCI_SCANNER.find_usb_controllers() {
            rinux_kernel::printk::printk("  USB Controller: ");
            if let Some(ctrl_type) = device.usb_controller_type() {
                match ctrl_type {
                    UsbControllerType::UHCI => rinux_kernel::printk::printk("UHCI (USB 1.1)"),
                    UsbControllerType::OHCI => rinux_kernel::printk::printk("OHCI (USB 1.1)"),
                    UsbControllerType::EHCI => rinux_kernel::printk::printk("EHCI (USB 2.0)"),
                    UsbControllerType::XHCI => rinux_kernel::printk::printk("XHCI (USB 3.0+)"),
                }
            }
            rinux_kernel::printk::printk("\n");
        }
    }
}

/// Get a reference to the PCI scanner
#[allow(static_mut_refs)]
pub fn scanner() -> &'static PciScanner {
    unsafe { &PCI_SCANNER }
}

/// Get a mutable reference to the PCI scanner
///
/// # Safety
///
/// The caller must ensure that there are no other active references to the PCI scanner.
#[allow(static_mut_refs)]
pub unsafe fn scanner_mut() -> &'static mut PciScanner {
    &mut PCI_SCANNER
}

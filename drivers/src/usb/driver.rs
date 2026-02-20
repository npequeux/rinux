//! USB Driver Framework
//!
//! This module provides the framework for USB device drivers to register and bind to devices.

use super::{hid, mass_storage, UsbClass, UsbDeviceDescriptor};

/// USB driver match result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverMatch {
    Match,
    NoMatch,
}

/// USB driver interface
pub trait UsbDriver {
    /// Get driver name
    fn name(&self) -> &'static str;

    /// Check if this driver can handle the device
    fn probe(&self, descriptor: &UsbDeviceDescriptor) -> DriverMatch;

    /// Bind driver to device
    fn bind(&mut self, device_address: u8, descriptor: &UsbDeviceDescriptor) -> Result<(), &'static str>;

    /// Unbind driver from device
    fn unbind(&mut self, device_address: u8);
}

/// Try to bind a device to an appropriate driver
pub fn bind_device(device_address: u8, descriptor: &UsbDeviceDescriptor) -> Result<(), &'static str> {
    // Check for HID devices
    if hid::is_hid_device(
        descriptor.device_class,
        descriptor.device_subclass,
        descriptor.device_protocol,
    ) {
        return hid::register_hid_device(device_address, descriptor.device_protocol);
    }

    // Check for mass storage devices
    if mass_storage::is_mass_storage_device(descriptor.device_class) {
        return mass_storage::register_mass_storage_device(
            device_address,
            descriptor.device_subclass,
            descriptor.device_protocol,
        );
    }

    // Check other device classes
    match descriptor.device_class {
        x if x == UsbClass::Hub as u8 => {
            rinux_kernel::printk::printk("  USB: Hub detected (not supported yet)\n");
            Err("Hub support not implemented")
        }
        x if x == UsbClass::Audio as u8 => {
            rinux_kernel::printk::printk("  USB: Audio device detected (not supported yet)\n");
            Err("Audio support not implemented")
        }
        x if x == UsbClass::Video as u8 => {
            rinux_kernel::printk::printk("  USB: Video device detected (not supported yet)\n");
            Err("Video support not implemented")
        }
        x if x == UsbClass::Printer as u8 => {
            rinux_kernel::printk::printk("  USB: Printer detected (not supported yet)\n");
            Err("Printer support not implemented")
        }
        x if x == UsbClass::Wireless as u8 => {
            rinux_kernel::printk::printk("  USB: Wireless device detected (not supported yet)\n");
            Err("Wireless support not implemented")
        }
        _ => {
            rinux_kernel::printk::printk("  USB: Unknown device class\n");
            Err("Unknown device class")
        }
    }
}

/// USB device driver manager
pub struct UsbDriverManager {
    drivers_count: usize,
}

impl UsbDriverManager {
    pub const fn new() -> Self {
        Self { drivers_count: 0 }
    }

    /// Get driver count
    pub fn driver_count(&self) -> usize {
        self.drivers_count
    }
}

impl Default for UsbDriverManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global driver manager
static mut DRIVER_MANAGER: UsbDriverManager = UsbDriverManager::new();

/// Get driver manager
#[allow(static_mut_refs)]
pub fn driver_manager() -> &'static UsbDriverManager {
    unsafe { &DRIVER_MANAGER }
}

/// Get mutable driver manager
///
/// # Safety
///
/// The caller must ensure that there are no other active references to the driver manager.
#[allow(static_mut_refs)]
pub unsafe fn driver_manager_mut() -> &'static mut UsbDriverManager {
    &mut DRIVER_MANAGER
}

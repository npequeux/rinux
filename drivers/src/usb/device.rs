//! USB Device Management
//!
//! This module provides device enumeration and management.

use super::{UsbDevice, UsbDeviceDescriptor, UsbSpeed};

/// Maximum number of USB devices
const MAX_USB_DEVICES: usize = 128;

/// USB device state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDeviceState {
    Uninitialized,
    Attached,
    Powered,
    Default,
    Addressed,
    Configured,
    Suspended,
}

/// Extended USB device information
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceInfo {
    pub device: UsbDevice,
    pub state: UsbDeviceState,
    pub port: u8,
    pub descriptor: Option<UsbDeviceDescriptor>,
}

impl Default for UsbDeviceInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl UsbDeviceInfo {
    pub const fn new() -> Self {
        Self {
            device: UsbDevice::new(),
            state: UsbDeviceState::Uninitialized,
            port: 0,
            descriptor: None,
        }
    }
}

/// USB device manager
pub struct UsbDeviceManager {
    devices: [Option<UsbDeviceInfo>; MAX_USB_DEVICES],
    count: usize,
}

impl Default for UsbDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UsbDeviceManager {
    pub const fn new() -> Self {
        Self {
            devices: [None; MAX_USB_DEVICES],
            count: 0,
        }
    }

    /// Register a new device
    pub fn register_device(&mut self, port: u8, speed: UsbSpeed) -> Option<u8> {
        if self.count >= MAX_USB_DEVICES {
            return None;
        }

        let address = (self.count + 1) as u8;
        let mut device_info = UsbDeviceInfo::new();
        device_info.device.address = address;
        device_info.device.speed = speed;
        device_info.port = port;
        device_info.state = UsbDeviceState::Attached;

        self.devices[self.count] = Some(device_info);
        self.count += 1;

        Some(address)
    }

    /// Update device descriptor
    pub fn set_descriptor(&mut self, address: u8, descriptor: UsbDeviceDescriptor) -> bool {
        for device_info in self.devices.iter_mut().take(self.count).flatten() {
            if device_info.device.address == address {
                device_info.descriptor = Some(descriptor);
                device_info.device.vendor_id = descriptor.vendor_id;
                device_info.device.product_id = descriptor.product_id;
                device_info.device.class = descriptor.device_class;
                device_info.device.subclass = descriptor.device_subclass;
                device_info.device.protocol = descriptor.device_protocol;
                return true;
            }
        }
        false
    }

    /// Update device state
    pub fn set_state(&mut self, address: u8, state: UsbDeviceState) -> bool {
        for device_info in self.devices.iter_mut().take(self.count).flatten() {
            if device_info.device.address == address {
                device_info.state = state;
                return true;
            }
        }
        false
    }

    /// Get device by address
    pub fn get_device(&self, address: u8) -> Option<&UsbDeviceInfo> {
        self.devices
            .iter()
            .take(self.count)
            .flatten()
            .find(|device_info| device_info.device.address == address)
    }

    /// Get device count
    pub fn device_count(&self) -> usize {
        self.count
    }

    /// Iterate over all devices
    pub fn iter(&self) -> impl Iterator<Item = &UsbDeviceInfo> {
        self.devices[..self.count].iter().filter_map(|d| d.as_ref())
    }
}

/// Global device manager
static mut DEVICE_MANAGER: UsbDeviceManager = UsbDeviceManager::new();

/// Get device manager
///
/// # Safety
///
/// This function is safe to call during single-threaded initialization.
/// In a multi-threaded context, external synchronization is required.
#[allow(static_mut_refs)]
pub fn device_manager() -> &'static UsbDeviceManager {
    unsafe { &DEVICE_MANAGER }
}

/// Get mutable device manager
///
/// # Safety
///
/// The caller must ensure that:
/// - There are no other active references to the device manager
/// - No concurrent access occurs (e.g., during single-threaded boot)
///
/// TODO: Replace with proper synchronization (Mutex) when threading is added
#[allow(static_mut_refs)]
pub unsafe fn device_manager_mut() -> &'static mut UsbDeviceManager {
    &mut DEVICE_MANAGER
}

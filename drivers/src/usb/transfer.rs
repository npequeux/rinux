//! USB Transfer Management
//!
//! This module provides USB transfer mechanisms for communication with devices.

use super::{UsbDirection, UsbTransferType};

/// USB transfer request
#[derive(Debug, Clone, Copy)]
pub struct UsbTransferRequest {
    pub device_address: u8,
    pub endpoint: u8,
    pub direction: UsbDirection,
    pub transfer_type: UsbTransferType,
    pub data_length: usize,
}

impl UsbTransferRequest {
    /// Create a new transfer request
    pub const fn new(
        device_address: u8,
        endpoint: u8,
        direction: UsbDirection,
        transfer_type: UsbTransferType,
        data_length: usize,
    ) -> Self {
        Self {
            device_address,
            endpoint,
            direction,
            transfer_type,
            data_length,
        }
    }
}

/// USB control setup packet
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbSetupPacket {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl UsbSetupPacket {
    /// Create a new setup packet
    pub const fn new(request_type: u8, request: u8, value: u16, index: u16, length: u16) -> Self {
        Self {
            request_type,
            request,
            value,
            index,
            length,
        }
    }

    /// Create a GET_DESCRIPTOR setup packet
    pub const fn get_descriptor(descriptor_type: u8, descriptor_index: u8, length: u16) -> Self {
        Self::new(
            0x80, // Device to Host
            0x06, // GET_DESCRIPTOR
            ((descriptor_type as u16) << 8) | (descriptor_index as u16),
            0,
            length,
        )
    }

    /// Create a SET_ADDRESS setup packet
    pub const fn set_address(address: u8) -> Self {
        Self::new(
            0x00, // Host to Device
            0x05, // SET_ADDRESS
            address as u16,
            0,
            0,
        )
    }

    /// Create a SET_CONFIGURATION setup packet
    pub const fn set_configuration(config_value: u8) -> Self {
        Self::new(
            0x00, // Host to Device
            0x09, // SET_CONFIGURATION
            config_value as u16,
            0,
            0,
        )
    }

    /// Create a GET_STATUS setup packet
    pub const fn get_status(recipient: u8, index: u16) -> Self {
        Self::new(
            0x80 | recipient, // Device to Host
            0x00,             // GET_STATUS
            0,
            index,
            2,
        )
    }
}

/// USB standard requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UsbRequest {
    GetStatus = 0x00,
    ClearFeature = 0x01,
    SetFeature = 0x03,
    SetAddress = 0x05,
    GetDescriptor = 0x06,
    SetDescriptor = 0x07,
    GetConfiguration = 0x08,
    SetConfiguration = 0x09,
    GetInterface = 0x0A,
    SetInterface = 0x0B,
    SynchFrame = 0x0C,
}

/// USB descriptor types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UsbDescriptorType {
    Device = 0x01,
    Configuration = 0x02,
    String = 0x03,
    Interface = 0x04,
    Endpoint = 0x05,
    DeviceQualifier = 0x06,
    OtherSpeedConfiguration = 0x07,
    InterfacePower = 0x08,
    Hid = 0x21,
    HidReport = 0x22,
}

/// USB transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferStatus {
    Success,
    Pending,
    Error,
    Stalled,
    Timeout,
    Babble,
    DataBufferError,
    NotSupported,
}

/// USB transfer result
pub type UsbTransferResult = Result<usize, UsbTransferStatus>;

//! USB Mass Storage Driver
//!
//! This module provides support for USB mass storage devices (flash drives, external hard drives).

use super::UsbClass;

/// Mass storage subclass codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MassStorageSubclass {
    SCSI = 0x06,           // SCSI transparent command set
    Obsolete1 = 0x01,      // RBC (reduced block commands)
    Obsolete2 = 0x02,      // MMC-5 (ATAPI)
    Obsolete3 = 0x03,      // Obsolete
    Obsolete4 = 0x04,      // UFI
    Obsolete5 = 0x05,      // Obsolete
    LockingMedia = 0x07,   // LSD FS
    IEEE1667 = 0x08,       // IEEE 1667
    VendorSpecific = 0xFF, // Vendor specific
}

/// Mass storage protocol codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MassStorageProtocol {
    CBI = 0x00,            // Control/Bulk/Interrupt
    CBINoInt = 0x01,       // Control/Bulk/Interrupt without command completion
    BBB = 0x50,            // Bulk-Only Transport (most common)
    UAS = 0x62,            // USB Attached SCSI
    VendorSpecific = 0xFF, // Vendor specific
}

/// Mass storage device information
#[derive(Debug, Clone, Copy)]
pub struct MassStorageDevice {
    pub device_address: u8,
    pub subclass: MassStorageSubclass,
    pub protocol: MassStorageProtocol,
    pub lun: u8, // Logical Unit Number
    pub bulk_in_endpoint: u8,
    pub bulk_out_endpoint: u8,
    pub max_packet_size: u16,
}

impl MassStorageDevice {
    pub const fn new(device_address: u8) -> Self {
        Self {
            device_address,
            subclass: MassStorageSubclass::SCSI,
            protocol: MassStorageProtocol::BBB,
            lun: 0,
            bulk_in_endpoint: 0,
            bulk_out_endpoint: 0,
            max_packet_size: 512,
        }
    }
}

/// Bulk-Only Transport Command Block Wrapper
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CommandBlockWrapper {
    pub signature: u32,          // 0x43425355 "USBC"
    pub tag: u32,                // Command block tag
    pub data_transfer_length: u32, // Number of bytes to transfer
    pub flags: u8,               // Bit 7: Direction (0=Out, 1=In)
    pub lun: u8,                 // Logical Unit Number
    pub cb_length: u8,           // Command block length (1-16)
    pub cb: [u8; 16],            // Command block
}

impl CommandBlockWrapper {
    const SIGNATURE: u32 = 0x43425355; // "USBC"

    pub const fn new(tag: u32, data_length: u32, direction: bool, lun: u8) -> Self {
        Self {
            signature: Self::SIGNATURE,
            tag,
            data_transfer_length: data_length,
            flags: if direction { 0x80 } else { 0x00 },
            lun,
            cb_length: 0,
            cb: [0; 16],
        }
    }
}

/// Bulk-Only Transport Command Status Wrapper
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CommandStatusWrapper {
    pub signature: u32,   // 0x53425355 "USBS"
    pub tag: u32,         // Command block tag
    pub data_residue: u32, // Difference between expected and actual data transfer
    pub status: u8,       // Command status (0=passed, 1=failed, 2=phase error)
}

impl CommandStatusWrapper {
    const SIGNATURE: u32 = 0x53425355; // "USBS"

    pub fn is_valid(&self) -> bool {
        self.signature == Self::SIGNATURE
    }
}

/// SCSI commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScsiCommand {
    TestUnitReady = 0x00,
    RequestSense = 0x03,
    Inquiry = 0x12,
    ReadCapacity10 = 0x25,
    Read10 = 0x28,
    Write10 = 0x2A,
    ReadCapacity16 = 0x9E,
}

/// Check if a device is a mass storage device
pub fn is_mass_storage_device(class: u8) -> bool {
    class == UsbClass::MassStorage as u8
}

/// Initialize mass storage driver
pub fn init() {
    rinux_kernel::printk::printk("  Mass Storage: Initializing mass storage driver\n");
}

/// Register a mass storage device
pub fn register_mass_storage_device(device_address: u8, subclass: u8, protocol: u8) -> Result<(), &'static str> {
    let _device = MassStorageDevice::new(device_address);

    rinux_kernel::printk::printk("  Mass Storage: Registered device (subclass: ");
    match subclass {
        0x06 => rinux_kernel::printk::printk("SCSI"),
        _ => rinux_kernel::printk::printk("Unknown"),
    }
    rinux_kernel::printk::printk(", protocol: ");
    match protocol {
        0x50 => rinux_kernel::printk::printk("Bulk-Only Transport"),
        0x62 => rinux_kernel::printk::printk("UAS"),
        _ => rinux_kernel::printk::printk("Unknown"),
    }
    rinux_kernel::printk::printk(")\n");

    Ok(())
}

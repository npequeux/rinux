//! AHCI (Advanced Host Controller Interface) Driver
//!
//! Driver for SATA devices using AHCI

use crate::device::{BlockDevice, BlockDeviceError};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

/// AHCI PCI Class/Subclass
pub const AHCI_PCI_CLASS: u8 = 0x01;  // Mass Storage Controller
pub const AHCI_PCI_SUBCLASS: u8 = 0x06;  // SATA Controller

/// AHCI HBA (Host Bus Adapter) Registers
#[repr(C)]
struct HbaRegisters {
    capability: u32,
    global_host_control: u32,
    interrupt_status: u32,
    ports_implemented: u32,
    version: u32,
    ccc_control: u32,
    ccc_ports: u32,
    em_location: u32,
    em_control: u32,
    capability2: u32,
    bohc: u32,
}

/// AHCI Port Registers
#[repr(C)]
struct PortRegisters {
    command_list_base: u32,
    command_list_base_upper: u32,
    fis_base: u32,
    fis_base_upper: u32,
    interrupt_status: u32,
    interrupt_enable: u32,
    command_and_status: u32,
    _reserved: u32,
    task_file_data: u32,
    signature: u32,
    sata_status: u32,
    sata_control: u32,
    sata_error: u32,
    sata_active: u32,
    command_issue: u32,
    sata_notification: u32,
    fis_based_switching: u32,
}

/// AHCI Device
pub struct AhciDevice {
    name: String,
    port: usize,
    block_size: usize,
    num_blocks: u64,
    hba: *mut HbaRegisters,
}

unsafe impl Send for AhciDevice {}
unsafe impl Sync for AhciDevice {}

impl AhciDevice {
    /// Create a new AHCI device
    pub fn new(name: String, port: usize, hba: *mut HbaRegisters) -> Self {
        AhciDevice {
            name,
            port,
            block_size: 512,  // Standard sector size
            num_blocks: 0,    // Will be detected
            hba,
        }
    }

    /// Identify the device and get capacity
    fn identify(&mut self) -> Result<(), BlockDeviceError> {
        // This would send an ATA IDENTIFY command to the device
        // For now, we'll just set a default capacity
        self.num_blocks = 1024 * 1024 * 1024 / 512;  // 1GB default
        Ok(())
    }

    /// Issue a read command to the device
    fn read_dma(&self, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        // This would:
        // 1. Build a command FIS (Frame Information Structure)
        // 2. Set up the command table
        // 3. Issue the command to the port
        // 4. Wait for completion
        // 5. Copy data from DMA buffer to user buffer
        
        // For now, this is a stub
        let _ = (lba, count, buffer);
        Err(BlockDeviceError::NotReady)
    }

    /// Issue a write command to the device
    fn write_dma(&self, lba: u64, count: u16, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        // Similar to read_dma but for writing
        let _ = (lba, count, buffer);
        Err(BlockDeviceError::NotReady)
    }
}

impl BlockDevice for AhciDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn block_size(&self) -> usize {
        self.block_size
    }

    fn num_blocks(&self) -> u64 {
        self.num_blocks
    }

    fn read_blocks(&self, block_offset: u64, buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
        if block_offset >= self.num_blocks {
            return Err(BlockDeviceError::InvalidOffset);
        }

        let blocks_to_read = (buffer.len() / self.block_size).min((self.num_blocks - block_offset) as usize);
        if blocks_to_read == 0 {
            return Ok(0);
        }

        self.read_dma(block_offset, blocks_to_read as u16, buffer)?;
        Ok(blocks_to_read)
    }

    fn write_blocks(&self, block_offset: u64, buffer: &[u8]) -> Result<usize, BlockDeviceError> {
        if block_offset >= self.num_blocks {
            return Err(BlockDeviceError::InvalidOffset);
        }

        let blocks_to_write = (buffer.len() / self.block_size).min((self.num_blocks - block_offset) as usize);
        if blocks_to_write == 0 {
            return Ok(0);
        }

        self.write_dma(block_offset, blocks_to_write as u16, buffer)?;
        Ok(blocks_to_write)
    }

    fn flush(&self) -> Result<(), BlockDeviceError> {
        // Issue FLUSH CACHE command
        Ok(())
    }

    fn model(&self) -> Option<&str> {
        Some("AHCI SATA Device")
    }
}

/// AHCI Controller
pub struct AhciController {
    hba: *mut HbaRegisters,
    devices: Vec<Arc<AhciDevice>>,
}

impl AhciController {
    /// Create a new AHCI controller
    ///
    /// # Safety
    ///
    /// The caller must ensure that `hba_base` points to valid AHCI MMIO registers
    pub unsafe fn new(hba_base: usize) -> Self {
        AhciController {
            hba: hba_base as *mut HbaRegisters,
            devices: Vec::new(),
        }
    }

    /// Initialize the controller
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Reset HBA
        // Enable AHCI mode
        // Detect ports with devices attached
        // Initialize each port
        
        // For now, this is a stub
        Ok(())
    }

    /// Probe for devices on all ports
    pub fn probe_devices(&mut self) {
        // Check each port (typically 0-31)
        // For each port that has a device:
        //   1. Initialize the port
        //   2. Identify the device
        //   3. Create an AhciDevice and register it
        
        // Stub: assume port 0 has a device
        let device = AhciDevice::new(
            String::from("sda"),
            0,
            self.hba,
        );
        self.devices.push(Arc::new(device));
    }
}

static AHCI_CONTROLLERS: Mutex<Vec<AhciController>> = Mutex::new(Vec::new());

/// Initialize AHCI driver
pub fn init() {
    // Scan PCI for AHCI controllers
    // For each controller found:
    //   1. Map the HBA registers
    //   2. Create an AhciController
    //   3. Initialize it
    //   4. Probe for devices
    //   5. Register devices with block layer
    
    // This is a stub - full implementation would scan PCI bus
}

/// Scan PCI bus for AHCI controllers
fn scan_pci_for_ahci() -> Vec<usize> {
    // Scan PCI configuration space for devices with:
    // - Class = 0x01 (Mass Storage)
    // - Subclass = 0x06 (SATA)
    // - Programming Interface = 0x01 (AHCI)
    
    // Return list of MMIO base addresses
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ahci_device_creation() {
        let hba = core::ptr::null_mut();
        let device = AhciDevice::new(String::from("sda"), 0, hba);
        assert_eq!(device.name(), "sda");
        assert_eq!(device.block_size(), 512);
    }
}

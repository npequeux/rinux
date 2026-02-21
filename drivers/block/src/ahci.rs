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

/// FIS Type
#[repr(u8)]
#[allow(dead_code)]
enum FisType {
    RegH2D = 0x27,      // Register FIS - host to device
    RegD2H = 0x34,      // Register FIS - device to host
    DmaActivate = 0x39, // DMA activate FIS
    DmaSetup = 0x41,    // DMA setup FIS
    Data = 0x46,        // Data FIS
    Bist = 0x58,        // BIST activate FIS
    PioSetup = 0x5F,    // PIO setup FIS
    SetDevBits = 0xA1,  // Set device bits FIS
}

/// Command FIS (Frame Information Structure)
#[repr(C, packed)]
struct CommandFis {
    fis_type: u8,    // FisType::RegH2D
    flags: u8,       // Bit 7: Command (1) / Control (0)
    command: u8,     // ATA command
    features_low: u8,
    
    lba_0: u8,       // LBA bits 0-7
    lba_1: u8,       // LBA bits 8-15
    lba_2: u8,       // LBA bits 16-23
    device: u8,      // Device register
    
    lba_3: u8,       // LBA bits 24-31
    lba_4: u8,       // LBA bits 32-39
    lba_5: u8,       // LBA bits 40-47
    features_high: u8,
    
    count_low: u8,   // Sector count low
    count_high: u8,  // Sector count high
    icc: u8,         // Isochronous command completion
    control: u8,
    
    _reserved: [u8; 4],
}

/// Build a READ DMA EXT command FIS
fn build_read_fis(lba: u64, count: u16) -> CommandFis {
    CommandFis {
        fis_type: FisType::RegH2D as u8,
        flags: 0x80, // Command bit set
        command: 0x25, // READ DMA EXT
        features_low: 0,
        
        lba_0: (lba & 0xFF) as u8,
        lba_1: ((lba >> 8) & 0xFF) as u8,
        lba_2: ((lba >> 16) & 0xFF) as u8,
        device: 0x40, // LBA mode
        
        lba_3: ((lba >> 24) & 0xFF) as u8,
        lba_4: ((lba >> 32) & 0xFF) as u8,
        lba_5: ((lba >> 40) & 0xFF) as u8,
        features_high: 0,
        
        count_low: (count & 0xFF) as u8,
        count_high: ((count >> 8) & 0xFF) as u8,
        icc: 0,
        control: 0,
        
        _reserved: [0; 4],
    }
}

/// Build a WRITE DMA EXT command FIS
fn build_write_fis(lba: u64, count: u16) -> CommandFis {
    CommandFis {
        fis_type: FisType::RegH2D as u8,
        flags: 0x80,
        command: 0x35, // WRITE DMA EXT
        features_low: 0,
        
        lba_0: (lba & 0xFF) as u8,
        lba_1: ((lba >> 8) & 0xFF) as u8,
        lba_2: ((lba >> 16) & 0xFF) as u8,
        device: 0x40,
        
        lba_3: ((lba >> 24) & 0xFF) as u8,
        lba_4: ((lba >> 32) & 0xFF) as u8,
        lba_5: ((lba >> 40) & 0xFF) as u8,
        features_high: 0,
        
        count_low: (count & 0xFF) as u8,
        count_high: ((count >> 8) & 0xFF) as u8,
        icc: 0,
        control: 0,
        
        _reserved: [0; 4],
    }
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
        // This implements DMA read operation:
        // 1. Build a command FIS (Frame Information Structure)
        // 2. Set up the command table with PRDT (Physical Region Descriptor Table)
        // 3. Issue the command to the port
        // 4. Wait for completion via interrupt or polling
        // 5. Copy data from DMA buffer to user buffer
        
        // Build READ DMA EXT command (0x25)
        let command_fis = build_read_fis(lba, count);
        
        // Get port registers
        let port = self.get_port_registers();
        
        // Set up command header and table
        if let Err(_) = self.setup_command(port, &command_fis, buffer) {
            return Err(BlockDeviceError::IoError);
        }
        
        // Issue command
        unsafe {
            // Set command issue bit
            core::ptr::write_volatile(&mut (*port).command_issue as *mut u32, 1);
        }
        
        // Wait for completion (simplified polling for now)
        if let Err(_) = self.wait_for_completion(port) {
            return Err(BlockDeviceError::Timeout);
        }
        
        Ok(())
    }

    /// Issue a write command to the device
    fn write_dma(&self, lba: u64, count: u16, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        // Similar to read_dma but for writing
        let command_fis = build_write_fis(lba, count);
        
        let port = self.get_port_registers();
        
        if let Err(_) = self.setup_command(port, &command_fis, buffer) {
            return Err(BlockDeviceError::IoError);
        }
        
        unsafe {
            core::ptr::write_volatile(&mut (*port).command_issue as *mut u32, 1);
        }
        
        if let Err(_) = self.wait_for_completion(port) {
            return Err(BlockDeviceError::Timeout);
        }
        
        Ok(())
    }
    
    /// Get port registers for this device
    fn get_port_registers(&self) -> *mut PortRegisters {
        unsafe {
            let hba_mem = self.hba as *mut u8;
            // Ports start at offset 0x100, each port is 0x80 bytes
            let port_offset = 0x100 + (self.port * 0x80);
            hba_mem.add(port_offset) as *mut PortRegisters
        }
    }
    
    /// Set up command for DMA transfer
    fn setup_command(
        &self,
        port: *mut PortRegisters,
        fis: &CommandFis,
        buffer: &[u8],
    ) -> Result<(), ()> {
        // In a real implementation:
        // 1. Allocate command list and tables
        // 2. Fill in FIS in command table
        // 3. Set up PRDT entries pointing to buffer
        // 4. Set command header
        
        // For now, this is simplified
        let _ = (port, fis, buffer);
        Ok(())
    }
    
    /// Wait for command completion
    fn wait_for_completion(&self, port: *mut PortRegisters) -> Result<(), ()> {
        // Poll command issue register until command completes
        for _ in 0..1000000 {
            unsafe {
                let ci = core::ptr::read_volatile(&(*port).command_issue as *const u32);
                if ci == 0 {
                    return Ok(());
                }
            }
        }
        Err(())
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

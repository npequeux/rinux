//! NVMe (Non-Volatile Memory Express) Driver
//!
//! Driver for NVMe SSDs

use crate::device::{BlockDevice, BlockDeviceError};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

/// NVMe PCI Class/Subclass
pub const NVME_PCI_CLASS: u8 = 0x01;  // Mass Storage Controller
pub const NVME_PCI_SUBCLASS: u8 = 0x08;  // Non-Volatile Memory Controller
pub const NVME_PCI_INTERFACE: u8 = 0x02;  // NVMe

/// NVMe Controller Registers
#[repr(C)]
struct NvmeRegisters {
    capability: u64,           // Controller Capabilities
    version: u32,              // Version
    interrupt_mask_set: u32,   // Interrupt Mask Set
    interrupt_mask_clear: u32, // Interrupt Mask Clear
    controller_config: u32,    // Controller Configuration
    _reserved: u32,
    controller_status: u32,    // Controller Status
    nvm_subsystem_reset: u32,  // NVM Subsystem Reset
    admin_queue_attr: u32,     // Admin Queue Attributes
    admin_sq_base: u64,        // Admin Submission Queue Base Address
    admin_cq_base: u64,        // Admin Completion Queue Base Address
}

/// NVMe Submission Queue Entry
#[repr(C)]
struct NvmeSubmissionQueueEntry {
    opcode: u8,
    flags: u8,
    command_id: u16,
    namespace_id: u32,
    _reserved: [u32; 2],
    metadata_ptr: u64,
    data_ptr: [u64; 2],
    dword: [u32; 6],
}

/// NVMe Completion Queue Entry
#[repr(C)]
struct NvmeCompletionQueueEntry {
    result: u32,
    _reserved: u32,
    submission_queue_head: u16,
    submission_queue_id: u16,
    command_id: u16,
    status: u16,
}

/// NVMe Device (Namespace)
pub struct NvmeDevice {
    name: String,
    namespace_id: u32,
    block_size: usize,
    num_blocks: u64,
    controller: *mut NvmeRegisters,
}

unsafe impl Send for NvmeDevice {}
unsafe impl Sync for NvmeDevice {}

impl NvmeDevice {
    /// Create a new NVMe device
    pub fn new(
        name: String,
        namespace_id: u32,
        controller: *mut NvmeRegisters,
    ) -> Self {
        NvmeDevice {
            name,
            namespace_id,
            block_size: 512,  // Default, will be queried
            num_blocks: 0,    // Will be queried
            controller,
        }
    }

    /// Identify the namespace and get capacity
    fn identify(&mut self) -> Result<(), BlockDeviceError> {
        // Send IDENTIFY NAMESPACE admin command
        // Parse the returned data to get:
        // - Block size (LBAF - LBA Format)
        // - Number of blocks (NSZE - Namespace Size)
        
        // For now, set defaults
        self.block_size = 4096;  // NVMe often uses 4K
        self.num_blocks = 128 * 1024 * 1024;  // 512GB default (with 4K blocks)
        Ok(())
    }

    /// Submit an I/O read command
    fn read_io(&self, lba: u64, count: u16, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        // This would:
        // 1. Build a read command (opcode 0x02)
        // 2. Set up PRP (Physical Region Pages) entries for the buffer
        // 3. Submit to I/O submission queue
        // 4. Ring doorbell
        // 5. Wait for completion queue entry
        // 6. Check status
        
        let _ = (lba, count, buffer);
        Err(BlockDeviceError::NotReady)
    }

    /// Submit an I/O write command
    fn write_io(&self, lba: u64, count: u16, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        // Similar to read_io but with write command (opcode 0x01)
        let _ = (lba, count, buffer);
        Err(BlockDeviceError::NotReady)
    }
}

impl BlockDevice for NvmeDevice {
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

        self.read_io(block_offset, blocks_to_read as u16, buffer)?;
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

        self.write_io(block_offset, blocks_to_write as u16, buffer)?;
        Ok(blocks_to_write)
    }

    fn flush(&self) -> Result<(), BlockDeviceError> {
        // Send FLUSH command (opcode 0x00)
        Ok(())
    }

    fn model(&self) -> Option<&str> {
        Some("NVMe SSD")
    }
}

/// NVMe Controller
pub struct NvmeController {
    controller: *mut NvmeRegisters,
    devices: Vec<Arc<NvmeDevice>>,
    num_namespaces: u32,
}

impl NvmeController {
    /// Create a new NVMe controller
    ///
    /// # Safety
    ///
    /// The caller must ensure that `controller_base` points to valid NVMe MMIO registers
    pub unsafe fn new(controller_base: usize) -> Self {
        NvmeController {
            controller: controller_base as *mut NvmeRegisters,
            devices: Vec::new(),
            num_namespaces: 0,
        }
    }

    /// Initialize the controller
    pub fn init(&mut self) -> Result<(), &'static str> {
        // 1. Wait for controller ready (CSTS.RDY = 0)
        // 2. Configure admin queues
        // 3. Enable controller (CC.EN = 1)
        // 4. Wait for controller ready (CSTS.RDY = 1)
        // 5. Identify controller to get number of namespaces
        
        Ok(())
    }

    /// Probe for namespaces
    pub fn probe_namespaces(&mut self) {
        // For each namespace ID (1 to nn from IDENTIFY CONTROLLER):
        //   1. Send IDENTIFY NAMESPACE
        //   2. Check if namespace is active
        //   3. Create NvmeDevice
        //   4. Register with block layer
        
        // Stub: assume namespace 1 exists
        let device = NvmeDevice::new(
            String::from("nvme0n1"),
            1,
            self.controller,
        );
        self.devices.push(Arc::new(device));
    }
}

static NVME_CONTROLLERS: Mutex<Vec<NvmeController>> = Mutex::new(Vec::new());

/// Initialize NVMe driver
pub fn init() {
    // Scan PCI for NVMe controllers
    // For each controller found:
    //   1. Map the controller registers
    //   2. Create an NvmeController
    //   3. Initialize it
    //   4. Probe for namespaces
    //   5. Register devices with block layer
    
    // This is a stub - full implementation would scan PCI bus
}

/// Scan PCI bus for NVMe controllers
fn scan_pci_for_nvme() -> Vec<usize> {
    // Scan PCI configuration space for devices with:
    // - Class = 0x01 (Mass Storage)
    // - Subclass = 0x08 (Non-Volatile Memory Controller)
    // - Programming Interface = 0x02 (NVMe)
    
    // Return list of MMIO base addresses
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvme_device_creation() {
        let controller = core::ptr::null_mut();
        let device = NvmeDevice::new(String::from("nvme0n1"), 1, controller);
        assert_eq!(device.name(), "nvme0n1");
    }
}

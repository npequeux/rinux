//! Block Device Layer
//!
//! Provides abstraction for block devices (disks, SSDs, etc.)

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Block size (512 bytes - standard sector size)
pub const BLOCK_SIZE: usize = 512;

/// Block device operations
pub trait BlockDevice: Send + Sync {
    /// Read blocks from the device
    ///
    /// # Arguments
    ///
    /// * `start_block` - Starting block number
    /// * `buffer` - Buffer to read into
    ///
    /// # Returns
    ///
    /// Number of blocks read
    fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<usize, &'static str>;

    /// Write blocks to the device
    ///
    /// # Arguments
    ///
    /// * `start_block` - Starting block number
    /// * `buffer` - Buffer to write from
    ///
    /// # Returns
    ///
    /// Number of blocks written
    fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<usize, &'static str>;

    /// Flush any cached writes to device
    fn flush(&self) -> Result<(), &'static str>;

    /// Get device block count
    fn block_count(&self) -> u64;

    /// Get device block size
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    /// Get device name
    fn name(&self) -> &str;

    /// Check if device is read-only
    fn is_read_only(&self) -> bool {
        false
    }
}

/// Block device metadata
#[derive(Debug, Clone)]
pub struct BlockDeviceInfo {
    pub major: u32,
    pub minor: u32,
    pub name: String,
    pub block_count: u64,
    pub block_size: usize,
    pub read_only: bool,
}

/// Block I/O request
pub struct BlockRequest {
    pub device_id: u32,
    pub operation: BlockOp,
    pub start_block: u64,
    pub block_count: usize,
    pub buffer: Vec<u8>,
}

/// Block operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockOp {
    Read,
    Write,
    Flush,
}

/// Block I/O statistics
#[derive(Debug, Clone, Copy)]
pub struct BlockStats {
    pub read_count: u64,
    pub write_count: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub errors: u64,
}

/// Block device wrapper with statistics
pub struct ManagedBlockDevice {
    device: Box<dyn BlockDevice>,
    stats: Mutex<BlockStats>,
}

impl ManagedBlockDevice {
    /// Create a new managed block device
    pub fn new(device: Box<dyn BlockDevice>) -> Self {
        ManagedBlockDevice {
            device,
            stats: Mutex::new(BlockStats {
                read_count: 0,
                write_count: 0,
                read_bytes: 0,
                write_bytes: 0,
                errors: 0,
            }),
        }
    }

    /// Read blocks with statistics tracking
    pub fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<usize, &'static str> {
        let result = self.device.read_blocks(start_block, buffer);
        
        let mut stats = self.stats.lock();
        match result {
            Ok(count) => {
                stats.read_count += 1;
                stats.read_bytes += (count * self.device.block_size()) as u64;
            }
            Err(_) => {
                stats.errors += 1;
            }
        }
        
        result
    }

    /// Write blocks with statistics tracking
    pub fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<usize, &'static str> {
        let result = self.device.write_blocks(start_block, buffer);
        
        let mut stats = self.stats.lock();
        match result {
            Ok(count) => {
                stats.write_count += 1;
                stats.write_bytes += (count * self.device.block_size()) as u64;
            }
            Err(_) => {
                stats.errors += 1;
            }
        }
        
        result
    }

    /// Get statistics
    pub fn get_stats(&self) -> BlockStats {
        *self.stats.lock()
    }

    /// Get device info
    pub fn info(&self) -> BlockDeviceInfo {
        BlockDeviceInfo {
            major: 0, // TODO: Assign major numbers dynamically
            minor: 0,
            name: String::from(self.device.name()),
            block_count: self.device.block_count(),
            block_size: self.device.block_size(),
            read_only: self.device.is_read_only(),
        }
    }
}

/// Block device registry
static BLOCK_DEVICES: Mutex<Vec<ManagedBlockDevice>> = Mutex::new(Vec::new());
static NEXT_DEVICE_ID: AtomicU64 = AtomicU64::new(0);

/// Register a block device
pub fn register_device(device: Box<dyn BlockDevice>) -> u32 {
    let device_id = NEXT_DEVICE_ID.fetch_add(1, Ordering::SeqCst) as u32;
    let managed = ManagedBlockDevice::new(device);
    
    let mut devices = BLOCK_DEVICES.lock();
    devices.push(managed);
    
    device_id
}

/// Get a block device by ID
pub fn get_device(device_id: u32) -> Option<&'static ManagedBlockDevice> {
    // TODO: This is unsafe - need proper lifetime management
    // For now, return None
    let _ = device_id;
    None
}

/// List all block devices
pub fn list_devices() -> Vec<BlockDeviceInfo> {
    let devices = BLOCK_DEVICES.lock();
    devices.iter().map(|d| d.info()).collect()
}

/// Initialize block device layer
pub fn init() {
    // Block device layer initialized
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBlockDevice {
        name: String,
        blocks: u64,
    }

    impl BlockDevice for TestBlockDevice {
        fn read_blocks(&self, _start_block: u64, buffer: &mut [u8]) -> Result<usize, &'static str> {
            let blocks = buffer.len() / BLOCK_SIZE;
            Ok(blocks)
        }

        fn write_blocks(&self, _start_block: u64, buffer: &[u8]) -> Result<usize, &'static str> {
            let blocks = buffer.len() / BLOCK_SIZE;
            Ok(blocks)
        }

        fn flush(&self) -> Result<(), &'static str> {
            Ok(())
        }

        fn block_count(&self) -> u64 {
            self.blocks
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_block_device_creation() {
        let device = TestBlockDevice {
            name: String::from("test0"),
            blocks: 1000,
        };
        
        assert_eq!(device.block_count(), 1000);
        assert_eq!(device.block_size(), BLOCK_SIZE);
        assert_eq!(device.name(), "test0");
    }

    #[test]
    fn test_managed_device_stats() {
        let device = Box::new(TestBlockDevice {
            name: String::from("test0"),
            blocks: 1000,
        });
        
        let managed = ManagedBlockDevice::new(device);
        let mut buffer = vec![0u8; BLOCK_SIZE * 2];
        
        managed.read_blocks(0, &mut buffer).unwrap();
        
        let stats = managed.get_stats();
        assert_eq!(stats.read_count, 1);
        assert_eq!(stats.read_bytes, (BLOCK_SIZE * 2) as u64);
    }
}

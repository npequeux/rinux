//! Block Device Interface
//!
//! Defines the interface that all block devices must implement.

use alloc::vec::Vec;
use core::fmt;

/// Block device trait
pub trait BlockDevice: Send + Sync {
    /// Get device name
    fn name(&self) -> &str;

    /// Get block size in bytes
    fn block_size(&self) -> usize;

    /// Get total number of blocks
    fn num_blocks(&self) -> u64;

    /// Read blocks from device
    ///
    /// # Arguments
    ///
    /// * `block_offset` - Starting block number
    /// * `buffer` - Buffer to read into
    ///
    /// # Returns
    ///
    /// Number of blocks read, or error
    fn read_blocks(&self, block_offset: u64, buffer: &mut [u8]) -> Result<usize, BlockDeviceError>;

    /// Write blocks to device
    ///
    /// # Arguments
    ///
    /// * `block_offset` - Starting block number
    /// * `buffer` - Buffer to write from
    ///
    /// # Returns
    ///
    /// Number of blocks written, or error
    fn write_blocks(&self, block_offset: u64, buffer: &[u8]) -> Result<usize, BlockDeviceError>;

    /// Flush any cached writes
    fn flush(&self) -> Result<(), BlockDeviceError>;

    /// Get device capacity in bytes
    fn capacity(&self) -> u64 {
        self.num_blocks() * self.block_size() as u64
    }

    /// Check if device is read-only
    fn is_read_only(&self) -> bool {
        false
    }

    /// Get device UUID (if available)
    fn uuid(&self) -> Option<[u8; 16]> {
        None
    }

    /// Get device serial number (if available)
    fn serial_number(&self) -> Option<&str> {
        None
    }

    /// Get device model (if available)
    fn model(&self) -> Option<&str> {
        None
    }
}

/// Block device error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDeviceError {
    /// Device not found
    NotFound,
    /// Invalid block offset
    InvalidOffset,
    /// Invalid buffer size
    InvalidBufferSize,
    /// Read error
    ReadError,
    /// Write error
    WriteError,
    /// Device is read-only
    ReadOnly,
    /// Device not ready
    NotReady,
    /// Timeout
    Timeout,
    /// Hardware error
    HardwareError,
    /// Out of memory
    OutOfMemory,
}

impl fmt::Display for BlockDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockDeviceError::NotFound => write!(f, "Device not found"),
            BlockDeviceError::InvalidOffset => write!(f, "Invalid block offset"),
            BlockDeviceError::InvalidBufferSize => write!(f, "Invalid buffer size"),
            BlockDeviceError::ReadError => write!(f, "Read error"),
            BlockDeviceError::WriteError => write!(f, "Write error"),
            BlockDeviceError::ReadOnly => write!(f, "Device is read-only"),
            BlockDeviceError::NotReady => write!(f, "Device not ready"),
            BlockDeviceError::Timeout => write!(f, "Operation timeout"),
            BlockDeviceError::HardwareError => write!(f, "Hardware error"),
            BlockDeviceError::OutOfMemory => write!(f, "Out of memory"),
        }
    }
}

/// Block device statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockDeviceStats {
    /// Number of read operations
    pub reads: u64,
    /// Number of write operations
    pub writes: u64,
    /// Number of bytes read
    pub bytes_read: u64,
    /// Number of bytes written
    pub bytes_written:  u64,
    /// Number of errors
    pub errors: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBlockDevice {
        name: &'static str,
        block_size: usize,
        num_blocks: u64,
    }

    impl BlockDevice for MockBlockDevice {
        fn name(&self) -> &str {
            self.name
        }

        fn block_size(&self) -> usize {
            self.block_size
        }

        fn num_blocks(&self) -> u64 {
            self.num_blocks
        }

        fn read_blocks(&self, _block_offset: u64, _buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
            Ok(0)
        }

        fn write_blocks(&self, _block_offset: u64, _buffer: &[u8]) -> Result<usize, BlockDeviceError> {
            Ok(0)
        }

        fn flush(&self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
    }

    #[test]
    fn test_block_device_capacity() {
        let device = MockBlockDevice {
            name: "test",
            block_size: 512,
            num_blocks: 1000,
        };
        assert_eq!(device.capacity(), 512000);
    }
}

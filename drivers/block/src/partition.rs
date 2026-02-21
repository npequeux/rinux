//! Partition Table Support
//!
//! Support for GPT and MBR partition tables

use crate::device::{BlockDevice, BlockDeviceError};
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::string::String;

/// Partition table type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableType {
    /// GUID Partition Table
    GPT,
    /// Master Boot Record
    MBR,
    /// Unknown or no partition table
    Unknown,
}

/// Partition information
#[derive(Debug, Clone)]
pub struct Partition {
    /// Partition number (1-based)
    pub number: u32,
    /// Partition type GUID (for GPT) or type code (for MBR)
    pub type_id: [u8; 16],
    /// Starting LBA
    pub start_lba: u64,
    /// Ending LBA (inclusive)
    pub end_lba: u64,
    /// Partition name (for GPT)
    pub name: String,
    /// Parent device
    pub device: Arc<dyn BlockDevice>,
}

impl Partition {
    /// Get partition size in blocks
    pub fn size_blocks(&self) -> u64 {
        self.end_lba - self.start_lba + 1
    }

    /// Get partition size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.size_blocks() * self.device.block_size() as u64
    }
}

/// GPT Header (simplified)
#[repr(C, packed)]
struct GptHeader {
    signature: [u8; 8],           // "EFI PART"
    revision: u32,
    header_size: u32,
    header_crc32: u32,
    _reserved: u32,
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_guid: [u8; 16],
    partition_entry_lba: u64,
    num_partition_entries: u32,
    partition_entry_size: u32,
    partition_array_crc32: u32,
}

/// GPT Partition Entry (simplified)
#[repr(C, packed)]
struct GptPartitionEntry {
    type_guid: [u8; 16],
    unique_guid: [u8; 16],
    starting_lba: u64,
    ending_lba: u64,
    attributes: u64,
    name: [u16; 36],  // UTF-16LE
}

/// MBR Partition Entry
#[repr(C, packed)]
struct MbrPartitionEntry {
    status: u8,
    first_chs: [u8; 3],
    partition_type: u8,
    last_chs: [u8; 3],
    first_lba: u32,
    num_sectors: u32,
}

/// Detect partition table type
pub fn detect_partition_table(device: &dyn BlockDevice) -> PartitionTableType {
    let mut buffer = [0u8; 512];
    
    // Read first sector
    if device.read_blocks(0, &mut buffer).is_err() {
        return PartitionTableType::Unknown;
    }
    
    // Check for GPT signature at LBA 1
    let mut gpt_buffer = [0u8; 512];
    if device.read_blocks(1, &mut gpt_buffer).is_ok() {
        if &gpt_buffer[0..8] == b"EFI PART" {
            return PartitionTableType::GPT;
        }
    }
    
    // Check for MBR signature
    if buffer[510] == 0x55 && buffer[511] == 0xAA {
        return PartitionTableType::MBR;
    }
    
    PartitionTableType::Unknown
}

/// Parse GPT partition table
pub fn parse_gpt(device: Arc<dyn BlockDevice>) -> Result<Vec<Partition>, &'static str> {
    let mut header_buffer = [0u8; 512];
    
    // Read GPT header from LBA 1
    device.read_blocks(1, &mut header_buffer)
        .map_err(|_| "Failed to read GPT header")?;
    
    // Verify signature
    if &header_buffer[0..8] != b"EFI PART" {
        return Err("Invalid GPT signature");
    }
    
    // Parse header (simplified - not reading all fields safely)
    let num_entries = u32::from_le_bytes([
        header_buffer[80], header_buffer[81],
        header_buffer[82], header_buffer[83],
    ]);
    
    let entry_lba = u64::from_le_bytes([
        header_buffer[72], header_buffer[73], header_buffer[74], header_buffer[75],
        header_buffer[76], header_buffer[77], header_buffer[78], header_buffer[79],
    ]);
    
    let mut partitions = Vec::new();
    
    // Read partition entries
    // Each entry is typically 128 bytes, but we should use the value from header
    // For simplicity, we'll assume 128 bytes and read up to 4 entries per sector
    
    let entries_per_sector = 512 / 128;
    let sectors_to_read = ((num_entries as usize) + entries_per_sector - 1) / entries_per_sector;
    
    for sector in 0..sectors_to_read {
        let mut entry_buffer = [0u8; 512];
        device.read_blocks(entry_lba + sector as u64, &mut entry_buffer)
            .map_err(|_| "Failed to read partition entries")?;
        
        for i in 0..entries_per_sector {
            let offset = i * 128;
            
            // Check if this is a valid entry (non-zero type GUID)
            let type_guid: [u8; 16] = entry_buffer[offset..offset+16].try_into().unwrap();
            if type_guid == [0u8; 16] {
                continue;  // Empty entry
            }
            
            let start_lba = u64::from_le_bytes(
                entry_buffer[offset+32..offset+40].try_into().unwrap()
            );
            let end_lba = u64::from_le_bytes(
                entry_buffer[offset+40..offset+48].try_into().unwrap()
            );
            
            partitions.push(Partition {
                number: (sector * entries_per_sector + i + 1) as u32,
                type_id: type_guid,
                start_lba,
                end_lba,
                name: String::from("partition"),  // Would parse UTF-16 name
                device: Arc::clone(&device),
            });
        }
    }
    
    Ok(partitions)
}

/// Parse MBR partition table
pub fn parse_mbr(device: Arc<dyn BlockDevice>) -> Result<Vec<Partition>, &'static str> {
    let mut buffer = [0u8; 512];
    
    // Read MBR
    device.read_blocks(0, &mut buffer)
        .map_err(|_| "Failed to read MBR")?;
    
    // Check signature
    if buffer[510] != 0x55 || buffer[511] != 0xAA {
        return Err("Invalid MBR signature");
    }
    
    let mut partitions = Vec::new();
    
    // Parse 4 primary partition entries (offset 446, each 16 bytes)
    for i in 0..4 {
        let offset = 446 + i * 16;
        let partition_type = buffer[offset + 4];
        
        if partition_type == 0 {
            continue;  // Empty entry
        }
        
        let first_lba = u32::from_le_bytes([
            buffer[offset + 8],
            buffer[offset + 9],
            buffer[offset + 10],
            buffer[offset + 11],
        ]) as u64;
        
        let num_sectors = u32::from_le_bytes([
            buffer[offset + 12],
            buffer[offset + 13],
            buffer[offset + 14],
            buffer[offset + 15],
        ]) as u64;
        
        // Create type_id from partition type byte
        let mut type_id = [0u8; 16];
        type_id[0] = partition_type;
        
        partitions.push(Partition {
            number: (i + 1) as u32,
            type_id,
            start_lba: first_lba,
            end_lba: first_lba + num_sectors - 1,
            name: String::from("partition"),
            device: Arc::clone(&device),
        });
    }
    
    Ok(partitions)
}

/// Scan all block devices for partitions
pub fn scan_all() {
    // Get number of registered block devices
    let device_count = crate::device_count();
    
    // Scan each device for partitions
    for i in 0..device_count {
        if let Some(device) = crate::get_device(i) {
            if let Err(e) = scan_device(device) {
                // Log error (would use printk in full implementation)
                let _ = e; // Suppress unused variable warning
            }
        }
    }
}

/// Scan a single block device for partitions
fn scan_device(device: Arc<dyn BlockDevice>) -> Result<(), &'static str> {
    // Read first sector to check for partition table
    let mut buffer = vec![0u8; 512];
    
    // Try to read first sector
    if device.read(0, &mut buffer).is_err() {
        return Err("Failed to read device");
    }
    
    // Check for GPT signature
    if is_gpt(&buffer) {
        let _ = parse_gpt_partitions(device)?;
    } else if is_mbr(&buffer) {
        let _ = parse_mbr_partitions(device)?;
    }
    
    Ok(())
}

/// Parse GPT partitions from a device
fn parse_gpt_partitions(device: Arc<dyn BlockDevice>) -> Result<Vec<Partition>, &'static str> {
    let gpt_header = read_gpt_header(device.clone())?;
    let mut partitions = Vec::new();
    
    // Calculate number of partition entries
    let entry_count = gpt_header.num_partition_entries.min(128); // Safety limit
    
    // Read partition entries
    for i in 0..entry_count {
        if let Ok(entry) = read_gpt_entry(device.clone(), &gpt_header, i) {
            if !is_zero_guid(&entry.partition_type_guid) {
                // Valid partition found
                partitions.push(Partition {
                    start_lba: entry.first_lba,
                    end_lba: entry.last_lba,
                    name: String::from("partition"),
                    device: Arc::clone(&device),
                });
            }
        }
    }
    
    Ok(partitions)
}

/// Parse MBR partitions from a device  
fn parse_mbr_partitions(device: Arc<dyn BlockDevice>) -> Result<Vec<Partition>, &'static str> {
    let mut buffer = vec![0u8; 512];
    device.read(0, &mut buffer)?;
    
    let mut partitions = Vec::new();
    
    // Parse primary partitions
    for i in 0..4 {
        let offset = 446 + (i * 16);
        if offset + 16 > buffer.len() {
            break;
        }
        
        let partition_type = buffer[offset + 4];
        if partition_type == 0 {
            continue; // Empty partition entry
        }
        
        let lba_start = u32::from_le_bytes([
            buffer[offset + 8],
            buffer[offset + 9],
            buffer[offset + 10],
            buffer[offset + 11],
        ]) as u64;
        
        let num_sectors = u32::from_le_bytes([
            buffer[offset + 12],
            buffer[offset + 13],
            buffer[offset + 14],
            buffer[offset + 15],
        ]) as u64;
        
        if num_sectors > 0 {
            // Valid partition found
            partitions.push(Partition {
                start_lba: lba_start,
                end_lba: lba_start + num_sectors - 1,
                name: String::from("partition"),
                device: Arc::clone(&device),
            });
        }
    }
    
    Ok(partitions)
}

/// Check if a GUID is all zeros
fn is_zero_guid(guid: &[u8; 16]) -> bool {
    guid.iter().all(|&b| b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_table_type() {
        // Tests would go here
    }
}

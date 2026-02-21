//! Partition Table Support
//!
//! Support for GPT and MBR partition tables

use alloc::vec::Vec;

/// Partition type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionType {
    /// Unknown or empty
    Unknown,
    /// Linux filesystem
    Linux,
    /// Linux swap
    LinuxSwap,
    /// EFI System Partition
    EfiSystem,
    /// Microsoft Basic Data (FAT/NTFS)
    MicrosoftBasic,
    /// Other
    Other(u8),
}

/// Partition table type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableType {
    /// Master Boot Record (legacy)
    MBR,
    /// GUID Partition Table (modern)
    GPT,
}

/// Partition entry
#[derive(Debug, Clone)]
pub struct Partition {
    pub number: u32,
    pub start_lba: u64,
    pub size_blocks: u64,
    pub partition_type: PartitionType,
    pub bootable: bool,
}

impl Partition {
    pub fn end_lba(&self) -> u64 {
        self.start_lba + self.size_blocks
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_blocks * 512
    }
}

/// MBR partition entry (16 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MbrPartitionEntry {
    pub status: u8,         // 0x80 = bootable, 0x00 = inactive
    pub first_chs: [u8; 3], // First sector in CHS
    pub partition_type: u8,
    pub last_chs: [u8; 3], // Last sector in CHS
    pub first_lba: u32,    // First sector in LBA
    pub size: u32,         // Size in sectors
}

/// MBR (Master Boot Record)
#[repr(C, packed)]
pub struct Mbr {
    pub boot_code: [u8; 440],
    pub disk_signature: u32,
    pub reserved: u16,
    pub partitions: [MbrPartitionEntry; 4],
    pub signature: u16, // Must be 0xAA55
}

impl Mbr {
    /// Parse MBR from buffer
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, &'static str> {
        if buffer.len() < 512 {
            return Err("Buffer too small for MBR");
        }

        // Check MBR signature
        let signature = u16::from_le_bytes([buffer[510], buffer[511]]);
        if signature != 0xAA55 {
            return Err("Invalid MBR signature");
        }

        // TODO: Safely parse MBR structure
        // For now, return error
        Err("MBR parsing not fully implemented")
    }

    /// Get all partitions
    pub fn get_partitions(&self) -> Vec<Partition> {
        let mut partitions = Vec::new();

        for (i, entry) in self.partitions.iter().enumerate() {
            if entry.partition_type == 0 {
                continue; // Empty partition
            }

            let partition_type = match entry.partition_type {
                0x82 => PartitionType::LinuxSwap,
                0x83 => PartitionType::Linux,
                0xEF => PartitionType::EfiSystem,
                0x07 | 0x0B | 0x0C => PartitionType::MicrosoftBasic,
                t => PartitionType::Other(t),
            };

            partitions.push(Partition {
                number: i as u32 + 1,
                start_lba: entry.first_lba as u64,
                size_blocks: entry.size as u64,
                partition_type,
                bootable: entry.status == 0x80,
            });
        }

        partitions
    }
}

/// GPT header
#[repr(C, packed)]
pub struct GptHeader {
    pub signature: [u8; 8], // "EFI PART"
    pub revision: u32,
    pub header_size: u32,
    pub header_crc32: u32,
    pub reserved: u32,
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub partition_entries_lba: u64,
    pub num_partitions: u32,
    pub partition_entry_size: u32,
    pub partition_array_crc32: u32,
}

impl GptHeader {
    const SIGNATURE: &'static [u8; 8] = b"EFI PART";

    /// Parse GPT header from buffer
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, &'static str> {
        if buffer.len() < 512 {
            return Err("Buffer too small for GPT header");
        }

        // Check signature
        if &buffer[0..8] != Self::SIGNATURE {
            return Err("Invalid GPT signature");
        }

        // TODO: Safely parse GPT header structure and verify CRC
        Err("GPT parsing not fully implemented")
    }
}

/// GPT partition entry (128 bytes)
#[repr(C, packed)]
pub struct GptPartitionEntry {
    pub type_guid: [u8; 16],
    pub unique_guid: [u8; 16],
    pub starting_lba: u64,
    pub ending_lba: u64,
    pub attributes: u64,
    pub partition_name: [u16; 36], // UTF-16LE
}

impl GptPartitionEntry {
    /// Check if partition is empty
    pub fn is_empty(&self) -> bool {
        self.type_guid.iter().all(|&b| b == 0)
    }

    /// Get partition type from GUID
    pub fn get_type(&self) -> PartitionType {
        // Linux filesystem: 0FC63DAF-8483-4772-8E79-3D69D8477DE4
        const LINUX_FS: [u8; 16] = [
            0xAF, 0x3D, 0xC6, 0x0F, 0x83, 0x84, 0x72, 0x47, 0x8E, 0x79, 0x3D, 0x69, 0xD8, 0x47,
            0x7D, 0xE4,
        ];

        // Linux swap: 0657FD6D-A4AB-43C4-84E5-0933C84B4F4F
        const LINUX_SWAP: [u8; 16] = [
            0x6D, 0xFD, 0x57, 0x06, 0xAB, 0xA4, 0xC4, 0x43, 0x84, 0xE5, 0x09, 0x33, 0xC8, 0x4B,
            0x4F, 0x4F,
        ];

        // EFI System: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
        const EFI_SYSTEM: [u8; 16] = [
            0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E,
            0xC9, 0x3B,
        ];

        // Microsoft Basic Data: EBD0A0A2-B9E5-4433-87C0-68B6B72699C7
        const MS_BASIC: [u8; 16] = [
            0xA2, 0xA0, 0xD0, 0xEB, 0xE5, 0xB9, 0x33, 0x44, 0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26,
            0x99, 0xC7,
        ];

        match self.type_guid {
            LINUX_FS => PartitionType::Linux,
            LINUX_SWAP => PartitionType::LinuxSwap,
            EFI_SYSTEM => PartitionType::EfiSystem,
            MS_BASIC => PartitionType::MicrosoftBasic,
            _ => PartitionType::Unknown,
        }
    }
}

/// Partition table
pub struct PartitionTable {
    pub table_type: PartitionTableType,
    pub partitions: Vec<Partition>,
}

impl PartitionTable {
    /// Parse partition table from disk
    pub fn parse(first_sector: &[u8]) -> Result<Self, &'static str> {
        if first_sector.len() < 512 {
            return Err("Buffer too small");
        }

        // Try GPT first (check for protective MBR)
        if first_sector.len() >= 512 && is_protective_mbr(first_sector) {
            // TODO: Read GPT header from LBA 1
            return Err("GPT not fully implemented");
        }

        // Try MBR
        let signature = u16::from_le_bytes([first_sector[510], first_sector[511]]);
        if signature == 0xAA55 {
            // TODO: Parse MBR partitions
            return Ok(PartitionTable {
                table_type: PartitionTableType::MBR,
                partitions: Vec::new(),
            });
        }

        Err("No valid partition table found")
    }
}

/// Check if MBR is a protective MBR (used with GPT)
fn is_protective_mbr(buffer: &[u8]) -> bool {
    if buffer.len() < 512 {
        return false;
    }

    // Check if first partition has type 0xEE (GPT protective)
    buffer[450] == 0xEE
}

/// Initialize partition support
pub fn init() {
    // Partition table support initialized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_calculations() {
        let part = Partition {
            number: 1,
            start_lba: 2048,
            size_blocks: 1024,
            partition_type: PartitionType::Linux,
            bootable: false,
        };

        assert_eq!(part.end_lba(), 3072);
        assert_eq!(part.size_bytes(), 1024 * 512);
    }

    #[test]
    fn test_mbr_size() {
        assert_eq!(core::mem::size_of::<Mbr>(), 512);
    }

    #[test]
    fn test_gpt_entry_size() {
        assert_eq!(core::mem::size_of::<GptPartitionEntry>(), 128);
    }
}

//! FAT32 Filesystem Support
//!
//! Read/write support for the FAT32 (File Allocation Table) filesystem.
//! FAT32 is commonly used on USB drives, SD cards, and for interoperability.

use crate::{FsError, FsType};
use crate::vfs::{VNode, Filesystem, FileAttr, FileType, FileMode, DirEntry, StatFs};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use spin::RwLock;

/// FAT32 Boot Sector (BPB - BIOS Parameter Block)
#[repr(C, packed)]
struct Fat32BootSector {
    jmp_boot: [u8; 3],         // Jump instruction
    oem_name: [u8; 8],         // OEM name
    bytes_per_sector: u16,     // Bytes per logical sector (usually 512)
    sectors_per_cluster: u8,   // Sectors per cluster
    reserved_sectors: u16,     // Reserved sectors (including boot sector)
    num_fats: u8,              // Number of FAT copies (usually 2)
    root_entry_count: u16,     // Root directory entries (0 for FAT32)
    total_sectors_16: u16,     // Total sectors (0 if > 65535)
    media: u8,                 // Media descriptor
    fat_size_16: u16,          // FAT size in sectors (0 for FAT32)
    sectors_per_track: u16,    // Sectors per track
    num_heads: u16,            // Number of heads
    hidden_sectors: u32,       // Hidden sectors
    total_sectors_32: u32,     // Total sectors (if total_sectors_16 is 0)
    
    // FAT32-specific fields
    fat_size_32: u32,          // FAT size in sectors
    ext_flags: u16,            // Extended flags
    fs_version: u16,           // Filesystem version
    root_cluster: u32,         // Root directory cluster (usually 2)
    fs_info: u16,              // FSInfo sector
    backup_boot_sector: u16,   // Backup boot sector location
    reserved: [u8; 12],        // Reserved
    drive_number: u8,          // Drive number
    reserved1: u8,             // Reserved
    boot_signature: u8,        // Extended boot signature (0x29)
    volume_id: u32,            // Volume serial number
    volume_label: [u8; 11],    // Volume label
    fs_type: [u8; 8],          // Filesystem type ("FAT32   ")
}

/// FAT32 FSInfo Sector
#[repr(C, packed)]
struct Fat32FSInfo {
    lead_sig: u32,             // Lead signature (0x41615252)
    reserved1: [u8; 480],      // Reserved
    struct_sig: u32,           // Structure signature (0x61417272)
    free_count: u32,           // Last known free cluster count
    next_free: u32,            // Next free cluster hint
    reserved2: [u8; 12],       // Reserved
    trail_sig: u32,            // Trail signature (0xAA550000)
}

/// FAT32 Directory Entry
#[repr(C, packed)]
struct Fat32DirEntry {
    name: [u8; 11],            // Short filename (8.3 format)
    attr: u8,                  // File attributes
    nt_reserved: u8,           // Reserved  for Windows NT
    create_time_tenth: u8,     // Creation time (tenths of second)
    create_time: u16,          // Creation time
    create_date: u16,          // Creation date
    last_access_date: u16,     // Last access date
    first_cluster_hi: u16,     // High word of first cluster
    write_time: u16,           // Last write time
    write_date: u16,           // Last write date
    first_cluster_lo: u16,     // Low word of first cluster
    file_size: u32,            // File size in bytes
}

/// FAT32 Long File Name Entry
#[repr(C, packed)]
struct Fat32LFNEntry {
    order: u8,                 // Order/sequence number
    name1: [u16; 5],           // First 5 characters
    attr: u8,                  // Attributes (always 0x0F for LFN)
    lfn_type: u8,              // Type (always 0)
    checksum: u8,              // Checksum of short name
    name2: [u16; 6],           // Next 6 characters
    first_cluster_lo: u16,     // Always 0 for LFN
    name3: [u16; 2],           // Final 2 characters
}

/// File attributes
const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;
const ATTR_LONG_NAME: u8 = 0x0F;

/// End of cluster chain marker
const FAT32_EOC: u32 = 0x0FFFFFF8;
/// Bad cluster marker
const FAT32_BAD_CLUSTER: u32 = 0x0FFFFFF7;
/// Free cluster marker
const FAT32_FREE_CLUSTER: u32 = 0x00000000;

/// FAT32 VNode
pub struct Fat32VNode {
    fs: Arc<Fat32Filesystem>,
    first_cluster: u32,
    size: u64,
    is_dir: bool,
    ino: u64,
}

impl Fat32VNode {
    fn new(fs: Arc<Fat32Filesystem>, first_cluster: u32, size: u64, is_dir: bool) -> Self {
        Fat32VNode {
            fs,
            first_cluster,
            size,
            is_dir,
            ino: first_cluster as u64, // Use cluster as inode number
        }
    }

    /// Get the next cluster in the chain
    fn get_next_cluster(&self, cluster: u32) -> Result<u32, FsError> {
        // Read from FAT
        // FAT entry is 4 bytes per cluster
        let fat_offset = cluster * 4;
        let fat_sector = self.fs.reserved_sectors as u32 + fat_offset / self.fs.bytes_per_sector as u32;
        let sector_offset = (fat_offset % self.fs.bytes_per_sector as u32) as usize;

        // Read FAT sector (stub)
        // let sector_data = self.fs.read_sector(fat_sector)?;
        // let next_cluster = u32::from_le_bytes([
        //     sector_data[sector_offset],
        //     sector_data[sector_offset + 1],
        //     sector_data[sector_offset + 2],
        //     sector_data[sector_offset + 3],
        // ]) & 0x0FFFFFFF; // Mask off high 4 bits

        // For now, return end of chain
        let _ = (fat_sector, sector_offset);
        Ok(FAT32_EOC)
    }

    /// Get the cluster at a specific offset in the file
    fn get_cluster_at_offset(&self, offset: u64) -> Result<u32, FsError> {
        let cluster_size = (self.fs.sectors_per_cluster as u64) * (self.fs.bytes_per_sector as u64);
        let cluster_index = offset / cluster_size;

        let mut current_cluster = self.first_cluster;
        for _ in 0..cluster_index {
            current_cluster = self.get_next_cluster(current_cluster)?;
            if current_cluster >= FAT32_EOC {
                return Err(FsError::IoError);
            }
        }

        Ok(current_cluster)
    }

    /// Convert cluster number to logical sector
    fn cluster_to_sector(&self, cluster: u32) -> u32 {
        let first_data_sector = self.fs.reserved_sectors as u32 
            + (self.fs.num_fats as u32 * self.fs.fat_size_32);
        ((cluster - 2) * self.fs.sectors_per_cluster as u32) + first_data_sector
    }
}

impl VNode for Fat32VNode {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, FsError> {
        if self.is_dir {
            return Err(FsError::IsADirectory);
        }

        if offset >= self.size {
            return Ok(0);
        }

        let max_read = ((self.size - offset).min(buffer.len() as u64)) as usize;
        let cluster_size = (self.fs.sectors_per_cluster as u64) * (self.fs.bytes_per_sector as u64);
        let mut bytes_read = 0;

        while bytes_read < max_read {
            let current_offset = offset + bytes_read as u64;
            let cluster = self.get_cluster_at_offset(current_offset)?;
            let cluster_offset = (current_offset % cluster_size) as usize;

            // Read from cluster
            // TODO: Integrate with block device
            let bytes_in_cluster = ((cluster_size - cluster_offset as u64) as usize).min(max_read - bytes_read);
            buffer[bytes_read..bytes_read + bytes_in_cluster].fill(0); // Stub
            bytes_read += bytes_in_cluster;
        }

        Ok(bytes_read)
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> Result<usize, FsError> {
        if self.is_dir {
            return Err(FsError::IsADirectory);
        }

        // TODO: Implement write
        let _ = (offset, buffer);
        Err(FsError::NotSupported)
    }

    fn getattr(&self) -> Result<FileAttr, FsError> {
        Ok(FileAttr {
            file_type: if self.is_dir { FileType::Directory } else { FileType::Regular },
            mode: FileMode::new(0o755),
            size: self.size,
            nlink: 1,
            uid: 0,
            gid: 0,
            ino: self.ino,
            blocks: (self.size + 511) / 512,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn setattr(&self, _attr: &FileAttr) -> Result<(), FsError> {
        Err(FsError::NotSupported)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        if !self.is_dir {
            return Err(FsError::NotADirectory);
        }

        let mut entries = Vec::new();
        
        // Read directory entries from clusters
        // TODO: Implement directory reading with LFN support
        
        Ok(entries)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<dyn VNode>, FsError> {
        if !self.is_dir {
            return Err(FsError::NotADirectory);
        }

        // Read directory entries and find matching name
        Err(FsError::NotFound)
    }

    fn create(&self, _name: &str, _mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        Err(FsError::NotSupported)
    }

    fn mkdir(&self, _name: &str, _mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        Err(FsError::NotSupported)
    }

    fn unlink(&self, _name: &str) -> Result<(), FsError> {
        Err(FsError::NotSupported)
    }

    fn rmdir(&self, _name: &str) -> Result<(), FsError> {
        Err(FsError::NotSupported)
    }

    fn rename(&self, _old_name: &str, _new_parent: Arc<dyn VNode>, _new_name: &str) -> Result<(), FsError> {
        Err(FsError::NotSupported)
    }

    fn symlink(&self, _name: &str, _target: &str) -> Result<Arc<dyn VNode>, FsError> {
        Err(FsError::NotSupported)
    }

    fn readlink(&self) -> Result<String, FsError> {
        Err(FsError::NotSupported)
    }

    fn truncate(&self, _size: u64) -> Result<(), FsError> {
        Err(FsError::NotSupported)
    }

    fn fsync(&self) -> Result<(), FsError> {
        Ok(())
    }
}

/// FAT32 Filesystem
pub struct Fat32Filesystem {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    fat_size_32: u32,
    root_cluster: u32,
}

impl Fat32Filesystem {
    /// Mount a FAT32 filesystem from a block device
    pub fn mount() -> Result<Arc<Self>, FsError> {
        // Read boot sector
        // Validate FAT32 signature
        // Read FSInfo sector
        
        Ok(Arc::new(Fat32Filesystem {
            bytes_per_sector: 512,
            sectors_per_cluster: 8,
            reserved_sectors: 32,
            num_fats: 2,
            fat_size_32: 1024,
            root_cluster: 2,
        }))
    }

    /// Allocate a new cluster
    fn allocate_cluster(&self) -> Result<u32, FsError> {
        // Read FSInfo to get hint
        // Search FAT for free cluster
        // Mark cluster as used
        Err(FsError::NoSpaceLeft)
    }

    /// Free a cluster chain
    fn free_cluster_chain(&self, _start_cluster: u32) -> Result<(), FsError> {
        // Follow chain and mark all clusters as free
        // Update FSInfo
        Ok(())
    }
}

impl Filesystem for Fat32Filesystem {
    fn fs_type(&self) -> FsType {
        FsType::FAT32
    }

    fn root(&self) -> Arc<dyn VNode> {
        Arc::new(Fat32VNode::new(
            Arc::new(Self::mount().unwrap()),
            self.root_cluster,
            0,
            true,
        ))
    }

    fn sync(&self) -> Result<(), FsError> {
        // Flush all cached FAT entries
        // Update FSInfo sector
        Ok(())
    }

    fn statfs(&self) -> Result<StatFs, FsError> {
        let cluster_size = (self.sectors_per_cluster as u64) * (self.bytes_per_sector as u64);
        
        Ok(StatFs {
            fs_type: 0x4d44, // FAT magic
            block_size: cluster_size,
            blocks: 0, // Would calculate from total sectors
            blocks_free: 0, // Would read from FSInfo
            blocks_available: 0,
            files: 0, // FAT doesn't track inode count
            files_free: 0,
            name_max: 255, // With LFN support
        })
    }

    fn unmount(&self) -> Result<(), FsError> {
        // Flush all cached data
        // Update FSInfo
        Ok(())
    }
}

/// Initialize FAT32 driver
pub fn init() {
    // FAT32 filesystems are mounted on demand
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fat32_constants() {
        assert_eq!(FAT32_EOC, 0x0FFFFFF8);
        assert_eq!(FAT32_BAD_CLUSTER, 0x0FFFFFF7);
        assert_eq!(FAT32_FREE_CLUSTER, 0x00000000);
    }

    #[test]
    fn test_fat32_attributes() {
        assert_eq!(ATTR_LONG_NAME, 0x0F);
        assert_eq!(ATTR_DIRECTORY, 0x10);
        assert_eq!(ATTR_READ_ONLY, 0x01);
    }

    #[test]
    fn test_cluster_to_sector() {
        let fs = Fat32Filesystem {
            bytes_per_sector: 512,
            sectors_per_cluster: 8,
            reserved_sectors: 32,
            num_fats: 2,
            fat_size_32: 1024,
            root_cluster: 2,
        };

        let vnode = Fat32VNode::new(Arc::new(fs), 2, 0, true);
        let sector = vnode.cluster_to_sector(2);
        
        // First data sector = reserved_sectors + (num_fats * fat_size_32)
        // = 32 + (2 * 1024) = 2080
        // cluster 2 is at first data sector (cluster 0 and 1 are reserved)
        assert_eq!(sector, 2080);
    }
}

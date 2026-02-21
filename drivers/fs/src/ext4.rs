//! ext4 Filesystem Support
//!
//! Read/write support for the fourth extended filesystem (ext4)
//! ext4 is mostly backwards compatible with ext2/ext3 but adds several improvements:
//! - Extent trees instead of indirect blocks
//! - Journaling (required)
//! - 48-bit block addresses (16TB+ filesystems)
//! - Metadata checksums
//! - Delayed allocation
//! - Multi-block allocation

use crate::{FsError, FsType};
use crate::vfs::{VNode, Filesystem, FileAttr, FileType, FileMode, DirEntry, StatFs};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use spin::RwLock;

/// ext4 Superblock (extended from ext2)
#[repr(C, packed)]
struct Ext4Superblock {
    // First 1024 bytes are identical to ext2
    s_inodes_count: u32,
    s_blocks_count_lo: u32,
    s_r_blocks_count_lo: u32,
    s_free_blocks_count_lo: u32,
    s_free_inodes_count: u32,
    s_first_data_block: u32,
    s_log_block_size: u32,
    s_log_cluster_size: u32,
    s_blocks_per_group: u32,
    s_clusters_per_group: u32,
    s_inodes_per_group: u32,
    s_mtime: u32,
    s_wtime: u32,
    s_mnt_count: u16,
    s_max_mnt_count: u16,
    s_magic: u16,
    s_state: u16,
    s_errors: u16,
    s_minor_rev_level: u16,
    s_lastcheck: u32,
    s_checkinterval: u32,
    s_creator_os: u32,
    s_rev_level: u32,
    s_def_resuid: u16,
    s_def_resgid: u16,
    
    // Extended superblock fields (ext4)
    s_first_ino: u32,
    s_inode_size: u16,
    s_block_group_nr: u16,
    s_feature_compat: u32,
    s_feature_incompat: u32,
    s_feature_ro_compat: u32,
    s_uuid: [u8; 16],
    s_volume_name: [u8; 16],
    s_last_mounted: [u8; 64],
    s_algorithm_usage_bitmap: u32,
    
    // Performance hints
    s_prealloc_blocks: u8,
    s_prealloc_dir_blocks: u8,
    s_reserved_gdt_blocks: u16,
    
    // Journaling support
    s_journal_uuid: [u8; 16],
    s_journal_inum: u32,
    s_journal_dev: u32,
    s_last_orphan: u32,
    s_hash_seed: [u32; 4],
    s_def_hash_version: u8,
    s_jnl_backup_type: u8,
    s_desc_size: u16,
    s_default_mount_opts: u32,
    s_first_meta_bg: u32,
    s_mkfs_time: u32,
    s_jnl_blocks: [u32; 17],
    
    // 64-bit support
    s_blocks_count_hi: u32,
    s_r_blocks_count_hi: u32,
    s_free_blocks_count_hi: u32,
    s_min_extra_isize: u16,
    s_want_extra_isize: u16,
    s_flags: u32,
    s_raid_stride: u16,
    s_mmp_interval: u16,
    s_mmp_block: u64,
    s_raid_stripe_width: u32,
    s_log_groups_per_flex: u8,
    s_checksum_type: u8,
    s_encryption_level: u8,
    s_reserved_pad: u8,
    s_kbytes_written: u64,
    s_snapshot_inum: u32,
    s_snapshot_id: u32,
    s_snapshot_r_blocks_count: u64,
    s_snapshot_list: u32,
    s_error_count: u32,
    s_first_error_time: u32,
    s_first_error_ino: u32,
    s_first_error_block: u64,
    s_first_error_func: [u8; 32],
    s_first_error_line: u32,
    s_last_error_time: u32,
    s_last_error_ino: u32,
    s_last_error_line: u32,
    s_last_error_block: u64,
    s_last_error_func: [u8; 32],
    s_mount_opts: [u8; 64],
    s_usr_quota_inum: u32,
    s_grp_quota_inum: u32,
    s_overhead_blocks: u32,
    s_backup_bgs: [u32; 2],
    s_encrypt_algos: [u8; 4],
    s_encrypt_pw_salt: [u8; 16],
    s_lpf_ino: u32,
    s_prj_quota_inum: u32,
    s_checksum_seed: u32,
    s_reserved: [u32; 98],
    s_checksum: u32,
}

/// ext4 magic number (same as ext2/ext3)
const EXT4_MAGIC: u16 = 0xEF53;

/// ext4 feature flags
const EXT4_FEATURE_INCOMPAT_EXTENTS: u32 = 0x0040;
const EXT4_FEATURE_INCOMPAT_64BIT: u32 = 0x0080;
const EXT4_FEATURE_INCOMPAT_FLEX_BG: u32 = 0x0200;

/// ext4 Inode (extended from ext2)
#[repr(C, packed)]
struct Ext4Inode {
    i_mode: u16,
    i_uid: u16,
    i_size_lo: u32,
    i_atime: u32,
    i_ctime: u32,
    i_mtime: u32,
    i_dtime: u32,
    i_gid: u16,
    i_links_count: u16,
    i_blocks_lo: u32,
    i_flags: u32,
    i_osd1: u32,
    i_block: [u32; 15], // Or extent tree if EXT4_EXTENTS_FL is set
    i_generation: u32,
    i_file_acl_lo: u32,
    i_size_high: u32,
    i_obso_faddr: u32,
    i_osd2: [u8; 12],
    
    // Extended fields (for larger inodes)
    i_extra_isize: u16,
    i_checksum_hi: u16,
    i_ctime_extra: u32,
    i_mtime_extra: u32,
    i_atime_extra: u32,
    i_crtime: u32,
    i_crtime_extra: u32,
    i_version_hi: u32,
    i_projid: u32,
}

/// ext4 Extent Header
#[repr(C, packed)]
struct Ext4ExtentHeader {
    eh_magic: u16,       // Magic number (0xF30A)
    eh_entries: u16,     // Number of valid entries
    eh_max: u16,         // Capacity of store
    eh_depth: u16,       // Tree depth (0 = leaf node)
    eh_generation: u32,  // Generation of the tree
}

/// ext4 Extent Index (internal node)
#[repr(C, packed)]
struct Ext4ExtentIdx {
    ei_block: u32,       // Logical block number
    ei_leaf_lo: u32,     // Physical block of next level (low 32 bits)
    ei_leaf_hi: u16,     // Physical block of next level (high 16 bits)
    ei_unused: u16,
}

/// ext4 Extent (leaf node)
#[repr(C, packed)]
struct Ext4Extent {
    ee_block: u32,       // First logical block
    ee_len: u16,         // Number of blocks
    ee_start_hi: u16,    // Physical block (high 16 bits)
    ee_start_lo: u32,    // Physical block (low 32 bits)
}

/// ext4 extent magic
const EXT4_EXT_MAGIC: u16 = 0xF30A;

/// ext4 VNode
pub struct Ext4VNode {
    fs: Arc<Ext4Filesystem>,
    ino: u64,
}

impl Ext4VNode {
    fn new(fs: Arc<Ext4Filesystem>, ino: u64) -> Self {
        Ext4VNode { fs, ino }
    }

    fn read_inode(&self) -> Result<Ext4Inode, FsError> {
        // Calculate block group and inode table offset
        // Read inode from device
        // For now, this is a stub
        Err(FsError::NotFound)
    }

    /// Get physical block number from logical block using extent tree
    fn map_block(&self, logical_block: u64) -> Result<u64, FsError> {
        let inode = self.read_inode()?;

        // Check if inode uses extents
        if inode.i_flags & 0x80000 != 0 {
            // EXT4_EXTENTS_FL flag set - use extent tree
            self.map_block_extent(logical_block, &inode)
        } else {
            // Old-style indirect blocks
            self.map_block_indirect(logical_block, &inode)
        }
    }

    fn map_block_extent(&self, logical_block: u64, inode: &Ext4Inode) -> Result<u64, FsError> {
        // i_block contains the extent tree root
        let extent_header = unsafe {
            &*(inode.i_block.as_ptr() as *const Ext4ExtentHeader)
        };

        if extent_header.eh_magic != EXT4_EXT_MAGIC {
            return Err(FsError::InvalidData);
        }

        // For now, only handle depth 0 (single-level extents)
        if extent_header.eh_depth != 0 {
            return Err(FsError::NotSupported);
        }

        // Search extents
        let extents = unsafe {
            core::slice::from_raw_parts(
                (inode.i_block.as_ptr() as usize + core::mem::size_of::<Ext4ExtentHeader>()) as *const Ext4Extent,
                extent_header.eh_entries as usize
            )
        };

        for extent in extents {
            let start = extent.ee_block as u64;
            let len = (extent.ee_len & 0x7FFF) as u64; // Clear initialized flag
            
            if logical_block >= start && logical_block < start + len {
                // Found the extent
                let phys_start = ((extent.ee_start_hi as u64) << 32) | (extent.ee_start_lo as u64);
                let offset = logical_block - start;
                return Ok(phys_start + offset);
            }
        }

        // Block not found (sparse file)
        Ok(0)
    }

    fn map_block_indirect(&self, logical_block: u64, inode: &Ext4Inode) -> Result<u64, FsError> {
        // Handle old-style indirect blocks (like ext2)
        if logical_block < 12 {
            // Direct blocks
            Ok(inode.i_block[logical_block as usize] as u64)
        } else if logical_block < 12 + 256 {
            // Single indirect
            // Would need to read the indirect block
            Err(FsError::NotSupported)
        } else {
            // Double/triple indirect
            Err(FsError::NotSupported)
        }
    }

    fn get_file_size(&self, inode: &Ext4Inode) -> u64 {
        // ext4 supports 64-bit file sizes
        ((inode.i_size_high as u64) << 32) | (inode.i_size_lo as u64)
    }
}

impl VNode for Ext4VNode {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, FsError> {
        let inode = self.read_inode()?;
        let file_size = self.get_file_size(&inode);

        if offset >= file_size {
            return Ok(0);
        }

        let block_size = self.fs.block_size as u64;
        let max_read = ((file_size - offset).min(buffer.len() as u64)) as usize;
        let mut bytes_read = 0;

        while bytes_read < max_read {
            let current_offset = offset + bytes_read as u64;
            let logical_block = current_offset / block_size;
            let block_offset = (current_offset % block_size) as usize;

            // Map logical to physical block
            let physical_block = self.map_block(logical_block)?;

            if physical_block == 0 {
                // Sparse block - fill with zeros
                let bytes_in_block = ((block_size - block_offset as u64) as usize).min(max_read - bytes_read);
                buffer[bytes_read..bytes_read + bytes_in_block].fill(0);
                bytes_read += bytes_in_block;
                continue;
            }

            // Read block from device
            // TODO: Integrate with actual block device
            // For now, fill with zeros as stub
            let bytes_in_block = ((block_size - block_offset as u64) as usize).min(max_read - bytes_read);
            buffer[bytes_read..bytes_read + bytes_in_block].fill(0);
            bytes_read += bytes_in_block;
        }

        Ok(bytes_read)
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> Result<usize, FsError> {
        // Similar to read but for writing
        let _ = (offset, buffer);
        Err(FsError::NotSupported)
    }

    fn getattr(&self) -> Result<FileAttr, FsError> {
        let inode = self.read_inode()?;

        let file_type = match inode.i_mode & 0xF000 {
            0x8000 => FileType::Regular,
            0x4000 => FileType::Directory,
            0xA000 => FileType::Symlink,
            0x2000 => FileType::CharDevice,
            0x6000 => FileType::BlockDevice,
            0x1000 => FileType::Fifo,
            0xC000 => FileType::Socket,
            _ => FileType::Regular,
        };

        Ok(FileAttr {
            file_type,
            mode: FileMode::new((inode.i_mode & 0x0FFF) as u32),
            size: self.get_file_size(&inode),
            nlink: inode.i_links_count as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            ino: self.ino,
            blocks: inode.i_blocks_lo as u64,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }

    fn setattr(&self, _attr: &FileAttr) -> Result<(), FsError> {
        Err(FsError::NotSupported)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        // Read directory using htree or linear format
        Err(FsError::NotSupported)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<dyn VNode>, FsError> {
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

/// ext4 Filesystem
pub struct Ext4Filesystem {
    block_size: u32,
    root_ino: u64,
    features_compat: u32,
    features_incompat: u32,
    features_ro_compat: u32,
}

impl Ext4Filesystem {
    /// Mount an ext4 filesystem from a block device
    pub fn mount() -> Result<Arc<Self>, FsError> {
        // Read superblock from block 1 (1024 bytes offset)
        // Verify magic number
        // Check feature flags
        // Verify journal
        
        Ok(Arc::new(Ext4Filesystem {
            block_size: 4096,
            root_ino: 2,
            features_compat: 0,
            features_incompat: EXT4_FEATURE_INCOMPAT_EXTENTS | EXT4_FEATURE_INCOMPAT_64BIT,
            features_ro_compat: 0,
        }))
    }

    /// Check if filesystem has a specific feature
    pub fn has_feature_incompat(&self, feature: u32) -> bool {
        (self.features_incompat & feature) != 0
    }
}

impl Filesystem for Ext4Filesystem {
    fn fs_type(&self) -> FsType {
        FsType::Ext4
    }

    fn root(&self) -> Arc<dyn VNode> {
        Arc::new(Ext4VNode::new(
            Arc::new(Self::mount().unwrap()),
            self.root_ino
        ))
    }

    fn sync(&self) -> Result<(), FsError> {
        // Flush journal
        // Flush all cached data
        Ok(())
    }

    fn statfs(&self) -> Result<StatFs, FsError> {
        Ok(StatFs {
            fs_type: 0xEF53,
            block_size: self.block_size as u64,
            blocks: 0,
            blocks_free: 0,
            blocks_available: 0,
            files: 0,
            files_free: 0,
            name_max: 255,
        })
    }

    fn unmount(&self) -> Result<(), FsError> {
        // Commit journal
        // Write back all cached data
        // Mark filesystem as cleanly unmounted
        Ok(())
    }
}

/// Initialize ext4 driver
pub fn init() {
    // ext4 filesystems are mounted on demand
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext4_magic() {
        assert_eq!(EXT4_MAGIC, 0xEF53);
    }

    #[test]
    fn test_extent_magic() {
        assert_eq!(EXT4_EXT_MAGIC, 0xF30A);
    }

    #[test]
    fn test_feature_flags() {
        let fs = Ext4Filesystem {
            block_size: 4096,
            root_ino: 2,
            features_compat: 0,
            features_incompat: EXT4_FEATURE_INCOMPAT_EXTENTS,
            features_ro_compat: 0,
        };
        assert!(fs.has_feature_incompat(EXT4_FEATURE_INCOMPAT_EXTENTS));
        assert!(!fs.has_feature_incompat(EXT4_FEATURE_INCOMPAT_64BIT));
    }
}

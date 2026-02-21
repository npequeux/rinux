//! ext2 Filesystem Support
//!
//! Read/write support for the second extended filesystem

use crate::{FsError, FsType};
use crate::vfs::{VNode, Filesystem, FileAttr, FileType, FileMode, DirEntry, StatFs};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use spin::RwLock;

/// ext2 Superblock (simplified)
#[repr(C, packed)]
struct Ext2Superblock {
    s_inodes_count: u32,      // Total number of inodes
    s_blocks_count: u32,      // Total number of blocks
    s_r_blocks_count: u32,    // Reserved blocks count
    s_free_blocks_count: u32, // Free blocks count
    s_free_inodes_count: u32, // Free inodes count
    s_first_data_block: u32,  // First data block
    s_log_block_size: u32,    // Block size = 1024 << s_log_block_size
    s_log_frag_size: i32,     // Fragment size
    s_blocks_per_group: u32,  // Blocks per group
    s_frags_per_group: u32,   // Fragments per group
    s_inodes_per_group: u32,  // Inodes per group
    s_mtime: u32,             // Mount time
    s_wtime: u32,             // Write time
    s_mnt_count: u16,         // Mount count
    s_max_mnt_count: u16,     // Max mount count
    s_magic: u16,             // Magic signature (0xEF53)
    s_state: u16,             // File system state
    s_errors: u16,            // Behavior when detecting errors
    s_minor_rev_level: u16,   // Minor revision level
    s_lastcheck: u32,         // Time of last check
    s_checkinterval: u32,     // Max time between checks
    s_creator_os: u32,        // Creator OS
    s_rev_level: u32,         // Revision level
    s_def_resuid: u16,        // Default uid for reserved blocks
    s_def_resgid: u16,        // Default gid for reserved blocks
}

/// ext2 magic number
const EXT2_MAGIC: u16 = 0xEF53;

/// ext2 Inode (simplified)
#[repr(C, packed)]
struct Ext2Inode {
    i_mode: u16,              // File mode
    i_uid: u16,               // Owner UID
    i_size: u32,              // Size in bytes
    i_atime: u32,             // Access time
    i_ctime: u32,             // Creation time
    i_mtime: u32,             // Modification time
    i_dtime: u32,             // Deletion time
    i_gid: u16,               // Group ID
    i_links_count: u16,       // Links count
    i_blocks: u32,            // Blocks count
    i_flags: u32,             // File flags
    i_osd1: u32,              // OS dependent
    i_block: [u32; 15],       // Pointers to blocks
    i_generation: u32,        // File version (for NFS)
    i_file_acl: u32,          // File ACL
    i_dir_acl: u32,           // Directory ACL
    i_faddr: u32,             // Fragment address
    i_osd2: [u8; 12],         // OS dependent
}

/// ext2 Directory Entry (simplified)
#[repr(C, packed)]
struct Ext2DirEntry {
    inode: u32,               // Inode number
    rec_len: u16,             // Record length
    name_len: u8,             // Name length
    file_type: u8,            // File type
    // name follows (variable length)
}

/// ext2 VNode
pub struct Ext2VNode {
    fs: Arc<Ext2Filesystem>,
    ino: u64,
}

impl Ext2VNode {
    fn new(fs: Arc<Ext2Filesystem>, ino: u64) -> Self {
        Ext2VNode { fs, ino }
    }

    fn read_inode(&self) -> Result<Ext2Inode, FsError> {
        // Calculate block group and inode table offset
        // Read inode from device
        // For now, this is a stub
        Err(FsError::NotFound)
    }
}

impl VNode for Ext2VNode {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, FsError> {
        // Read inode to get file size and block pointers
        let inode = self.read_inode()?;
        
        // Check if offset is beyond file size
        if offset >= inode.i_size as u64 {
            return Ok(0);
        }

        let block_size = self.fs.block_size as u64;
        let max_read = ((inode.i_size as u64 - offset).min(buffer.len() as u64)) as usize;
        let mut bytes_read = 0;

        while bytes_read < max_read {
            let current_offset = offset + bytes_read as u64;
            let block_index = current_offset / block_size;
            let block_offset = (current_offset % block_size) as usize;

            // Get physical block number
            let block_num = if block_index < 12 {
                // Direct blocks
                inode.i_block[block_index as usize]
            } else if block_index < 12 + 256 {
                // Single indirect blocks
                // Would need to read the indirect block first
                // For now, return error
                return Err(FsError::IoError);
            } else if block_index < 12 + 256 + 256 * 256 {
                // Double indirect blocks
                return Err(FsError::IoError);
            } else {
                // Triple indirect blocks
                return Err(FsError::IoError);
            };

            if block_num == 0 {
                // Sparse block (hole in file) - fill with zeros
                let bytes_in_block = ((block_size - block_offset as u64) as usize).min(max_read - bytes_read);
                buffer[bytes_read..bytes_read + bytes_in_block].fill(0);
                bytes_read += bytes_in_block;
                continue;
            }

            // Read block from device
            // In a real implementation:
            // let block_data = self.fs.read_block(block_num)?;
            // For now, just fill with zeros as a stub
            let bytes_in_block = ((block_size - block_offset as u64) as usize).min(max_read - bytes_read);
            buffer[bytes_read..bytes_read + bytes_in_block].fill(0);
            bytes_read += bytes_in_block;
        }

        Ok(bytes_read)
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> Result<usize, FsError> {
        // Similar to read but for writing
        let _ = (offset, buffer);
        Err(FsError::IoError)
    }

    fn getattr(&self) -> Result<FileAttr, FsError> {
        let inode = self.read_inode()?;

        // ext2 file types
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
            size: inode.i_size as u64,
            nlink: inode.i_links_count as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            ino: self.ino,
            blocks: inode.i_blocks as u64,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }

    fn setattr(&self, _attr: &FileAttr) -> Result<(), FsError> {
        // Write updated inode back to device
        Err(FsError::IoError)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        // Read directory inode
        let inode = self.read_inode()?;

        // Verify this is a directory
        if inode.i_mode & 0xF000 != 0x4000 {
            return Err(FsError::NotADirectory);
        }

        let mut entries = Vec::new();
        let mut offset = 0u64;

        // Read directory data blocks
        while offset < inode.i_size as u64 {
            let mut buf = [0u8; 512]; // Read directory entries in chunks
            let bytes_read = self.read(offset, &mut buf)?;

            if bytes_read == 0 {
                break;
            }

            // Parse directory entries from buffer
            let mut pos = 0;
            while pos < bytes_read {
                if pos + core::mem::size_of::<Ext2DirEntry>() > bytes_read {
                    break;
                }

                let dir_entry = unsafe {
                    core::ptr::read_unaligned(buf.as_ptr().add(pos) as *const Ext2DirEntry)
                };

                if dir_entry.inode == 0 {
                    // Skip deleted entries
                    pos += dir_entry.rec_len as usize;
                    continue;
                }

                // Extract name
                let name_start = pos + core::mem::size_of::<Ext2DirEntry>();
                let name_end = name_start + dir_entry.name_len as usize;

                if name_end <= bytes_read {
                    let name_bytes = &buf[name_start..name_end];
                    if let Ok(name) = alloc::string::String::from_utf8(name_bytes.to_vec()) {
                        let file_type = match dir_entry.file_type {
                            1 => FileType::Regular,
                            2 => FileType::Directory,
                            3 => FileType::CharDevice,
                            4 => FileType::BlockDevice,
                            5 => FileType::Fifo,
                            6 => FileType::Socket,
                            7 => FileType::Symlink,
                            _ => FileType::Regular,
                        };

                        entries.push(DirEntry {
                            ino: dir_entry.inode as u64,
                            name,
                            file_type,
                        });
                    }
                }

                pos += dir_entry.rec_len as usize;
            }

            offset += bytes_read as u64;
        }

        Ok(entries)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<dyn VNode>, FsError> {
        // Read directory entries
        // Find entry with matching name
        // Return VNode for that inode
        Err(FsError::NotFound)
    }

    fn create(&self, _name: &str, _mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        // Allocate new inode
        // Initialize inode
        // Add directory entry to parent
        Err(FsError::IoError)
    }

    fn mkdir(&self, _name: &str, _mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        // Similar to create but for directory
        Err(FsError::IoError)
    }

    fn unlink(&self, _name: &str) -> Result<(), FsError> {
        // Remove directory entry
        // Decrement inode link count
        // Free inode if link count reaches 0
        Err(FsError::IoError)
    }

    fn rmdir(&self, _name: &str) -> Result<(), FsError> {
        // Check if directory is empty
        // Remove directory entry
        // Free inode
        Err(FsError::IoError)
    }

    fn rename(&self, _old_name: &str, _new_parent: Arc<dyn VNode>, _new_name: &str) -> Result<(), FsError> {
        Err(FsError::IoError)
    }

    fn symlink(&self, _name: &str, _target: &str) -> Result<Arc<dyn VNode>, FsError> {
        Err(FsError::IoError)
    }

    fn readlink(&self) -> Result<String, FsError> {
        Err(FsError::IoError)
    }

    fn truncate(&self, _size: u64) -> Result<(), FsError> {
        Err(FsError::IoError)
    }

    fn fsync(&self) -> Result<(), FsError> {
        // Flush all pending writes to device
        Ok(())
    }
}

/// ext2 Filesystem
pub struct Ext2Filesystem {
    // Device to read/write from
    // device: Arc<dyn BlockDevice>,
    // Cached superblock
    // superblock: RwLock<Ext2Superblock>,
    // Block size
    block_size: u32,
    // Root inode number (typically 2)
    root_ino: u64,
}

impl Ext2Filesystem {
    /// Mount an ext2 filesystem from a block device
    ///
    /// # Arguments
    ///
    /// * `device` - Block device containing the ext2 filesystem
    pub fn mount(/*device: Arc<dyn BlockDevice>*/) -> Result<Arc<Self>, FsError> {
        // Read superblock from block 1 (1024 bytes offset)
        // Verify magic number
        // Read block group descriptor table
        // Cache important data structures
        
        // For now, return a stub
        Ok(Arc::new(Ext2Filesystem {
            block_size: 4096,
            root_ino: 2,
        }))
    }
}

impl Filesystem for Ext2Filesystem {
    fn fs_type(&self) -> FsType {
        FsType::Ext2
    }

    fn root(&self) -> Arc<dyn VNode> {
        Arc::new(Ext2VNode::new(Arc::new(Self::mount().unwrap()), self.root_ino))
    }

    fn sync(&self) -> Result<(), FsError> {
        // Flush all cached data to device
        Ok(())
    }

    fn statfs(&self) -> Result<StatFs, FsError> {
        // Read from superblock
        Ok(StatFs {
            fs_type: 0xEF53,
            block_size: self.block_size as u64,
            blocks: 0,      // From superblock
            blocks_free: 0, // From superblock
            blocks_available: 0, // From superblock
            files: 0,       // From superblock
            files_free: 0,  // From superblock
            name_max: 255,
        })
    }

    fn unmount(&self) -> Result<(), FsError> {
        // Write back all cached data
        // Mark filesystem as cleanly unmounted
        Ok(())
    }
}

/// Initialize ext2 driver
pub fn init() {
    // ext2 filesystems are mounted on demand
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext2_magic() {
        assert_eq!(EXT2_MAGIC, 0xEF53);
    }
}

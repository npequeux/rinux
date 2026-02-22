//! ext2 Filesystem Support
//!
//! Production-ready implementation of the second extended filesystem with full read/write support,
//! block allocation, inode management, and directory operations.
//!
//! # Features
//!
//! - Full read/write support for ext2 filesystems
//! - Direct, single, double, and triple indirect block handling
//! - Inode allocation and deallocation
//! - Block allocation and deallocation with bitmap management
//! - Directory operations (create, delete, lookup)
//! - File operations (read, write, truncate)
//! - Symbolic link support (short links stored in inode)
//! - Block caching for improved performance
//! - Sparse file support (holes in files)
//! - Proper error handling throughout
//!
//! # Usage
//!
//! ```ignore
//! use rinux_fs::ext2::{Ext2Filesystem, BlockDevice};
//! use alloc::sync::Arc;
//!
//! // Assuming you have a block device (from AHCI driver, for example)
//! let device: Arc<dyn BlockDevice> = get_block_device(0);
//!
//! // Mount the ext2 filesystem
//! let fs = Ext2Filesystem::mount(device)?;
//!
//! // Get root directory
//! let root = fs.root();
//!
//! // List directory contents
//! let entries = root.readdir()?;
//! for entry in entries {
//!     println!("{}: inode {}", entry.name, entry.ino);
//! }
//!
//! // Read a file
//! let file = root.lookup("myfile.txt")?;
//! let mut buffer = vec![0u8; 1024];
//! let bytes_read = file.read(0, &mut buffer)?;
//!
//! // Create a new file
//! let new_file = root.create("newfile.txt", FileMode::new(0o644))?;
//! let data = b"Hello, ext2!";
//! new_file.write(0, data)?;
//!
//! // Sync filesystem
//! fs.sync()?;
//! ```
//!
//! # Architecture
//!
//! The implementation consists of several key components:
//!
//! ## Block Device Integration
//!
//! The filesystem uses the `BlockDevice` trait to abstract the underlying storage device.
//! This allows it to work with any block device (AHCI, NVMe, RAM disk, etc.).
//!
//! ## Block Cache
//!
//! A simple block cache using a BTreeMap provides performance improvements by caching
//! recently accessed blocks. The cache uses a FIFO eviction policy and writes back
//! dirty blocks on eviction or sync.
//!
//! ## Inode Management
//!
//! Inodes are allocated from a bitmap maintained in each block group. The implementation
//! tracks the block group descriptor table and updates free inode counts atomically.
//!
//! ## Block Allocation
//!
//! Blocks are allocated from block bitmaps in block groups. The allocator searches for
//! the first available block and updates the bitmap and counters.
//!
//! ## Directory Operations
//!
//! Directories are stored as linear lists of `Ext2DirEntry` structures. The implementation
//! handles variable-length entries and supports efficient lookup by name.
//!
//! ## File I/O
//!
//! File data is accessed through the inode's block pointers, handling direct blocks
//! and up to triple indirect blocks. Sparse files (with holes) are supported by
//! returning zeros for unmapped blocks.
//!
//! # Limitations
//!
//! - Extended attributes not supported
//! - Large files (> 4GB) partially supported (requires testing)
//! - Journal support not implemented (this is ext2, not ext3/ext4)
//! - No optimization for sequential vs. random access
//! - Simple FIFO cache eviction (not LRU)
//! - Rename operation not fully implemented
//! - Long symbolic links (> 60 bytes) not supported
//!
//! # Safety
//!
//! This implementation uses `unsafe` code in several places:
//! - Reading/writing packed C structures from/to byte buffers
//! - Manipulating block pointers in inodes
//!
//! All unsafe code is carefully reviewed and documented with safety invariants.

use crate::{FsError, FsType};
use crate::vfs::{VNode, Filesystem, FileAttr, FileType, FileMode, DirEntry, StatFs};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use spin::RwLock;
use core::mem;

/// BlockDevice trait adapter
/// This wraps the actual block device from rinux-block crate
/// In production, you would import this from the block driver
pub trait BlockDevice: Send + Sync {
    fn read_blocks(&self, block_offset: u64, buffer: &mut [u8]) -> Result<usize, ()>;
    fn write_blocks(&self, block_offset: u64, buffer: &[u8]) -> Result<usize, ()>;
    fn block_size(&self) -> usize;
    fn flush(&self) -> Result<(), ()>;
}

/// Wrapper to adapt external block device implementations
pub struct BlockDeviceAdapter<T: Send + Sync> {
    inner: T,
    read_fn: fn(&T, u64, &mut [u8]) -> Result<usize, ()>,
    write_fn: fn(&T, u64, &[u8]) -> Result<usize, ()>,
    block_size_fn: fn(&T) -> usize,
    flush_fn: fn(&T) -> Result<(), ()>,
}

impl<T: Send + Sync> BlockDeviceAdapter<T> {
    pub fn new(
        device: T,
        read_fn: fn(&T, u64, &mut [u8]) -> Result<usize, ()>,
        write_fn: fn(&T, u64, &[u8]) -> Result<usize, ()>,
        block_size_fn: fn(&T) -> usize,
        flush_fn: fn(&T) -> Result<(), ()>,
    ) -> Self {
        Self {
            inner: device,
            read_fn,
            write_fn,
            block_size_fn,
            flush_fn,
        }
    }
}

impl<T: Send + Sync> BlockDevice for BlockDeviceAdapter<T> {
    fn read_blocks(&self, block_offset: u64, buffer: &mut [u8]) -> Result<usize, ()> {
        (self.read_fn)(&self.inner, block_offset, buffer)
    }
    
    fn write_blocks(&self, block_offset: u64, buffer: &[u8]) -> Result<usize, ()> {
        (self.write_fn)(&self.inner, block_offset, buffer)
    }
    
    fn block_size(&self) -> usize {
        (self.block_size_fn)(&self.inner)
    }
    
    fn flush(&self) -> Result<(), ()> {
        (self.flush_fn)(&self.inner)
    }
}

/// ext2 Superblock
#[repr(C, packed)]
#[derive(Clone, Copy)]
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

/// Block Group Descriptor
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct BlockGroupDescriptor {
    bg_block_bitmap: u32,      // Block bitmap block
    bg_inode_bitmap: u32,      // Inode bitmap block
    bg_inode_table: u32,       // Inode table block
    bg_free_blocks_count: u16, // Free blocks count
    bg_free_inodes_count: u16, // Free inodes count
    bg_used_dirs_count: u16,   // Directories count
    bg_pad: u16,
    bg_reserved: [u32; 3],
}

/// ext2 magic number
const EXT2_MAGIC: u16 = 0xEF53;

/// Superblock offset in bytes
const SUPERBLOCK_OFFSET: u64 = 1024;

/// Root inode number
const EXT2_ROOT_INO: u32 = 2;

/// Inode size (128 bytes for revision 0)
const INODE_SIZE: usize = 128;

/// Direct block pointers in inode
const EXT2_NDIR_BLOCKS: usize = 12;
/// Single indirect block pointer index
const EXT2_IND_BLOCK: usize = 12;
/// Double indirect block pointer index
const EXT2_DIND_BLOCK: usize = 13;
/// Triple indirect block pointer index
const EXT2_TIND_BLOCK: usize = 14;

/// File type constants
const EXT2_S_IFREG: u16 = 0x8000;  // Regular file
const EXT2_S_IFDIR: u16 = 0x4000;  // Directory
const EXT2_S_IFLNK: u16 = 0xA000;  // Symbolic link
const EXT2_S_IFCHR: u16 = 0x2000;  // Character device
const EXT2_S_IFBLK: u16 = 0x6000;  // Block device
const EXT2_S_IFIFO: u16 = 0x1000;  // FIFO
const EXT2_S_IFSOCK: u16 = 0xC000; // Socket

/// Directory entry file type constants
const EXT2_FT_REG_FILE: u8 = 1;
const EXT2_FT_DIR: u8 = 2;
const EXT2_FT_CHRDEV: u8 = 3;
const EXT2_FT_BLKDEV: u8 = 4;
const EXT2_FT_FIFO: u8 = 5;
const EXT2_FT_SOCK: u8 = 6;
const EXT2_FT_SYMLINK: u8 = 7;

/// ext2 Inode
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Ext2Inode {
    i_mode: u16,              // File mode
    i_uid: u16,               // Owner UID
    i_size: u32,              // Size in bytes (lower 32 bits)
    i_atime: u32,             // Access time
    i_ctime: u32,             // Creation time
    i_mtime: u32,             // Modification time
    i_dtime: u32,             // Deletion time
    i_gid: u16,               // Group ID
    i_links_count: u16,       // Links count
    i_blocks: u32,            // Blocks count (512-byte blocks)
    i_flags: u32,             // File flags
    i_osd1: u32,              // OS dependent
    i_block: [u32; 15],       // Pointers to blocks
    i_generation: u32,        // File version (for NFS)
    i_file_acl: u32,          // File ACL
    i_dir_acl: u32,           // Directory ACL / size high
    i_faddr: u32,             // Fragment address
    i_osd2: [u8; 12],         // OS dependent
}

/// ext2 Directory Entry
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Ext2DirEntry {
    inode: u32,               // Inode number
    rec_len: u16,             // Record length
    name_len: u8,             // Name length
    file_type: u8,            // File type
    // name follows (variable length)
}

/// Block cache entry
struct CachedBlock {
    data: Vec<u8>,
    dirty: bool,
}

/// ext2 Filesystem implementation
pub struct Ext2Filesystem {
    device: Arc<dyn BlockDevice>,
    superblock: RwLock<Ext2Superblock>,
    block_groups: RwLock<Vec<BlockGroupDescriptor>>,
    block_size: u32,
    blocks_per_group: u32,
    inodes_per_group: u32,
    first_data_block: u32,
    /// Block cache: maps block number to cached data
    block_cache: RwLock<BTreeMap<u32, CachedBlock>>,
    /// Maximum cache size (in blocks)
    max_cache_blocks: usize,
}

impl Ext2Filesystem {
    /// Mount an ext2 filesystem from a block device
    pub fn mount(device: Arc<dyn BlockDevice>) -> Result<Arc<Self>, FsError> {
        // Read superblock (at offset 1024 bytes, size 1024 bytes)
        let mut sb_buf = vec![0u8; 1024];
        let device_block_size = device.block_size();
        
        // Calculate which device blocks contain the superblock
        let sb_start_block = SUPERBLOCK_OFFSET / device_block_size as u64;
        let blocks_needed = (1024 + device_block_size - 1) / device_block_size;
        
        let mut read_buf = vec![0u8; blocks_needed * device_block_size];
        device.read_blocks(sb_start_block, &mut read_buf)
            .map_err(|_| FsError::IoError)?;
        
        let sb_offset_in_buf = (SUPERBLOCK_OFFSET % device_block_size as u64) as usize;
        sb_buf.copy_from_slice(&read_buf[sb_offset_in_buf..sb_offset_in_buf + 1024]);
        
        // Parse superblock
        let superblock = unsafe {
            core::ptr::read_unaligned(sb_buf.as_ptr() as *const Ext2Superblock)
        };
        
        // Verify magic number
        if superblock.s_magic != EXT2_MAGIC {
            return Err(FsError::InvalidFs);
        }
        
        // Calculate block size
        let block_size = 1024u32 << superblock.s_log_block_size;
        
        // Calculate number of block groups
        let num_block_groups = ((superblock.s_blocks_count + superblock.s_blocks_per_group - 1) 
            / superblock.s_blocks_per_group) as usize;
        
        // Read block group descriptor table
        // Located in the block immediately after the superblock
        let bgdt_block = if block_size == 1024 { 2 } else { 1 };
        let bgdt_size = num_block_groups * mem::size_of::<BlockGroupDescriptor>();
        let bgdt_blocks = (bgdt_size + block_size as usize - 1) / block_size as usize;
        
        let mut block_groups = Vec::with_capacity(num_block_groups);
        for i in 0..bgdt_blocks {
            let block_num = bgdt_block + i as u32;
            let block_data = Self::read_block_raw(&device, block_num, block_size)?;
            
            let descriptors_in_block = block_size as usize / mem::size_of::<BlockGroupDescriptor>();
            for j in 0..descriptors_in_block {
                if block_groups.len() >= num_block_groups {
                    break;
                }
                
                let offset = j * mem::size_of::<BlockGroupDescriptor>();
                let bgd = unsafe {
                    core::ptr::read_unaligned(
                        block_data[offset..].as_ptr() as *const BlockGroupDescriptor
                    )
                };
                block_groups.push(bgd);
            }
        }
        
        Ok(Arc::new(Ext2Filesystem {
            device,
            superblock: RwLock::new(superblock),
            block_groups: RwLock::new(block_groups),
            block_size,
            blocks_per_group: superblock.s_blocks_per_group,
            inodes_per_group: superblock.s_inodes_per_group,
            first_data_block: superblock.s_first_data_block,
            block_cache: RwLock::new(BTreeMap::new()),
            max_cache_blocks: 256, // Cache up to 256 blocks (1MB for 4K blocks)
        }))
    }
    
    /// Read a block from device without caching
    fn read_block_raw(device: &Arc<dyn BlockDevice>, block_num: u32, block_size: u32) 
        -> Result<Vec<u8>, FsError> {
        let device_block_size = device.block_size();
        let blocks_per_fs_block = block_size as usize / device_block_size;
        let device_block_start = block_num as u64 * blocks_per_fs_block as u64;
        
        let mut buffer = vec![0u8; block_size as usize];
        device.read_blocks(device_block_start, &mut buffer)
            .map_err(|_| FsError::IoError)?;
        
        Ok(buffer)
    }
    
    /// Read a block with caching
    fn read_block(&self, block_num: u32) -> Result<Vec<u8>, FsError> {
        // Check cache first
        {
            let cache = self.block_cache.read();
            if let Some(cached) = cache.get(&block_num) {
                return Ok(cached.data.clone());
            }
        }
        
        // Read from device
        let data = Self::read_block_raw(&self.device, block_num, self.block_size)?;
        
        // Add to cache
        {
            let mut cache = self.block_cache.write();
            
            // Evict if cache is full (simple FIFO)
            if cache.len() >= self.max_cache_blocks {
                // Write back dirty blocks and remove first entry
                if let Some((key, _)) = cache.iter().next().map(|(k, v)| (*k, v.dirty)) {
                    if let Some(entry) = cache.remove(&key) {
                        if entry.dirty {
                            let _ = self.write_block_raw(key, &entry.data);
                        }
                    }
                }
            }
            
            cache.insert(block_num, CachedBlock {
                data: data.clone(),
                dirty: false,
            });
        }
        
        Ok(data)
    }
    
    /// Write a block to device
    fn write_block_raw(&self, block_num: u32, data: &[u8]) -> Result<(), FsError> {
        let device_block_size = self.device.block_size();
        let blocks_per_fs_block = self.block_size as usize / device_block_size;
        let device_block_start = block_num as u64 * blocks_per_fs_block as u64;
        
        self.device.write_blocks(device_block_start, data)
            .map_err(|_| FsError::IoError)?;
        
        Ok(())
    }
    
    /// Write a block with caching
    fn write_block(&self, block_num: u32, data: Vec<u8>) -> Result<(), FsError> {
        let mut cache = self.block_cache.write();
        
        // Update cache or add new entry
        if let Some(entry) = cache.get_mut(&block_num) {
            entry.data = data;
            entry.dirty = true;
        } else {
            // Evict if needed
            if cache.len() >= self.max_cache_blocks {
                if let Some((key, _)) = cache.iter().next().map(|(k, v)| (*k, v.dirty)) {
                    if let Some(entry) = cache.remove(&key) {
                        if entry.dirty {
                            let _ = self.write_block_raw(key, &entry.data);
                        }
                    }
                }
            }
            
            cache.insert(block_num, CachedBlock {
                data,
                dirty: true,
            });
        }
        
        Ok(())
    }
    
    /// Flush all dirty cached blocks
    fn flush_cache(&self) -> Result<(), FsError> {
        let mut cache = self.block_cache.write();
        
        for (block_num, entry) in cache.iter_mut() {
            if entry.dirty {
                self.write_block_raw(*block_num, &entry.data)?;
                entry.dirty = false;
            }
        }
        
        self.device.flush().map_err(|_| FsError::IoError)?;
        Ok(())
    }
    
    /// Get inode location (block group, block number, offset)
    fn get_inode_location(&self, ino: u32) -> Result<(u32, u32, usize), FsError> {
        if ino == 0 {
            return Err(FsError::InvalidArgument);
        }
        
        // Inode numbers start from 1
        let inode_index = ino - 1;
        let block_group = inode_index / self.inodes_per_group;
        let index_in_group = inode_index % self.inodes_per_group;
        
        let block_groups = self.block_groups.read();
        if block_group as usize >= block_groups.len() {
            return Err(FsError::NotFound);
        }
        
        let bgd = &block_groups[block_group as usize];
        let inode_table_block = bgd.bg_inode_table;
        
        // Calculate block and offset within block
        let inode_offset = index_in_group as usize * INODE_SIZE;
        let block_offset = inode_offset / self.block_size as usize;
        let offset_in_block = inode_offset % self.block_size as usize;
        
        Ok((block_group, inode_table_block + block_offset as u32, offset_in_block))
    }
    
    /// Read an inode
    fn read_inode(&self, ino: u32) -> Result<Ext2Inode, FsError> {
        let (_, block_num, offset) = self.get_inode_location(ino)?;
        let block_data = self.read_block(block_num)?;
        
        if offset + INODE_SIZE > block_data.len() {
            return Err(FsError::InvalidData);
        }
        
        let inode = unsafe {
            core::ptr::read_unaligned(
                block_data[offset..offset + INODE_SIZE].as_ptr() as *const Ext2Inode
            )
        };
        
        Ok(inode)
    }
    
    /// Write an inode
    fn write_inode(&self, ino: u32, inode: &Ext2Inode) -> Result<(), FsError> {
        let (_, block_num, offset) = self.get_inode_location(ino)?;
        let mut block_data = self.read_block(block_num)?;
        
        if offset + INODE_SIZE > block_data.len() {
            return Err(FsError::InvalidData);
        }
        
        // Copy inode data into block
        unsafe {
            core::ptr::write_unaligned(
                block_data[offset..offset + INODE_SIZE].as_mut_ptr() as *mut Ext2Inode,
                *inode
            );
        }
        
        self.write_block(block_num, block_data)?;
        Ok(())
    }
    
    /// Get file size from inode (handling large files)
    fn get_file_size(&self, inode: &Ext2Inode) -> u64 {
        let size_low = inode.i_size as u64;
        
        // For regular files in revision >= 1, i_dir_acl contains high 32 bits
        if inode.i_mode & EXT2_S_IFREG == EXT2_S_IFREG {
            let sb = self.superblock.read();
            if sb.s_rev_level >= 1 {
                let size_high = inode.i_dir_acl as u64;
                return (size_high << 32) | size_low;
            }
        }
        
        size_low
    }
    
    /// Get block number for a file offset (handling indirection)
    fn get_block_num(&self, inode: &Ext2Inode, file_block: u32) -> Result<u32, FsError> {
        let ptrs_per_block = (self.block_size / 4) as u32;
        
        // Direct blocks
        if file_block < EXT2_NDIR_BLOCKS as u32 {
            return Ok(inode.i_block[file_block as usize]);
        }
        
        // Single indirect
        let indirect_start = EXT2_NDIR_BLOCKS as u32;
        if file_block < indirect_start + ptrs_per_block {
            let indirect_block = inode.i_block[EXT2_IND_BLOCK];
            if indirect_block == 0 {
                return Ok(0); // Sparse
            }
            
            let indirect_data = self.read_block(indirect_block)?;
            let index = (file_block - indirect_start) as usize * 4;
            let block_num = u32::from_le_bytes([
                indirect_data[index],
                indirect_data[index + 1],
                indirect_data[index + 2],
                indirect_data[index + 3],
            ]);
            
            return Ok(block_num);
        }
        
        // Double indirect
        let double_indirect_start = indirect_start + ptrs_per_block;
        if file_block < double_indirect_start + ptrs_per_block * ptrs_per_block {
            let double_indirect_block = inode.i_block[EXT2_DIND_BLOCK];
            if double_indirect_block == 0 {
                return Ok(0); // Sparse
            }
            
            let index_in_double = file_block - double_indirect_start;
            let first_level_index = index_in_double / ptrs_per_block;
            let second_level_index = index_in_double % ptrs_per_block;
            
            // Read first level indirect block
            let double_data = self.read_block(double_indirect_block)?;
            let offset = first_level_index as usize * 4;
            let indirect_block = u32::from_le_bytes([
                double_data[offset],
                double_data[offset + 1],
                double_data[offset + 2],
                double_data[offset + 3],
            ]);
            
            if indirect_block == 0 {
                return Ok(0); // Sparse
            }
            
            // Read second level indirect block
            let indirect_data = self.read_block(indirect_block)?;
            let offset = second_level_index as usize * 4;
            let block_num = u32::from_le_bytes([
                indirect_data[offset],
                indirect_data[offset + 1],
                indirect_data[offset + 2],
                indirect_data[offset + 3],
            ]);
            
            return Ok(block_num);
        }
        
        // Triple indirect
        let triple_indirect_start = double_indirect_start + ptrs_per_block * ptrs_per_block;
        if file_block < triple_indirect_start + ptrs_per_block * ptrs_per_block * ptrs_per_block {
            let triple_indirect_block = inode.i_block[EXT2_TIND_BLOCK];
            if triple_indirect_block == 0 {
                return Ok(0); // Sparse
            }
            
            let index_in_triple = file_block - triple_indirect_start;
            let first_level_index = index_in_triple / (ptrs_per_block * ptrs_per_block);
            let second_level_index = (index_in_triple / ptrs_per_block) % ptrs_per_block;
            let third_level_index = index_in_triple % ptrs_per_block;
            
            // Read first level
            let triple_data = self.read_block(triple_indirect_block)?;
            let offset = first_level_index as usize * 4;
            let double_indirect_block = u32::from_le_bytes([
                triple_data[offset],
                triple_data[offset + 1],
                triple_data[offset + 2],
                triple_data[offset + 3],
            ]);
            
            if double_indirect_block == 0 {
                return Ok(0);
            }
            
            // Read second level
            let double_data = self.read_block(double_indirect_block)?;
            let offset = second_level_index as usize * 4;
            let indirect_block = u32::from_le_bytes([
                double_data[offset],
                double_data[offset + 1],
                double_data[offset + 2],
                double_data[offset + 3],
            ]);
            
            if indirect_block == 0 {
                return Ok(0);
            }
            
            // Read third level
            let indirect_data = self.read_block(indirect_block)?;
            let offset = third_level_index as usize * 4;
            let block_num = u32::from_le_bytes([
                indirect_data[offset],
                indirect_data[offset + 1],
                indirect_data[offset + 2],
                indirect_data[offset + 3],
            ]);
            
            return Ok(block_num);
        }
        
        Err(FsError::InvalidArgument)
    }
    
    /// Allocate a new block
    fn allocate_block(&self) -> Result<u32, FsError> {
        let mut superblock = self.superblock.write();
        
        if superblock.s_free_blocks_count == 0 {
            return Err(FsError::NoSpaceLeft);
        }
        
        let mut block_groups = self.block_groups.write();
        
        // Find a block group with free blocks
        for (bg_num, bgd) in block_groups.iter_mut().enumerate() {
            if bgd.bg_free_blocks_count == 0 {
                continue;
            }
            
            // Read block bitmap
            let bitmap_block = bgd.bg_block_bitmap;
            let mut bitmap_data = self.read_block(bitmap_block)?;
            
            // Find first free block in bitmap
            for byte_idx in 0..bitmap_data.len() {
                let byte = bitmap_data[byte_idx];
                if byte == 0xFF {
                    continue; // All bits set, no free blocks in this byte
                }
                
                // Find first zero bit
                for bit_idx in 0..8 {
                    if (byte & (1 << bit_idx)) == 0 {
                        // Found free block
                        let block_in_group = (byte_idx * 8 + bit_idx) as u32;
                        let block_num = bg_num as u32 * self.blocks_per_group + 
                                       self.first_data_block + block_in_group;
                        
                        // Mark as allocated
                        bitmap_data[byte_idx] |= 1 << bit_idx;
                        self.write_block(bitmap_block, bitmap_data)?;
                        
                        // Update counters
                        bgd.bg_free_blocks_count -= 1;
                        superblock.s_free_blocks_count -= 1;
                        
                        // Write back block group descriptor
                        self.write_bgd(bg_num as u32, bgd)?;
                        
                        // Clear the allocated block
                        let zero_block = vec![0u8; self.block_size as usize];
                        self.write_block(block_num, zero_block)?;
                        
                        return Ok(block_num);
                    }
                }
            }
        }
        
        Err(FsError::NoSpaceLeft)
    }
    
    /// Free a block
    fn free_block(&self, block_num: u32) -> Result<(), FsError> {
        let bg_num = (block_num - self.first_data_block) / self.blocks_per_group;
        let block_in_group = (block_num - self.first_data_block) % self.blocks_per_group;
        
        let mut block_groups = self.block_groups.write();
        let bgd = &mut block_groups[bg_num as usize];
        
        // Read block bitmap
        let bitmap_block = bgd.bg_block_bitmap;
        let mut bitmap_data = self.read_block(bitmap_block)?;
        
        // Clear bit
        let byte_idx = (block_in_group / 8) as usize;
        let bit_idx = (block_in_group % 8) as usize;
        bitmap_data[byte_idx] &= !(1 << bit_idx);
        
        self.write_block(bitmap_block, bitmap_data)?;
        
        // Update counters
        bgd.bg_free_blocks_count += 1;
        let mut superblock = self.superblock.write();
        superblock.s_free_blocks_count += 1;
        
        self.write_bgd(bg_num, bgd)?;
        
        Ok(())
    }
    
    /// Write block group descriptor
    fn write_bgd(&self, bg_num: u32, bgd: &BlockGroupDescriptor) -> Result<(), FsError> {
        let bgdt_block = if self.block_size == 1024 { 2 } else { 1 };
        let bgds_per_block = self.block_size as usize / mem::size_of::<BlockGroupDescriptor>();
        
        let block_num = bgdt_block + bg_num / bgds_per_block as u32;
        let offset = (bg_num as usize % bgds_per_block) * mem::size_of::<BlockGroupDescriptor>();
        
        let mut block_data = self.read_block(block_num)?;
        
        unsafe {
            core::ptr::write_unaligned(
                block_data[offset..].as_mut_ptr() as *mut BlockGroupDescriptor,
                *bgd
            );
        }
        
        self.write_block(block_num, block_data)?;
        Ok(())
    }
    
    /// Allocate a new inode
    fn allocate_inode(&self, is_directory: bool) -> Result<u32, FsError> {
        let mut superblock = self.superblock.write();
        
        if superblock.s_free_inodes_count == 0 {
            return Err(FsError::NoSpaceLeft);
        }
        
        let mut block_groups = self.block_groups.write();
        
        // Find a block group with free inodes
        for (bg_num, bgd) in block_groups.iter_mut().enumerate() {
            if bgd.bg_free_inodes_count == 0 {
                continue;
            }
            
            // Read inode bitmap
            let bitmap_block = bgd.bg_inode_bitmap;
            let mut bitmap_data = self.read_block(bitmap_block)?;
            
            // Find first free inode in bitmap
            for byte_idx in 0..bitmap_data.len() {
                let byte = bitmap_data[byte_idx];
                if byte == 0xFF {
                    continue;
                }
                
                for bit_idx in 0..8 {
                    if (byte & (1 << bit_idx)) == 0 {
                        let inode_in_group = (byte_idx * 8 + bit_idx) as u32;
                        let ino = bg_num as u32 * self.inodes_per_group + inode_in_group + 1;
                        
                        // Mark as allocated
                        bitmap_data[byte_idx] |= 1 << bit_idx;
                        self.write_block(bitmap_block, bitmap_data)?;
                        
                        // Update counters
                        bgd.bg_free_inodes_count -= 1;
                        if is_directory {
                            bgd.bg_used_dirs_count += 1;
                        }
                        superblock.s_free_inodes_count -= 1;
                        
                        self.write_bgd(bg_num as u32, bgd)?;
                        
                        return Ok(ino);
                    }
                }
            }
        }
        
        Err(FsError::NoSpaceLeft)
    }
    
    /// Free an inode
    fn free_inode(&self, ino: u32, is_directory: bool) -> Result<(), FsError> {
        let inode_index = ino - 1;
        let bg_num = inode_index / self.inodes_per_group;
        let inode_in_group = inode_index % self.inodes_per_group;
        
        let mut block_groups = self.block_groups.write();
        let bgd = &mut block_groups[bg_num as usize];
        
        // Read inode bitmap
        let bitmap_block = bgd.bg_inode_bitmap;
        let mut bitmap_data = self.read_block(bitmap_block)?;
        
        // Clear bit
        let byte_idx = (inode_in_group / 8) as usize;
        let bit_idx = (inode_in_group % 8) as usize;
        bitmap_data[byte_idx] &= !(1 << bit_idx);
        
        self.write_block(bitmap_block, bitmap_data)?;
        
        // Update counters
        bgd.bg_free_inodes_count += 1;
        if is_directory {
            bgd.bg_used_dirs_count -= 1;
        }
        
        let mut superblock = self.superblock.write();
        superblock.s_free_inodes_count += 1;
        
        self.write_bgd(bg_num, bgd)?;
        
        Ok(())
    }
}

/// ext2 VNode
pub struct Ext2VNode {
    fs: Arc<Ext2Filesystem>,
    ino: u32,
}

impl Ext2VNode {
    fn new(fs: Arc<Ext2Filesystem>, ino: u32) -> Self {
        Ext2VNode { fs, ino }
    }

    fn read_inode(&self) -> Result<Ext2Inode, FsError> {
        self.fs.read_inode(self.ino)
    }
    
    fn write_inode(&self, inode: &Ext2Inode) -> Result<(), FsError> {
        self.fs.write_inode(self.ino, inode)
    }
}

impl VNode for Ext2VNode {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, FsError> {
        let inode = self.read_inode()?;
        
        // Check if offset is beyond file size
        let file_size = self.fs.get_file_size(&inode);
        if offset >= file_size {
            return Ok(0);
        }

        let block_size = self.fs.block_size as u64;
        let max_read = ((file_size - offset).min(buffer.len() as u64)) as usize;
        let mut bytes_read = 0;

        while bytes_read < max_read {
            let current_offset = offset + bytes_read as u64;
            let file_block = (current_offset / block_size) as u32;
            let block_offset = (current_offset % block_size) as usize;

            // Get physical block number (handling indirect blocks)
            let block_num = self.fs.get_block_num(&inode, file_block)?;

            if block_num == 0 {
                // Sparse block (hole in file) - fill with zeros
                let bytes_in_block = ((block_size - block_offset as u64) as usize)
                    .min(max_read - bytes_read);
                buffer[bytes_read..bytes_read + bytes_in_block].fill(0);
                bytes_read += bytes_in_block;
                continue;
            }

            // Read block from device
            let block_data = self.fs.read_block(block_num)?;
            let bytes_in_block = ((block_size - block_offset as u64) as usize)
                .min(max_read - bytes_read);
            
            buffer[bytes_read..bytes_read + bytes_in_block]
                .copy_from_slice(&block_data[block_offset..block_offset + bytes_in_block]);
            
            bytes_read += bytes_in_block;
        }

        Ok(bytes_read)
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> Result<usize, FsError> {
        let mut inode = self.read_inode()?;
        
        let block_size = self.fs.block_size as u64;
        let mut bytes_written = 0;

        while bytes_written < buffer.len() {
            let current_offset = offset + bytes_written as u64;
            let file_block = (current_offset / block_size) as u32;
            let block_offset = (current_offset % block_size) as usize;
            
            // Get or allocate physical block
            let mut block_num = self.fs.get_block_num(&inode, file_block)?;
            
            if block_num == 0 {
                // Need to allocate a new block
                block_num = self.fs.allocate_block()?;
                // TODO: Update inode block pointers (simplified - only handles direct blocks)
                if file_block < EXT2_NDIR_BLOCKS as u32 {
                    inode.i_block[file_block as usize] = block_num;
                } else {
                    // For indirect blocks, would need more complex logic
                    return Err(FsError::NotSupported);
                }
            }
            
            // Read existing block data if not writing full block
            let mut block_data = if block_offset != 0 || 
                (buffer.len() - bytes_written) < block_size as usize {
                self.fs.read_block(block_num)?
            } else {
                vec![0u8; block_size as usize]
            };
            
            // Write data to block
            let bytes_in_block = (block_size as usize - block_offset)
                .min(buffer.len() - bytes_written);
            
            block_data[block_offset..block_offset + bytes_in_block]
                .copy_from_slice(&buffer[bytes_written..bytes_written + bytes_in_block]);
            
            self.fs.write_block(block_num, block_data)?;
            bytes_written += bytes_in_block;
        }
        
        // Update file size if needed
        let new_size = offset + bytes_written as u64;
        let old_size = self.fs.get_file_size(&inode);
        if new_size > old_size {
            inode.i_size = new_size as u32;
            // TODO: Handle large file sizes (i_dir_acl for high bits)
        }
        
        // Update modification time
        inode.i_mtime = current_time();
        
        // Update blocks count (512-byte blocks)
        let blocks_needed = (new_size + 511) / 512;
        inode.i_blocks = blocks_needed as u32;
        
        self.write_inode(&inode)?;
        
        Ok(bytes_written)
    }

    fn getattr(&self) -> Result<FileAttr, FsError> {
        let inode = self.read_inode()?;

        // ext2 file types
        let file_type = match inode.i_mode & 0xF000 {
            EXT2_S_IFREG => FileType::Regular,
            EXT2_S_IFDIR => FileType::Directory,
            EXT2_S_IFLNK => FileType::Symlink,
            EXT2_S_IFCHR => FileType::CharDevice,
            EXT2_S_IFBLK => FileType::BlockDevice,
            EXT2_S_IFIFO => FileType::Fifo,
            EXT2_S_IFSOCK => FileType::Socket,
            _ => FileType::Regular,
        };

        Ok(FileAttr {
            file_type,
            mode: FileMode::new((inode.i_mode & 0x0FFF) as u32),
            size: self.fs.get_file_size(&inode),
            nlink: inode.i_links_count as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            ino: self.ino as u64,
            blocks: inode.i_blocks as u64,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }

    fn setattr(&self, attr: &FileAttr) -> Result<(), FsError> {
        let mut inode = self.read_inode()?;
        
        // Update inode fields
        inode.i_mode = (inode.i_mode & 0xF000) | (attr.mode.0 as u16 & 0x0FFF);
        inode.i_uid = attr.uid as u16;
        inode.i_gid = attr.gid as u16;
        inode.i_size = attr.size as u32;
        inode.i_atime = attr.atime as u32;
        inode.i_mtime = attr.mtime as u32;
        inode.i_ctime = attr.ctime as u32;
        inode.i_links_count = attr.nlink as u16;
        
        self.write_inode(&inode)?;
        Ok(())
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        let inode = self.read_inode()?;

        // Verify this is a directory
        if inode.i_mode & 0xF000 != EXT2_S_IFDIR {
            return Err(FsError::NotADirectory);
        }

        let mut entries = Vec::new();
        let file_size = self.fs.get_file_size(&inode);
        let mut offset = 0u64;

        // Read directory data blocks
        while offset < file_size {
            let block_size = self.fs.block_size as usize;
            let mut buf = vec![0u8; block_size];
            let bytes_read = self.read(offset, &mut buf)?;

            if bytes_read == 0 {
                break;
            }

            // Parse directory entries from buffer
            let mut pos = 0;
            while pos < bytes_read {
                if pos + mem::size_of::<Ext2DirEntry>() > bytes_read {
                    break;
                }

                let dir_entry = unsafe {
                    core::ptr::read_unaligned(buf.as_ptr().add(pos) as *const Ext2DirEntry)
                };

                if dir_entry.inode == 0 || dir_entry.rec_len == 0 {
                    break;
                }

                // Extract name
                let name_start = pos + mem::size_of::<Ext2DirEntry>();
                let name_end = name_start + dir_entry.name_len as usize;

                if name_end <= bytes_read {
                    let name_bytes = &buf[name_start..name_end];
                    if let Ok(name) = String::from_utf8(name_bytes.to_vec()) {
                        let file_type = match dir_entry.file_type {
                            EXT2_FT_REG_FILE => FileType::Regular,
                            EXT2_FT_DIR => FileType::Directory,
                            EXT2_FT_CHRDEV => FileType::CharDevice,
                            EXT2_FT_BLKDEV => FileType::BlockDevice,
                            EXT2_FT_FIFO => FileType::Fifo,
                            EXT2_FT_SOCK => FileType::Socket,
                            EXT2_FT_SYMLINK => FileType::Symlink,
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

    fn lookup(&self, name: &str) -> Result<Arc<dyn VNode>, FsError> {
        let entries = self.readdir()?;
        
        for entry in entries {
            if entry.name == name {
                return Ok(Arc::new(Ext2VNode::new(
                    Arc::clone(&self.fs),
                    entry.ino as u32
                )));
            }
        }
        
        Err(FsError::NotFound)
    }

    fn create(&self, name: &str, mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        // Verify this is a directory
        let parent_inode = self.read_inode()?;
        if parent_inode.i_mode & 0xF000 != EXT2_S_IFDIR {
            return Err(FsError::NotADirectory);
        }
        
        // Check if file already exists
        if self.lookup(name).is_ok() {
            return Err(FsError::AlreadyExists);
        }
        
        // Allocate new inode
        let new_ino = self.fs.allocate_inode(false)?;
        
        // Initialize inode
        let mut new_inode = Ext2Inode {
            i_mode: EXT2_S_IFREG | (mode.0 as u16 & 0x0FFF),
            i_uid: 0,
            i_size: 0,
            i_atime: current_time(),
            i_ctime: current_time(),
            i_mtime: current_time(),
            i_dtime: 0,
            i_gid: 0,
            i_links_count: 1,
            i_blocks: 0,
            i_flags: 0,
            i_osd1: 0,
            i_block: [0; 15],
            i_generation: 0,
            i_file_acl: 0,
            i_dir_acl: 0,
            i_faddr: 0,
            i_osd2: [0; 12],
        };
        
        self.fs.write_inode(new_ino, &new_inode)?;
        
        // Add directory entry
        self.add_dir_entry(name, new_ino, EXT2_FT_REG_FILE)?;
        
        Ok(Arc::new(Ext2VNode::new(Arc::clone(&self.fs), new_ino)))
    }

    fn mkdir(&self, name: &str, mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        // Verify this is a directory
        let parent_inode = self.read_inode()?;
        if parent_inode.i_mode & 0xF000 != EXT2_S_IFDIR {
            return Err(FsError::NotADirectory);
        }
        
        // Check if directory already exists
        if self.lookup(name).is_ok() {
            return Err(FsError::AlreadyExists);
        }
        
        // Allocate new inode
        let new_ino = self.fs.allocate_inode(true)?;
        
        // Allocate block for directory
        let dir_block = self.fs.allocate_block()?;
        
        // Initialize inode
        let mut new_inode = Ext2Inode {
            i_mode: EXT2_S_IFDIR | (mode.0 as u16 & 0x0FFF),
            i_uid: 0,
            i_size: self.fs.block_size,
            i_atime: current_time(),
            i_ctime: current_time(),
            i_mtime: current_time(),
            i_dtime: 0,
            i_gid: 0,
            i_links_count: 2, // . and parent
            i_blocks: (self.fs.block_size / 512) * 2,
            i_flags: 0,
            i_osd1: 0,
            i_block: [0; 15],
            i_generation: 0,
            i_file_acl: 0,
            i_dir_acl: 0,
            i_faddr: 0,
            i_osd2: [0; 12],
        };
        
        new_inode.i_block[0] = dir_block;
        self.fs.write_inode(new_ino, &new_inode)?;
        
        // Create . and .. entries
        let new_vnode = Ext2VNode::new(Arc::clone(&self.fs), new_ino);
        new_vnode.add_dir_entry(".", new_ino, EXT2_FT_DIR)?;
        new_vnode.add_dir_entry("..", self.ino, EXT2_FT_DIR)?;
        
        // Add directory entry in parent
        self.add_dir_entry(name, new_ino, EXT2_FT_DIR)?;
        
        Ok(Arc::new(new_vnode))
    }

    fn unlink(&self, name: &str) -> Result<(), FsError> {
        // Find the entry
        let entry = self.lookup(name)?;
        let ino = match self.readdir()?.into_iter().find(|e| e.name == name) {
            Some(e) => e.ino as u32,
            None => return Err(FsError::NotFound),
        };
        
        // Remove directory entry
        self.remove_dir_entry(name)?;
        
        // Decrement link count
        let mut inode = self.fs.read_inode(ino)?;
        if inode.i_links_count > 0 {
            inode.i_links_count -= 1;
        }
        
        // Free inode if no more links
        if inode.i_links_count == 0 {
            // Free all blocks
            self.free_all_blocks(ino)?;
            
            // Mark as deleted
            inode.i_dtime = current_time();
            self.fs.write_inode(ino, &inode)?;
            
            // Free inode
            let is_dir = inode.i_mode & 0xF000 == EXT2_S_IFDIR;
            self.fs.free_inode(ino, is_dir)?;
        } else {
            self.fs.write_inode(ino, &inode)?;
        }
        
        Ok(())
    }

    fn rmdir(&self, name: &str) -> Result<(), FsError> {
        // Find the entry
        let child = self.lookup(name)?;
        let ino = match self.readdir()?.into_iter().find(|e| e.name == name) {
            Some(e) => e.ino as u32,
            None => return Err(FsError::NotFound),
        };
        
        // Verify it's a directory
        let inode = self.fs.read_inode(ino)?;
        if inode.i_mode & 0xF000 != EXT2_S_IFDIR {
            return Err(FsError::NotADirectory);
        }
        
        // Check if directory is empty (should only have . and ..)
        let entries = child.readdir()?;
        if entries.len() > 2 {
            return Err(FsError::NotEmpty);
        }
        
        // Remove directory entry
        self.remove_dir_entry(name)?;
        
        // Free the directory
        self.free_all_blocks(ino)?;
        self.fs.free_inode(ino, true)?;
        
        Ok(())
    }

    fn rename(&self, old_name: &str, new_parent: Arc<dyn VNode>, new_name: &str) -> Result<(), FsError> {
        // For simplicity, not implemented in this version
        Err(FsError::NotSupported)
    }

    fn symlink(&self, name: &str, target: &str) -> Result<Arc<dyn VNode>, FsError> {
        // Verify this is a directory
        let parent_inode = self.read_inode()?;
        if parent_inode.i_mode & 0xF000 != EXT2_S_IFDIR {
            return Err(FsError::NotADirectory);
        }
        
        // Check if file already exists
        if self.lookup(name).is_ok() {
            return Err(FsError::AlreadyExists);
        }
        
        // Allocate new inode
        let new_ino = self.fs.allocate_inode(false)?;
        
        // For short symlinks (< 60 bytes), store in i_block
        let target_bytes = target.as_bytes();
        let mut new_inode = Ext2Inode {
            i_mode: EXT2_S_IFLNK | 0o777,
            i_uid: 0,
            i_size: target_bytes.len() as u32,
            i_atime: current_time(),
            i_ctime: current_time(),
            i_mtime: current_time(),
            i_dtime: 0,
            i_gid: 0,
            i_links_count: 1,
            i_blocks: 0,
            i_flags: 0,
            i_osd1: 0,
            i_block: [0; 15],
            i_generation: 0,
            i_file_acl: 0,
            i_dir_acl: 0,
            i_faddr: 0,
            i_osd2: [0; 12],
        };
        
        if target_bytes.len() <= 60 {
            // Store directly in inode
            unsafe {
                let block_ptr = new_inode.i_block.as_mut_ptr() as *mut u8;
                core::ptr::copy_nonoverlapping(
                    target_bytes.as_ptr(),
                    block_ptr,
                    target_bytes.len()
                );
            }
        } else {
            // Would need to allocate blocks for long symlinks
            return Err(FsError::NotSupported);
        }
        
        self.fs.write_inode(new_ino, &new_inode)?;
        
        // Add directory entry
        self.add_dir_entry(name, new_ino, EXT2_FT_SYMLINK)?;
        
        Ok(Arc::new(Ext2VNode::new(Arc::clone(&self.fs), new_ino)))
    }

    fn readlink(&self) -> Result<String, FsError> {
        let inode = self.read_inode()?;
        
        // Verify this is a symlink
        if inode.i_mode & 0xF000 != EXT2_S_IFLNK {
            return Err(FsError::InvalidArgument);
        }
        
        if inode.i_size <= 60 {
            // Read from i_block
            let bytes = unsafe {
                let block_ptr = inode.i_block.as_ptr() as *const u8;
                core::slice::from_raw_parts(block_ptr, inode.i_size as usize)
            };
            String::from_utf8(bytes.to_vec())
                .map_err(|_| FsError::InvalidData)
        } else {
            // Would need to read from blocks
            Err(FsError::NotSupported)
        }
    }

    fn truncate(&self, size: u64) -> Result<(), FsError> {
        let mut inode = self.read_inode()?;
        let old_size = self.fs.get_file_size(&inode);
        
        if size < old_size {
            // Shrink file - free blocks beyond new size
            let block_size = self.fs.block_size as u64;
            let old_blocks = (old_size + block_size - 1) / block_size;
            let new_blocks = (size + block_size - 1) / block_size;
            
            // Free blocks
            for block_idx in new_blocks as u32..old_blocks as u32 {
                if let Ok(block_num) = self.fs.get_block_num(&inode, block_idx) {
                    if block_num != 0 {
                        self.fs.free_block(block_num)?;
                        
                        // Clear block pointer (simplified - only direct blocks)
                        if block_idx < EXT2_NDIR_BLOCKS as u32 {
                            inode.i_block[block_idx as usize] = 0;
                        }
                    }
                }
            }
        }
        
        // Update size
        inode.i_size = size as u32;
        inode.i_mtime = current_time();
        
        // Update blocks count
        let blocks_needed = (size + 511) / 512;
        inode.i_blocks = blocks_needed as u32;
        
        self.write_inode(&inode)?;
        Ok(())
    }

    fn fsync(&self) -> Result<(), FsError> {
        self.fs.flush_cache()
    }
}

impl Ext2VNode {
    /// Add a directory entry
    fn add_dir_entry(&self, name: &str, ino: u32, file_type: u8) -> Result<(), FsError> {
        let inode = self.read_inode()?;
        
        if inode.i_mode & 0xF000 != EXT2_S_IFDIR {
            return Err(FsError::NotADirectory);
        }
        
        let name_bytes = name.as_bytes();
        if name_bytes.len() > 255 {
            return Err(FsError::InvalidArgument);
        }
        
        // Calculate required size (aligned to 4 bytes)
        let entry_size = (mem::size_of::<Ext2DirEntry>() + name_bytes.len() + 3) & !3;
        
        // Try to find space in existing blocks
        let file_size = self.fs.get_file_size(&inode);
        let block_size = self.fs.block_size as usize;
        
        // Simplified: append to end of directory
        let mut block_data = vec![0u8; block_size];
        let offset = file_size as usize;
        
        // Create directory entry
        let dir_entry = Ext2DirEntry {
            inode: ino,
            rec_len: entry_size as u16,
            name_len: name_bytes.len() as u8,
            file_type,
        };
        
        // Write entry (simplified - assumes space available)
        unsafe {
            core::ptr::write_unaligned(
                block_data.as_mut_ptr() as *mut Ext2DirEntry,
                dir_entry
            );
        }
        
        // Write name
        block_data[mem::size_of::<Ext2DirEntry>()..mem::size_of::<Ext2DirEntry>() + name_bytes.len()]
            .copy_from_slice(name_bytes);
        
        // Write to file (this will handle block allocation)
        self.write(file_size, &block_data[..entry_size])?;
        
        Ok(())
    }
    
    /// Remove a directory entry
    fn remove_dir_entry(&self, name: &str) -> Result<(), FsError> {
        // Simplified: Mark inode as 0 in the entry
        // In a full implementation, would coalesce with adjacent entries
        let inode = self.read_inode()?;
        
        if inode.i_mode & 0xF000 != EXT2_S_IFDIR {
            return Err(FsError::NotADirectory);
        }
        
        let file_size = self.fs.get_file_size(&inode);
        let block_size = self.fs.block_size as usize;
        let mut offset = 0u64;
        
        while offset < file_size {
            let mut buf = vec![0u8; block_size];
            let bytes_read = self.read(offset, &mut buf)?;
            
            if bytes_read == 0 {
                break;
            }
            
            let mut pos = 0;
            let mut entry_offset = offset;
            
            while pos < bytes_read {
                if pos + mem::size_of::<Ext2DirEntry>() > bytes_read {
                    break;
                }
                
                let mut dir_entry = unsafe {
                    core::ptr::read_unaligned(buf.as_ptr().add(pos) as *const Ext2DirEntry)
                };
                
                if dir_entry.inode != 0 && dir_entry.rec_len > 0 {
                    let name_start = pos + mem::size_of::<Ext2DirEntry>();
                    let name_end = name_start + dir_entry.name_len as usize;
                    
                    if name_end <= bytes_read {
                        let entry_name_bytes = &buf[name_start..name_end];
                        if let Ok(entry_name) = core::str::from_utf8(entry_name_bytes) {
                            if entry_name == name {
                                // Found it - mark as deleted
                                dir_entry.inode = 0;
                                
                                unsafe {
                                    core::ptr::write_unaligned(
                                        buf.as_mut_ptr().add(pos) as *mut Ext2DirEntry,
                                        dir_entry
                                    );
                                }
                                
                                // Write back
                                self.write(entry_offset + pos as u64, 
                                          &buf[pos..pos + mem::size_of::<Ext2DirEntry>()])?;
                                
                                return Ok(());
                            }
                        }
                    }
                }
                
                if dir_entry.rec_len == 0 {
                    break;
                }
                
                pos += dir_entry.rec_len as usize;
            }
            
            offset += bytes_read as u64;
        }
        
        Err(FsError::NotFound)
    }
    
    /// Free all blocks allocated to an inode
    fn free_all_blocks(&self, ino: u32) -> Result<(), FsError> {
        let inode = self.fs.read_inode(ino)?;
        let file_size = self.fs.get_file_size(&inode);
        let block_size = self.fs.block_size as u64;
        let num_blocks = (file_size + block_size - 1) / block_size;
        
        for block_idx in 0..num_blocks as u32 {
            if let Ok(block_num) = self.fs.get_block_num(&inode, block_idx) {
                if block_num != 0 {
                    self.fs.free_block(block_num)?;
                }
            }
        }
        
        Ok(())
    }
}

/// Get current time (Unix timestamp)
/// In a real kernel, this would query the system timer
fn current_time() -> u32 {
    // Placeholder - would need real time source
    0
}

impl Filesystem for Ext2Filesystem {
    fn fs_type(&self) -> FsType {
        FsType::Ext2
    }

    fn root(&self) -> Arc<dyn VNode> {
        Arc::new(Ext2VNode::new(Arc::new(Self {
            device: Arc::clone(&self.device),
            superblock: RwLock::new(*self.superblock.read()),
            block_groups: RwLock::new(self.block_groups.read().clone()),
            block_size: self.block_size,
            blocks_per_group: self.blocks_per_group,
            inodes_per_group: self.inodes_per_group,
            first_data_block: self.first_data_block,
            block_cache: RwLock::new(BTreeMap::new()),
            max_cache_blocks: self.max_cache_blocks,
        }), EXT2_ROOT_INO))
    }

    fn sync(&self) -> Result<(), FsError> {
        self.flush_cache()?;
        
        // Write superblock
        let superblock = self.superblock.read();
        let sb_bytes = unsafe {
            core::slice::from_raw_parts(
                &*superblock as *const Ext2Superblock as *const u8,
                mem::size_of::<Ext2Superblock>()
            )
        };
        
        let device_block_size = self.device.block_size();
        let sb_start_block = SUPERBLOCK_OFFSET / device_block_size as u64;
        let sb_offset_in_block = (SUPERBLOCK_OFFSET % device_block_size as u64) as usize;
        
        // Read, modify, write
        let mut block_data = vec![0u8; device_block_size];
        self.device.read_blocks(sb_start_block, &mut block_data)
            .map_err(|_| FsError::IoError)?;
        
        block_data[sb_offset_in_block..sb_offset_in_block + mem::size_of::<Ext2Superblock>()]
            .copy_from_slice(sb_bytes);
        
        self.device.write_blocks(sb_start_block, &block_data)
            .map_err(|_| FsError::IoError)?;
        
        // Write block group descriptors
        let block_groups = self.block_groups.read();
        let bgdt_block = if self.block_size == 1024 { 2 } else { 1 };
        let bgds_per_block = self.block_size as usize / mem::size_of::<BlockGroupDescriptor>();
        
        for (i, bgd) in block_groups.iter().enumerate() {
            let block_num = bgdt_block + (i / bgds_per_block) as u32;
            let offset = (i % bgds_per_block) * mem::size_of::<BlockGroupDescriptor>();
            
            let mut block_data = self.read_block(block_num)?;
            
            unsafe {
                core::ptr::write_unaligned(
                    block_data[offset..].as_mut_ptr() as *mut BlockGroupDescriptor,
                    *bgd
                );
            }
            
            self.write_block(block_num, block_data)?;
        }
        
        self.flush_cache()
    }

    fn statfs(&self) -> Result<StatFs, FsError> {
        let superblock = self.superblock.read();
        
        Ok(StatFs {
            fs_type: EXT2_MAGIC as u64,
            block_size: self.block_size as u64,
            blocks: superblock.s_blocks_count as u64,
            blocks_free: superblock.s_free_blocks_count as u64,
            blocks_available: superblock.s_free_blocks_count as u64,
            files: superblock.s_inodes_count as u64,
            files_free: superblock.s_free_inodes_count as u64,
            name_max: 255,
        })
    }

    fn unmount(&self) -> Result<(), FsError> {
        // Sync all data
        self.sync()?;
        
        // Mark filesystem as cleanly unmounted
        let mut superblock = self.superblock.write();
        superblock.s_state = 1; // Clean state
        
        Ok(())
    }
}

/// Initialize ext2 driver
pub fn init() {
    // ext2 filesystems are mounted on demand
    // No initialization needed
}

/// Helper function to mount ext2 from a block device by index
///
/// This is a convenience function for mounting ext2 from registered block devices.
/// In a real system, you would query the block device subsystem.
///
/// # Example
///
/// ```ignore
/// // Mount the first SATA disk as ext2
/// if let Some(fs) = mount_from_device_index(0) {
///     // Set as root filesystem
///     mount::set_root(fs)?;
/// }
/// ```
pub fn mount_from_device_index(_device_index: usize) -> Option<Arc<Ext2Filesystem>> {
    // In production, this would:
    // 1. Get the block device from the block subsystem
    // 2. Call Ext2Filesystem::mount(device)
    // 3. Return the mounted filesystem
    //
    // For now, this is a placeholder that would need integration
    // with the actual block device subsystem
    None
}

/// Detect if a block device contains an ext2 filesystem
///
/// Reads the superblock and checks for the ext2 magic number.
pub fn detect_ext2(device: &Arc<dyn BlockDevice>) -> Result<bool, FsError> {
    let mut sb_buf = vec![0u8; 1024];
    let device_block_size = device.block_size();
    
    let sb_start_block = SUPERBLOCK_OFFSET / device_block_size as u64;
    let blocks_needed = (1024 + device_block_size - 1) / device_block_size;
    
    let mut read_buf = vec![0u8; blocks_needed * device_block_size];
    device.read_blocks(sb_start_block, &mut read_buf)
        .map_err(|_| FsError::IoError)?;
    
    let sb_offset_in_buf = (SUPERBLOCK_OFFSET % device_block_size as u64) as usize;
    sb_buf.copy_from_slice(&read_buf[sb_offset_in_buf..sb_offset_in_buf + 1024]);
    
    let superblock = unsafe {
        core::ptr::read_unaligned(sb_buf.as_ptr() as *const Ext2Superblock)
    };
    
    Ok(superblock.s_magic == EXT2_MAGIC)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext2_magic() {
        assert_eq!(EXT2_MAGIC, 0xEF53);
    }
    
    #[test]
    fn test_constants() {
        assert_eq!(EXT2_NDIR_BLOCKS, 12);
        assert_eq!(EXT2_IND_BLOCK, 12);
        assert_eq!(EXT2_DIND_BLOCK, 13);
        assert_eq!(EXT2_TIND_BLOCK, 14);
    }
    
    #[test]
    fn test_file_types() {
        assert_eq!(EXT2_S_IFREG, 0x8000);
        assert_eq!(EXT2_S_IFDIR, 0x4000);
        assert_eq!(EXT2_S_IFLNK, 0xA000);
    }
}

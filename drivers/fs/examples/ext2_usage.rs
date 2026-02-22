//! ext2 Filesystem Usage Examples
//!
//! This module demonstrates how to use the ext2 filesystem implementation.
//! These examples show common operations like mounting, reading, writing,
//! and managing files and directories.

#![allow(dead_code)]

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;

/// Example: Mount an ext2 filesystem
///
/// ```ignore
/// use rinux_fs::ext2::{Ext2Filesystem, BlockDevice};
/// 
/// let device: Arc<dyn BlockDevice> = get_ahci_device(0)?;
/// let fs = Ext2Filesystem::mount(device)?;
/// ```

/// Example: Read a file
///
/// ```ignore
/// let root = fs.root();
/// let file = root.lookup("config.txt")?;
/// 
/// let attr = file.getattr()?;
/// let mut buffer = vec![0u8; attr.size as usize];
/// let bytes_read = file.read(0, &mut buffer)?;
/// ```

/// Example: Write a file
///
/// ```ignore
/// let root = fs.root();
/// let mode = FileMode::new(0o644);
/// let file = root.create("output.txt", mode)?;
/// 
/// let data = b"Hello, ext2!";
/// file.write(0, data)?;
/// file.fsync()?;
/// ```

/// Example: Create directories
///
/// ```ignore
/// let root = fs.root();
/// let mode = FileMode::new(0o755);
/// let dir = root.mkdir("documents", mode)?;
/// dir.mkdir("projects", mode)?;
/// ```

/// Example: List directory
///
/// ```ignore
/// let entries = root.readdir()?;
/// for entry in entries {
///     println!("{}: inode {}", entry.name, entry.ino);
/// }
/// ```

/// Example: Symbolic links
///
/// ```ignore
/// // Create symlink
/// let link = root.symlink("readme", "README.txt")?;
/// 
/// // Read link target
/// let target = link.readlink()?;
/// ```

/// Example: File operations
///
/// ```ignore
/// // Delete file
/// root.unlink("oldfile.txt")?;
/// 
/// // Truncate file
/// file.truncate(1024)?;
/// 
/// // Get file stats
/// let attr = file.getattr()?;
/// println!("Size: {} bytes", attr.size);
/// ```

/// Example: Filesystem operations
///
/// ```ignore
/// // Get filesystem stats
/// let stats = fs.statfs()?;
/// println!("Free blocks: {}", stats.blocks_free);
/// 
/// // Sync to disk
/// fs.sync()?;
/// 
/// // Unmount
/// fs.unmount()?;
/// ```

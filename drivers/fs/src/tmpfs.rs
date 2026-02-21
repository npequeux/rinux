//! TmpFS - Temporary Filesystem (in-memory)
//!
//! A simple RAM-based filesystem

use crate::{FsError, FsType};
use crate::vfs::{VNode, Filesystem, FileAttr, FileType, FileMode, DirEntry, StatFs};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use spin::RwLock;

/// TmpFS inode
struct TmpFsInode {
    /// Inode number
    ino: u64,
    /// File type
    file_type: FileType,
    /// File mode
    mode: FileMode,
    /// File size
    size: u64,
    /// User ID
    uid: u32,
    /// Group ID
    gid: u32,
    /// Access time
    atime: u64,
    /// Modification time
    mtime: u64,
    /// Change time
    ctime: u64,
    /// Number of hard links
    nlink: u32,
    /// File data (for regular files)
    data: Vec<u8>,
    /// Directory entries (for directories)
    entries: BTreeMap<String, u64>,
    /// Parent inode (for directories)
    parent: Option<u64>,
    /// Symlink target (for symlinks)
    symlink_target: Option<String>,
}

impl TmpFsInode {
    fn new(ino: u64, file_type: FileType, mode: FileMode) -> Self {
        let now = 0; // TODO: Get current time
        TmpFsInode {
            ino,
            file_type,
            mode,
            size: 0,
            uid: 0,
            gid: 0,
            atime: now,
            mtime: now,
            ctime: now,
            nlink: 1,
            data: Vec::new(),
            entries: BTreeMap::new(),
            parent: None,
            symlink_target: None,
        }
    }
}

/// TmpFS VNode implementation
pub struct TmpFsVNode {
    fs: Arc<TmpFsFilesystem>,
    ino: u64,
}

impl TmpFsVNode {
    fn new(fs: Arc<TmpFsFilesystem>, ino: u64) -> Self {
        TmpFsVNode { fs, ino }
    }

    fn get_inode(&self) -> Result<alloc::sync::Arc<RwLock<TmpFsInode>>, FsError> {
        self.fs.inodes.read()
            .get(&self.ino)
            .cloned()
            .ok_or(FsError::NotFound)
    }
}

impl VNode for TmpFsVNode {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, FsError> {
        let inode = self.get_inode()?;
        let inode = inode.read();

        if inode.file_type != FileType::Regular {
            return Err(FsError::IsADirectory);
        }

        let offset = offset as usize;
        if offset >= inode.data.len() {
            return Ok(0);
        }

        let to_read = buffer.len().min(inode.data.len() - offset);
        buffer[..to_read].copy_from_slice(&inode.data[offset..offset + to_read]);

        Ok(to_read)
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> Result<usize, FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        if inode.file_type != FileType::Regular {
            return Err(FsError::IsADirectory);
        }

        let offset = offset as usize;
        let end = offset + buffer.len();

        // Extend data if necessary
        if end > inode.data.len() {
            inode.data.resize(end, 0);
            inode.size = end as u64;
        }

        inode.data[offset..end].copy_from_slice(buffer);
        inode.mtime = 0; // TODO: Update to current time

        Ok(buffer.len())
    }

    fn getattr(&self) -> Result<FileAttr, FsError> {
        let inode = self.get_inode()?;
        let inode = inode.read();

        Ok(FileAttr {
            file_type: inode.file_type,
            mode: inode.mode,
            size: inode.size,
            nlink: inode.nlink,
            uid: inode.uid,
            gid: inode.gid,
            ino: inode.ino,
            blocks: (inode.size + 511) / 512,
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: inode.ctime,
        })
    }

    fn setattr(&self, attr: &FileAttr) -> Result<(), FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        inode.mode = attr.mode;
        inode.uid = attr.uid;
        inode.gid = attr.gid;
        inode.ctime = 0; // TODO: Update to current time

        Ok(())
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        let inode = self.get_inode()?;
        let inode = inode.read();

        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }

        let mut entries = Vec::new();

        // Add . and ..
        entries.push(DirEntry {
            ino: inode.ino,
            file_type: FileType::Directory,
            name: String::from("."),
        });

        if let Some(parent_ino) = inode.parent {
            entries.push(DirEntry {
                ino: parent_ino,
                file_type: FileType::Directory,
                name: String::from(".."),
            });
        }

        // Add children
        for (name, child_ino) in &inode.entries {
            if let Some(child_inode) = self.fs.inodes.read().get(child_ino) {
                let child = child_inode.read();
                entries.push(DirEntry {
                    ino: *child_ino,
                    file_type: child.file_type,
                    name: name.clone(),
                });
            }
        }

        Ok(entries)
    }

    fn lookup(&self, name: &str) -> Result<Arc<dyn VNode>, FsError> {
        if name == "." {
            return Ok(Arc::new(TmpFsVNode::new(Arc::clone(&self.fs), self.ino)));
        }

        let inode = self.get_inode()?;
        let inode = inode.read();

        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }

        if name == ".." {
            let parent_ino = inode.parent.unwrap_or(self.ino);
            return Ok(Arc::new(TmpFsVNode::new(Arc::clone(&self.fs), parent_ino)));
        }

        let child_ino = inode.entries.get(name).ok_or(FsError::NotFound)?;
        Ok(Arc::new(TmpFsVNode::new(Arc::clone(&self.fs), *child_ino)))
    }

    fn create(&self, name: &str, mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }

        if inode.entries.contains_key(name) {
            return Err(FsError::AlreadyExists);
        }

        // Create new inode
        let new_ino = self.fs.allocate_inode();
        let new_inode = Arc::new(RwLock::new(TmpFsInode::new(new_ino, FileType::Regular, mode)));

        // Add to parent
        inode.entries.insert(String::from(name), new_ino);
        inode.mtime = 0; // TODO: Update to current time

        // Add to filesystem
        self.fs.inodes.write().insert(new_ino, new_inode);

        Ok(Arc::new(TmpFsVNode::new(Arc::clone(&self.fs), new_ino)))
    }

    fn mkdir(&self, name: &str, mode: FileMode) -> Result<Arc<dyn VNode>, FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }

        if inode.entries.contains_key(name) {
            return Err(FsError::AlreadyExists);
        }

        // Create new directory inode
        let new_ino = self.fs.allocate_inode();
        let mut new_inode = TmpFsInode::new(new_ino, FileType::Directory, mode);
        new_inode.parent = Some(inode.ino);
        new_inode.nlink = 2; // . and parent's entry

        let new_inode = Arc::new(RwLock::new(new_inode));

        // Add to parent
        inode.entries.insert(String::from(name), new_ino);
        inode.nlink += 1; // Parent gets a link from child's ..
        inode.mtime = 0; // TODO: Update to current time

        // Add to filesystem
        self.fs.inodes.write().insert(new_ino, new_inode);

        Ok(Arc::new(TmpFsVNode::new(Arc::clone(&self.fs), new_ino)))
    }

    fn unlink(&self, name: &str) -> Result<(), FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }

        let child_ino = inode.entries.get(name).ok_or(FsError::NotFound)?;
        let child_ino = *child_ino;

        // Check if it's a directory
        if let Some(child_inode) = self.fs.inodes.read().get(&child_ino) {
            let child = child_inode.read();
            if child.file_type == FileType::Directory {
                return Err(FsError::IsADirectory);
            }
        }

        // Remove from parent
        inode.entries.remove(name);
        inode.mtime = 0; // TODO: Update to current time

        // TODO: Decrement link count and potentially free inode

        Ok(())
    }

    fn rmdir(&self, name: &str) -> Result<(), FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }

        let child_ino = inode.entries.get(name).ok_or(FsError::NotFound)?;
        let child_ino = *child_ino;

        // Check if it's a directory and empty
        if let Some(child_inode) = self.fs.inodes.read().get(&child_ino) {
            let child = child_inode.read();
            if child.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            if !child.entries.is_empty() {
                return Err(FsError::NotEmpty);
            }
        }

        // Remove from parent
        inode.entries.remove(name);
        inode.nlink -= 1;
        inode.mtime = 0; // TODO: Update to current time

        // TODO: Free inode

        Ok(())
    }

    fn rename(&self, _old_name: &str, _new_parent: Arc<dyn VNode>, _new_name: &str) -> Result<(), FsError> {
        // TODO: Implement rename
        Err(FsError::NotFound)
    }

    fn symlink(&self, name: &str, target: &str) -> Result<Arc<dyn VNode>, FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }

        if inode.entries.contains_key(name) {
            return Err(FsError::AlreadyExists);
        }

        // Create new symlink inode
        let new_ino = self.fs.allocate_inode();
        let mut new_inode = TmpFsInode::new(new_ino, FileType::Symlink, FileMode::new(0o777));
        new_inode.symlink_target = Some(String::from(target));
        new_inode.size = target.len() as u64;

        let new_inode = Arc::new(RwLock::new(new_inode));

        // Add to parent
        inode.entries.insert(String::from(name), new_ino);
        inode.mtime = 0; // TODO: Update to current time

        // Add to filesystem
        self.fs.inodes.write().insert(new_ino, new_inode);

        Ok(Arc::new(TmpFsVNode::new(Arc::clone(&self.fs), new_ino)))
    }

    fn readlink(&self) -> Result<String, FsError> {
        let inode = self.get_inode()?;
        let inode = inode.read();

        if inode.file_type != FileType::Symlink {
            return Err(FsError::InvalidArgument);
        }

        inode.symlink_target.clone().ok_or(FsError::InvalidArgument)
    }

    fn truncate(&self, size: u64) -> Result<(), FsError> {
        let inode = self.get_inode()?;
        let mut inode = inode.write();

        if inode.file_type != FileType::Regular {
            return Err(FsError::IsADirectory);
        }

        inode.data.resize(size as usize, 0);
        inode.size = size;
        inode.mtime = 0; // TODO: Update to current time

        Ok(())
    }

    fn fsync(&self) -> Result<(), FsError> {
        // No-op for in-memory filesystem
        Ok(())
    }
}

/// TmpFS filesystem
pub struct TmpFsFilesystem {
    inodes: RwLock<BTreeMap<u64, Arc<RwLock<TmpFsInode>>>>,
    next_ino: RwLock<u64>,
}

impl TmpFsFilesystem {
    /// Create a new TmpFS
    pub fn new() -> Arc<Self> {
        let fs = Arc::new(TmpFsFilesystem {
            inodes: RwLock::new(BTreeMap::new()),
            next_ino: RwLock::new(1),
        });

        // Create root directory
        let root_inode = Arc::new(RwLock::new(TmpFsInode::new(
            1,
            FileType::Directory,
            FileMode::new(0o755),
        )));
        root_inode.write().nlink = 2; // . and the root itself

        fs.inodes.write().insert(1, root_inode);
        *fs.next_ino.write() = 2;

        fs
    }

    fn allocate_inode(&self) -> u64 {
        let mut next_ino = self.next_ino.write();
        let ino = *next_ino;
        *next_ino += 1;
        ino
    }
}

impl Filesystem for TmpFsFilesystem {
    fn fs_type(&self) -> FsType {
        FsType::TmpFs
    }

    fn root(&self) -> Arc<dyn VNode> {
        Arc::new(TmpFsVNode::new(Arc::new(Self::new()), 1))
    }

    fn sync(&self) -> Result<(), FsError> {
        // No-op for in-memory filesystem
        Ok(())
    }

    fn statfs(&self) -> Result<StatFs, FsError> {
        let inodes = self.inodes.read();
        let total_size: u64 = inodes.values()
            .map(|inode| inode.read().size)
            .sum();

        Ok(StatFs {
            fs_type: 1,
            block_size: 4096,
            blocks: 0, // Unlimited for tmpfs
            blocks_free: 0,
            blocks_available: 0,
            files: inodes.len() as u64,
            files_free: u64::MAX,
            name_max: 255,
        })
    }

    fn unmount(&self) -> Result<(), FsError> {
        // Clear all data
        self.inodes.write().clear();
        Ok(())
    }
}

/// Initialize tmpfs driver
pub fn init() {
    // TmpFS can be created on demand, no initialization needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmpfs_creation() {
        let fs = TmpFsFilesystem::new();
        assert_eq!(fs.fs_type(), FsType::TmpFs);
    }
}

//! Tmpfs - In-Memory Filesystem
//!
//! Simple RAM-based filesystem, data lost on unmount

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Inode number type
pub type InodeNumber = u64;

/// File type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
}

/// File permissions
#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub owner_read: bool,
    pub owner_write: bool,
    pub owner_exec: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_exec: bool,
    pub others_read: bool,
    pub others_write: bool,
    pub others_exec: bool,
}

impl Permissions {
    /// Create default permissions (0644)
    pub fn default_file() -> Self {
        Permissions {
            owner_read: true,
            owner_write: true,
            owner_exec: false,
            group_read: true,
            group_write: false,
            group_exec: false,
            others_read: true,
            others_write: false,
            others_exec: false,
        }
    }

    /// Create default directory permissions (0755)
    pub fn default_dir() -> Self {
        Permissions {
            owner_read: true,
            owner_write: true,
            owner_exec: true,
            group_read: true,
            group_write: false,
            group_exec: true,
            others_read: true,
            others_write: false,
            others_exec: true,
        }
    }

    /// Convert to Unix mode
    pub fn to_mode(&self) -> u16 {
        let mut mode = 0u16;
        if self.owner_read {
            mode |= 0o400;
        }
        if self.owner_write {
            mode |= 0o200;
        }
        if self.owner_exec {
            mode |= 0o100;
        }
        if self.group_read {
            mode |= 0o040;
        }
        if self.group_write {
            mode |= 0o020;
        }
        if self.group_exec {
            mode |= 0o010;
        }
        if self.others_read {
            mode |= 0o004;
        }
        if self.others_write {
            mode |= 0o002;
        }
        if self.others_exec {
            mode |= 0o001;
        }
        mode
    }
}

/// Inode - represents a file or directory
pub struct Inode {
    pub number: InodeNumber,
    pub file_type: FileType,
    pub permissions: Permissions,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub data: InodeData,
    pub link_count: u32,
    pub created_time: u64,
    pub modified_time: u64,
    pub accessed_time: u64,
}

/// Inode data
pub enum InodeData {
    Regular(Vec<u8>),
    Directory(BTreeMap<String, InodeNumber>),
    Symlink(String),
    Device { major: u32, minor: u32 },
    Empty,
}

impl Inode {
    /// Create a new file inode
    pub fn new_file(number: InodeNumber) -> Self {
        Inode {
            number,
            file_type: FileType::Regular,
            permissions: Permissions::default_file(),
            uid: 0,
            gid: 0,
            size: 0,
            data: InodeData::Regular(Vec::new()),
            link_count: 1,
            created_time: 0,
            modified_time: 0,
            accessed_time: 0,
        }
    }

    /// Create a new directory inode
    pub fn new_directory(number: InodeNumber) -> Self {
        let mut entries = BTreeMap::new();
        // Add . and .. entries
        entries.insert(String::from("."), number);
        entries.insert(String::from(".."), number); // Parent set later

        Inode {
            number,
            file_type: FileType::Directory,
            permissions: Permissions::default_dir(),
            uid: 0,
            gid: 0,
            size: 0,
            data: InodeData::Directory(entries),
            link_count: 2, // . and parent reference
            created_time: 0,
            modified_time: 0,
            accessed_time: 0,
        }
    }

    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        self.file_type == FileType::Directory
    }

    /// Read file data
    pub fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize, &'static str> {
        match &self.data {
            InodeData::Regular(data) => {
                if offset >= data.len() as u64 {
                    return Ok(0);
                }

                let start = offset as usize;
                let end = (start + buffer.len()).min(data.len());
                let copy_len = end - start;

                buffer[..copy_len].copy_from_slice(&data[start..end]);
                Ok(copy_len)
            }
            _ => Err("Not a regular file"),
        }
    }

    /// Write file data
    pub fn write(&mut self, offset: u64, buffer: &[u8]) -> Result<usize, &'static str> {
        match &mut self.data {
            InodeData::Regular(data) => {
                let start = offset as usize;
                let end = start + buffer.len();

                // Extend if needed
                if end > data.len() {
                    data.resize(end, 0);
                }

                data[start..end].copy_from_slice(buffer);
                self.size = data.len() as u64;
                Ok(buffer.len())
            }
            _ => Err("Not a regular file"),
        }
    }

    /// Add directory entry
    pub fn add_entry(&mut self, name: String, inode: InodeNumber) -> Result<(), &'static str> {
        match &mut self.data {
            InodeData::Directory(entries) => {
                if entries.contains_key(&name) {
                    return Err("Entry already exists");
                }
                entries.insert(name, inode);
                Ok(())
            }
            _ => Err("Not a directory"),
        }
    }

    /// Remove directory entry
    pub fn remove_entry(&mut self, name: &str) -> Result<InodeNumber, &'static str> {
        match &mut self.data {
            InodeData::Directory(entries) => entries.remove(name).ok_or("Entry not found"),
            _ => Err("Not a directory"),
        }
    }

    /// Lookup directory entry
    pub fn lookup(&self, name: &str) -> Result<InodeNumber, &'static str> {
        match &self.data {
            InodeData::Directory(entries) => entries.get(name).copied().ok_or("Entry not found"),
            _ => Err("Not a directory"),
        }
    }

    /// List directory entries
    pub fn list_entries(&self) -> Result<Vec<String>, &'static str> {
        match &self.data {
            InodeData::Directory(entries) => Ok(entries.keys().cloned().collect()),
            _ => Err("Not a directory"),
        }
    }
}

/// Tmpfs filesystem
pub struct Tmpfs {
    inodes: Mutex<BTreeMap<InodeNumber, Box<Inode>>>,
    next_inode: AtomicU64,
    root_inode: InodeNumber,
}

impl Tmpfs {
    const ROOT_INODE: InodeNumber = 1;

    /// Create a new tmpfs instance
    pub fn new() -> Self {
        let tmpfs = Tmpfs {
            inodes: Mutex::new(BTreeMap::new()),
            next_inode: AtomicU64::new(Self::ROOT_INODE + 1),
            root_inode: Self::ROOT_INODE,
        };

        // Create root directory
        let root = Inode::new_directory(Self::ROOT_INODE);
        tmpfs.inodes.lock().insert(Self::ROOT_INODE, Box::new(root));

        tmpfs
    }

    /// Allocate a new inode number
    fn alloc_inode_number(&self) -> InodeNumber {
        self.next_inode.fetch_add(1, Ordering::SeqCst)
    }

    /// Get inode
    pub fn get_inode(&self, inode: InodeNumber) -> Option<Box<Inode>> {
        self.inodes.lock().get(&inode).map(|i| {
            // Clone the inode
            // This is a simplified approach; real filesystems use reference counting
            Box::new(Inode {
                number: i.number,
                file_type: i.file_type,
                permissions: i.permissions,
                uid: i.uid,
                gid: i.gid,
                size: i.size,
                data: match &i.data {
                    InodeData::Regular(d) => InodeData::Regular(d.clone()),
                    InodeData::Directory(d) => InodeData::Directory(d.clone()),
                    InodeData::Symlink(d) => InodeData::Symlink(d.clone()),
                    InodeData::Device { major, minor } => InodeData::Device {
                        major: *major,
                        minor: *minor,
                    },
                    InodeData::Empty => InodeData::Empty,
                },
                link_count: i.link_count,
                created_time: i.created_time,
                modified_time: i.modified_time,
                accessed_time: i.accessed_time,
            })
        })
    }

    /// Create a new file
    pub fn create_file(
        &self,
        parent: InodeNumber,
        name: String,
    ) -> Result<InodeNumber, &'static str> {
        let inode_num = self.alloc_inode_number();
        let inode = Inode::new_file(inode_num);

        let mut inodes = self.inodes.lock();

        // Add to parent directory
        if let Some(parent_inode) = inodes.get_mut(&parent) {
            parent_inode.add_entry(name, inode_num)?;
        } else {
            return Err("Parent directory not found");
        }

        inodes.insert(inode_num, Box::new(inode));
        Ok(inode_num)
    }

    /// Create a new directory
    pub fn create_directory(
        &self,
        parent: InodeNumber,
        name: String,
    ) -> Result<InodeNumber, &'static str> {
        let inode_num = self.alloc_inode_number();
        let mut inode = Inode::new_directory(inode_num);

        // Set parent (..)
        if let InodeData::Directory(ref mut entries) = inode.data {
            entries.insert(String::from(".."), parent);
        }

        let mut inodes = self.inodes.lock();

        // Add to parent directory
        if let Some(parent_inode) = inodes.get_mut(&parent) {
            parent_inode.add_entry(name, inode_num)?;
            parent_inode.link_count += 1; // New subdirectory references parent
        } else {
            return Err("Parent directory not found");
        }

        inodes.insert(inode_num, Box::new(inode));
        Ok(inode_num)
    }

    /// Resolve a path to an inode number
    ///
    /// Supports absolute paths (starting with `/`).
    pub fn lookup_path(&self, path: &str) -> Result<InodeNumber, &'static str> {
        let inodes = self.inodes.lock();

        if path.is_empty() || path == "/" {
            return Ok(self.root_inode);
        }

        let mut current = self.root_inode;

        for component in path.trim_start_matches('/').split('/') {
            if component.is_empty() || component == "." {
                continue;
            }
            let inode = inodes.get(&current).ok_or("Inode not found")?;
            match &inode.data {
                InodeData::Directory(entries) => {
                    current = *entries.get(component).ok_or("No such file or directory")?;
                }
                _ => return Err("Not a directory"),
            }
        }

        Ok(current)
    }

    /// Remove a file from its parent directory (unlink)
    pub fn unlink(&self, parent: InodeNumber, name: &str) -> Result<(), &'static str> {
        let mut inodes = self.inodes.lock();

        let file_inode_num = match inodes.get(&parent) {
            Some(p) => match &p.data {
                InodeData::Directory(entries) => {
                    *entries.get(name).ok_or("No such file or directory")?
                }
                _ => return Err("Not a directory"),
            },
            None => return Err("Parent directory not found"),
        };

        if let Some(inode) = inodes.get(&file_inode_num) {
            if inode.is_directory() {
                return Err("Is a directory");
            }
        } else {
            return Err("Inode not found");
        }

        // Remove entry from parent directory
        if let Some(parent_inode) = inodes.get_mut(&parent) {
            parent_inode.remove_entry(name)?;
        }

        // Decrement link count and free inode if zero
        if let Some(inode) = inodes.get_mut(&file_inode_num) {
            inode.link_count = inode.link_count.saturating_sub(1);
            if inode.link_count == 0 {
                inodes.remove(&file_inode_num);
            }
        }

        Ok(())
    }

    /// Remove an empty directory
    pub fn rmdir(&self, parent: InodeNumber, name: &str) -> Result<(), &'static str> {
        let mut inodes = self.inodes.lock();

        let dir_inode_num = match inodes.get(&parent) {
            Some(p) => match &p.data {
                InodeData::Directory(entries) => {
                    *entries.get(name).ok_or("No such file or directory")?
                }
                _ => return Err("Not a directory"),
            },
            None => return Err("Parent directory not found"),
        };

        match inodes.get(&dir_inode_num) {
            Some(inode) => {
                if !inode.is_directory() {
                    return Err("Not a directory");
                }
                if let InodeData::Directory(entries) = &inode.data {
                    if entries.len() > 2 {
                        return Err("Directory not empty");
                    }
                }
            }
            None => return Err("Inode not found"),
        }

        if let Some(parent_inode) = inodes.get_mut(&parent) {
            parent_inode.remove_entry(name)?;
            parent_inode.link_count = parent_inode.link_count.saturating_sub(1);
        }

        inodes.remove(&dir_inode_num);
        Ok(())
    }

    /// Rename a file or directory
    pub fn rename(
        &self,
        old_parent: InodeNumber,
        old_name: &str,
        new_parent: InodeNumber,
        new_name: &str,
    ) -> Result<(), &'static str> {
        let mut inodes = self.inodes.lock();

        let inode_num = match inodes.get(&old_parent) {
            Some(p) => match &p.data {
                InodeData::Directory(entries) => {
                    *entries.get(old_name).ok_or("No such file or directory")?
                }
                _ => return Err("Not a directory"),
            },
            None => return Err("Old parent not found"),
        };

        if !inodes.contains_key(&new_parent) {
            return Err("New parent directory not found");
        }

        // Remove from old parent
        if let Some(old_parent_inode) = inodes.get_mut(&old_parent) {
            old_parent_inode.remove_entry(old_name)?;
        }

        // Add to new parent, overwriting any existing entry with the same name
        if let Some(new_parent_inode) = inodes.get_mut(&new_parent) {
            match &mut new_parent_inode.data {
                InodeData::Directory(entries) => {
                    entries.insert(String::from(new_name), inode_num);
                }
                _ => return Err("New parent is not a directory"),
            }
        }

        Ok(())
    }

    /// Get file stat information
    pub fn stat(&self, inode_num: InodeNumber) -> Result<FileStat, &'static str> {
        let inodes = self.inodes.lock();
        let inode = inodes.get(&inode_num).ok_or("Inode not found")?;
        Ok(FileStat {
            inode: inode.number,
            file_type: inode.file_type,
            mode: inode.permissions.to_mode(),
            uid: inode.uid,
            gid: inode.gid,
            size: inode.size,
            link_count: inode.link_count,
            created_time: inode.created_time,
            modified_time: inode.modified_time,
            accessed_time: inode.accessed_time,
        })
    }

    /// Root inode number
    pub fn root(&self) -> InodeNumber {
        self.root_inode
    }
}

/// File stat information returned by stat/fstat
#[derive(Debug, Clone, Copy)]
pub struct FileStat {
    /// Inode number
    pub inode: InodeNumber,
    /// File type
    pub file_type: FileType,
    /// Permission mode bits
    pub mode: u16,
    /// User ID of owner
    pub uid: u32,
    /// Group ID of owner
    pub gid: u32,
    /// File size in bytes
    pub size: u64,
    /// Number of hard links
    pub link_count: u32,
    /// Creation time
    pub created_time: u64,
    /// Last modification time
    pub modified_time: u64,
    /// Last access time
    pub accessed_time: u64,
}

impl Default for Tmpfs {
    fn default() -> Self {
        Self::new()
    }
}

/// Global tmpfs instance
static TMPFS: Mutex<Option<Tmpfs>> = Mutex::new(None);

/// Initialize tmpfs
pub fn init() {
    let mut fs = TMPFS.lock();
    *fs = Some(Tmpfs::new());
}

/// Get the global tmpfs instance
pub fn get() -> Option<Tmpfs> {
    // This is a simplified approach - cloning the entire filesystem
    // Real implementation would use Arc or similar
    None // Return None for now as cloning is complex
}

/// Resolve a path in the global tmpfs
pub fn global_lookup_path(path: &str) -> Result<InodeNumber, &'static str> {
    let fs = TMPFS.lock();
    fs.as_ref()
        .ok_or("Tmpfs not initialized")?
        .lookup_path(path)
}

/// Create a directory in the global tmpfs
pub fn global_mkdir(path: &str) -> Result<InodeNumber, &'static str> {
    let (parent_path, name) = split_path(path);
    let parent_inode = global_lookup_path(parent_path)?;
    let fs = TMPFS.lock();
    fs.as_ref()
        .ok_or("Tmpfs not initialized")?
        .create_directory(parent_inode, String::from(name))
}

/// Remove a directory in the global tmpfs
pub fn global_rmdir(path: &str) -> Result<(), &'static str> {
    let (parent_path, name) = split_path(path);
    let parent_inode = global_lookup_path(parent_path)?;
    let fs = TMPFS.lock();
    fs.as_ref()
        .ok_or("Tmpfs not initialized")?
        .rmdir(parent_inode, name)
}

/// Unlink a file in the global tmpfs
pub fn global_unlink(path: &str) -> Result<(), &'static str> {
    let (parent_path, name) = split_path(path);
    let parent_inode = global_lookup_path(parent_path)?;
    let fs = TMPFS.lock();
    fs.as_ref()
        .ok_or("Tmpfs not initialized")?
        .unlink(parent_inode, name)
}

/// Rename a path in the global tmpfs
pub fn global_rename(old_path: &str, new_path: &str) -> Result<(), &'static str> {
    let (old_parent_path, old_name) = split_path(old_path);
    let (new_parent_path, new_name) = split_path(new_path);
    let old_parent = global_lookup_path(old_parent_path)?;
    let new_parent = global_lookup_path(new_parent_path)?;
    let fs = TMPFS.lock();
    fs.as_ref()
        .ok_or("Tmpfs not initialized")?
        .rename(old_parent, old_name, new_parent, new_name)
}

/// Get stat information for a path in the global tmpfs
pub fn global_stat(path: &str) -> Result<FileStat, &'static str> {
    let inode_num = global_lookup_path(path)?;
    let fs = TMPFS.lock();
    fs.as_ref().ok_or("Tmpfs not initialized")?.stat(inode_num)
}

/// Get stat information for an inode number directly in the global tmpfs
///
/// Returns `None` if tmpfs is not initialized or the inode does not exist.
pub fn global_stat_inode(inode_num: InodeNumber) -> Option<FileStat> {
    let fs = TMPFS.lock();
    fs.as_ref()?.stat(inode_num).ok()
}

/// Create a file in the global tmpfs
pub fn global_create_file(path: &str) -> Result<InodeNumber, &'static str> {
    let (parent_path, name) = split_path(path);
    let parent_inode = global_lookup_path(parent_path)?;
    let fs = TMPFS.lock();
    fs.as_ref()
        .ok_or("Tmpfs not initialized")?
        .create_file(parent_inode, String::from(name))
}

/// Get the root inode number of the global tmpfs
pub fn global_root() -> Result<InodeNumber, &'static str> {
    let fs = TMPFS.lock();
    Ok(fs.as_ref().ok_or("Tmpfs not initialized")?.root())
}

/// Split a path into its parent directory and final component
fn split_path(path: &str) -> (&str, &str) {
    let path = path.trim_end_matches('/');
    match path.rfind('/') {
        Some(0) => ("/", &path[1..]),
        Some(pos) => (&path[..pos], &path[pos + 1..]),
        None => ("/", path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_permissions() {
        let perms = Permissions::default_file();
        let mode = perms.to_mode();
        assert_eq!(mode, 0o644);
    }

    #[test]
    fn test_tmpfs_creation() {
        let fs = Tmpfs::new();
        assert_eq!(fs.root(), Tmpfs::ROOT_INODE);

        let root = fs.get_inode(fs.root()).unwrap();
        assert!(root.is_directory());
    }

    #[test]
    fn test_file_creation() {
        let fs = Tmpfs::new();
        let file_inode = fs.create_file(fs.root(), String::from("test.txt")).unwrap();

        let file = fs.get_inode(file_inode).unwrap();
        assert_eq!(file.file_type, FileType::Regular);
        assert_eq!(file.size, 0);
    }

    #[test]
    fn test_directory_creation() {
        let fs = Tmpfs::new();
        let dir_inode = fs
            .create_directory(fs.root(), String::from("testdir"))
            .unwrap();

        let dir = fs.get_inode(dir_inode).unwrap();
        assert!(dir.is_directory());

        // Check . and .. entries
        assert_eq!(dir.lookup(".").unwrap(), dir_inode);
        assert_eq!(dir.lookup("..").unwrap(), fs.root());
    }

    #[test]
    fn test_file_read_write() {
        let mut inode = Inode::new_file(1);

        let write_data = b"Hello, World!";
        let written = inode.write(0, write_data).unwrap();
        assert_eq!(written, write_data.len());
        assert_eq!(inode.size, write_data.len() as u64);

        let mut read_buffer = vec![0u8; write_data.len()];
        let read = inode.read(0, &mut read_buffer).unwrap();
        assert_eq!(read, write_data.len());
        assert_eq!(&read_buffer, write_data);
    }

    #[test]
    fn test_lookup_path() {
        let fs = Tmpfs::new();
        fs.create_directory(fs.root(), String::from("etc")).unwrap();
        fs.create_directory(fs.root(), String::from("usr")).unwrap();

        let etc_inode = fs.lookup_path("/etc").unwrap();
        assert_ne!(etc_inode, fs.root());

        let root_inode = fs.lookup_path("/").unwrap();
        assert_eq!(root_inode, fs.root());
    }

    #[test]
    fn test_unlink() {
        let fs = Tmpfs::new();
        let file_inode = fs
            .create_file(fs.root(), String::from("todelete.txt"))
            .unwrap();
        assert!(fs.get_inode(file_inode).is_some());

        fs.unlink(fs.root(), "todelete.txt").unwrap();
        assert!(fs.get_inode(file_inode).is_none());
    }

    #[test]
    fn test_rmdir() {
        let fs = Tmpfs::new();
        let dir_inode = fs
            .create_directory(fs.root(), String::from("emptydir"))
            .unwrap();
        assert!(fs.get_inode(dir_inode).is_some());

        fs.rmdir(fs.root(), "emptydir").unwrap();
        assert!(fs.get_inode(dir_inode).is_none());
    }

    #[test]
    fn test_rmdir_nonempty_fails() {
        let fs = Tmpfs::new();
        let dir_inode = fs
            .create_directory(fs.root(), String::from("nonempty"))
            .unwrap();
        fs.create_file(dir_inode, String::from("file.txt")).unwrap();

        assert!(fs.rmdir(fs.root(), "nonempty").is_err());
    }

    #[test]
    fn test_rename() {
        let fs = Tmpfs::new();
        fs.create_file(fs.root(), String::from("old.txt")).unwrap();

        fs.rename(fs.root(), "old.txt", fs.root(), "new.txt")
            .unwrap();

        assert!(fs.lookup_path("/old.txt").is_err());
        assert!(fs.lookup_path("/new.txt").is_ok());
    }

    #[test]
    fn test_stat() {
        let fs = Tmpfs::new();
        let file_inode = fs
            .create_file(fs.root(), String::from("stat_test.txt"))
            .unwrap();

        let stat = fs.stat(file_inode).unwrap();
        assert_eq!(stat.inode, file_inode);
        assert_eq!(stat.file_type, FileType::Regular);
        assert_eq!(stat.size, 0);
    }

    #[test]
    fn test_split_path() {
        assert_eq!(split_path("/etc/passwd"), ("/etc", "passwd"));
        assert_eq!(split_path("/file.txt"), ("/", "file.txt"));
        assert_eq!(split_path("file.txt"), ("/", "file.txt"));
        assert_eq!(split_path("/a/b/c"), ("/a/b", "c"));
    }
}

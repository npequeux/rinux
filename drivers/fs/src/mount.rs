//! VFS Mount Management
//!
//! Manages filesystem mount points

use super::vfs::{Filesystem, VNode};
use crate::FsError;
use alloc::sync::Arc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::RwLock;

/// Mount point information
pub struct MountPoint {
    /// Path where filesystem is mounted
    pub path: String,
    /// Mounted filesystem
    pub filesystem: Arc<dyn Filesystem>,
    /// Mount flags
    pub flags: MountFlags,
}

/// Mount flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountFlags {
    /// Read-only mount
    pub readonly: bool,
    /// No execution of programs
    pub noexec: bool,
    /// No device special files
    pub nodev: bool,
    /// No setuid/setgid
    pub nosuid: bool,
}

impl MountFlags {
    pub const fn new() -> Self {
        Self {
            readonly: false,
            noexec: false,
            nodev: false,
            nosuid: false,
        }
    }

    pub const fn readonly() -> Self {
        Self {
            readonly: true,
            noexec: false,
            nodev: false,
            nosuid: false,
        }
    }
}

/// Global mount table
static MOUNT_TABLE: RwLock<Vec<MountPoint>> = RwLock::new(Vec::new());

/// Root filesystem
static ROOT_FS: RwLock<Option<Arc<dyn Filesystem>>> = RwLock::new(None);

/// Mount a filesystem at the specified path
pub fn mount(
    path: &str,
    filesystem: Arc<dyn Filesystem>,
    flags: MountFlags,
) -> Result<(), FsError> {
    let mut table = MOUNT_TABLE.write();
    
    // Check if path is already mounted
    if table.iter().any(|mp| mp.path == path) {
        return Err(FsError::AlreadyExists);
    }
    
    let mount_point = MountPoint {
        path: path.to_string(),
        filesystem,
        flags,
    };
    
    table.push(mount_point);
    Ok(())
}

/// Unmount a filesystem
pub fn unmount(path: &str) -> Result<(), FsError> {
    let mut table = MOUNT_TABLE.write();
    
    let index = table
        .iter()
        .position(|mp| mp.path == path)
        .ok_or(FsError::NotFound)?;
    
    // Unmount the filesystem
    table[index].filesystem.unmount()?;
    
    // Remove from mount table
    table.remove(index);
    
    Ok(())
}

/// Get filesystem mounted at path
pub fn get_mount(path: &str) -> Option<Arc<dyn Filesystem>> {
    let table = MOUNT_TABLE.read();
    
    // Find longest matching mount point
    let mut best_match: Option<&MountPoint> = None;
    let mut best_len = 0;
    
    for mp in table.iter() {
        if path.starts_with(&mp.path) && mp.path.len() > best_len {
            best_match = Some(mp);
            best_len = mp.path.len();
        }
    }
    
    best_match.map(|mp| mp.filesystem.clone())
}

/// Set the root filesystem
pub fn set_root(filesystem: Arc<dyn Filesystem>) -> Result<(), FsError> {
    let mut root = ROOT_FS.write();
    
    if root.is_some() {
        return Err(FsError::AlreadyExists);
    }
    
    *root = Some(filesystem.clone());
    
    // Mount root at "/"
    drop(root);
    mount("/", filesystem, MountFlags::new())?;
    
    Ok(())
}

/// Get the root filesystem
pub fn get_root() -> Option<Arc<dyn Filesystem>> {
    ROOT_FS.read().clone()
}

/// Get root VNode
pub fn get_root_vnode() -> Option<Arc<dyn VNode>> {
    get_root().map(|fs| fs.root())
}

/// List all mount points
pub fn list_mounts() -> Vec<(String, String)> {
    let table = MOUNT_TABLE.read();
    table
        .iter()
        .map(|mp| (mp.path.clone(), format!("{:?}", mp.filesystem.fs_type())))
        .collect()
}

/// Initialize mount subsystem
pub fn init() {
    // Mount table is initialized statically
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mount_flags() {
        let flags = MountFlags::new();
        assert!(!flags.readonly);
        
        let ro_flags = MountFlags::readonly();
        assert!(ro_flags.readonly);
    }
}

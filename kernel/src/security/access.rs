//! Access Control
//!
//! Permission checking for files and resources.

use crate::types::{Gid, Uid};

/// File permission bits (Unix-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePermissions {
    /// Raw permission bits
    bits: u16,
}

impl FilePermissions {
    /// Owner read permission
    pub const OWNER_READ: u16 = 0o400;
    /// Owner write permission
    pub const OWNER_WRITE: u16 = 0o200;
    /// Owner execute permission
    pub const OWNER_EXEC: u16 = 0o100;
    /// Owner all permissions
    pub const OWNER_ALL: u16 = Self::OWNER_READ | Self::OWNER_WRITE | Self::OWNER_EXEC;

    /// Group read permission
    pub const GROUP_READ: u16 = 0o040;
    /// Group write permission
    pub const GROUP_WRITE: u16 = 0o020;
    /// Group execute permission
    pub const GROUP_EXEC: u16 = 0o010;
    /// Group all permissions
    pub const GROUP_ALL: u16 = Self::GROUP_READ | Self::GROUP_WRITE | Self::GROUP_EXEC;

    /// Other read permission
    pub const OTHER_READ: u16 = 0o004;
    /// Other write permission
    pub const OTHER_WRITE: u16 = 0o002;
    /// Other execute permission
    pub const OTHER_EXEC: u16 = 0o001;
    /// Other all permissions
    pub const OTHER_ALL: u16 = Self::OTHER_READ | Self::OTHER_WRITE | Self::OTHER_EXEC;

    /// Set-user-ID bit
    pub const SETUID: u16 = 0o4000;
    /// Set-group-ID bit
    pub const SETGID: u16 = 0o2000;
    /// Sticky bit
    pub const STICKY: u16 = 0o1000;

    /// Create new permissions from mode
    pub const fn new(mode: u16) -> Self {
        FilePermissions { bits: mode }
    }

    /// Create from octal mode (e.g., 0o755)
    pub const fn from_mode(mode: u16) -> Self {
        Self::new(mode & 0o7777)
    }

    /// Get raw bits
    pub const fn bits(&self) -> u16 {
        self.bits
    }

    /// Get octal mode
    pub const fn mode(&self) -> u16 {
        self.bits & 0o7777
    }

    /// Check if owner can read
    pub const fn owner_can_read(&self) -> bool {
        (self.bits & Self::OWNER_READ) != 0
    }

    /// Check if owner can write
    pub const fn owner_can_write(&self) -> bool {
        (self.bits & Self::OWNER_WRITE) != 0
    }

    /// Check if owner can execute
    pub const fn owner_can_exec(&self) -> bool {
        (self.bits & Self::OWNER_EXEC) != 0
    }

    /// Check if group can read
    pub const fn group_can_read(&self) -> bool {
        (self.bits & Self::GROUP_READ) != 0
    }

    /// Check if group can write
    pub const fn group_can_write(&self) -> bool {
        (self.bits & Self::GROUP_WRITE) != 0
    }

    /// Check if group can execute
    pub const fn group_can_exec(&self) -> bool {
        (self.bits & Self::GROUP_EXEC) != 0
    }

    /// Check if other can read
    pub const fn other_can_read(&self) -> bool {
        (self.bits & Self::OTHER_READ) != 0
    }

    /// Check if other can write
    pub const fn other_can_write(&self) -> bool {
        (self.bits & Self::OTHER_WRITE) != 0
    }

    /// Check if other can execute
    pub const fn other_can_exec(&self) -> bool {
        (self.bits & Self::OTHER_EXEC) != 0
    }

    /// Check if setuid bit is set
    pub const fn is_setuid(&self) -> bool {
        (self.bits & Self::SETUID) != 0
    }

    /// Check if setgid bit is set
    pub const fn is_setgid(&self) -> bool {
        (self.bits & Self::SETGID) != 0
    }

    /// Check if sticky bit is set
    pub const fn is_sticky(&self) -> bool {
        (self.bits & Self::STICKY) != 0
    }
}

/// Access mode for permission checks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Execute access
    Execute,
}

/// Check if user has permission to access a file
///
/// # Arguments
///
/// * `uid` - User ID attempting access
/// * `gid` - Group ID attempting access
/// * `file_uid` - File owner UID
/// * `file_gid` - File owner GID
/// * `perms` - File permissions
/// * `mode` - Requested access mode
///
/// # Returns
///
/// `true` if access is granted, `false` otherwise
pub fn check_permission(
    uid: Uid,
    gid: Gid,
    file_uid: Uid,
    file_gid: Gid,
    perms: FilePermissions,
    mode: AccessMode,
) -> bool {
    // Root (uid 0) can access anything (except execute on non-executable files)
    if uid == 0 {
        return match mode {
            AccessMode::Execute => {
                perms.owner_can_exec() || perms.group_can_exec() || perms.other_can_exec()
            }
            _ => true,
        };
    }

    // Check owner permissions
    if uid == file_uid {
        return match mode {
            AccessMode::Read => perms.owner_can_read(),
            AccessMode::Write => perms.owner_can_write(),
            AccessMode::Execute => perms.owner_can_exec(),
        };
    }

    // Check group permissions
    if gid == file_gid {
        return match mode {
            AccessMode::Read => perms.group_can_read(),
            AccessMode::Write => perms.group_can_write(),
            AccessMode::Execute => perms.group_can_exec(),
        };
    }

    // Check other permissions
    match mode {
        AccessMode::Read => perms.other_can_read(),
        AccessMode::Write => perms.other_can_write(),
        AccessMode::Execute => perms.other_can_exec(),
    }
}

/// Check read permission
pub fn can_read(uid: Uid, gid: Gid, file_uid: Uid, file_gid: Gid, perms: FilePermissions) -> bool {
    check_permission(uid, gid, file_uid, file_gid, perms, AccessMode::Read)
}

/// Check write permission
pub fn can_write(uid: Uid, gid: Gid, file_uid: Uid, file_gid: Gid, perms: FilePermissions) -> bool {
    check_permission(uid, gid, file_uid, file_gid, perms, AccessMode::Write)
}

/// Check execute permission
pub fn can_execute(
    uid: Uid,
    gid: Gid,
    file_uid: Uid,
    file_gid: Gid,
    perms: FilePermissions,
) -> bool {
    check_permission(uid, gid, file_uid, file_gid, perms, AccessMode::Execute)
}

/// Check if user can change file ownership
pub fn can_chown(uid: Uid, file_uid: Uid) -> bool {
    // Only root or file owner can change ownership
    uid == 0 || uid == file_uid
}

/// Check if user can change file permissions
pub fn can_chmod(uid: Uid, file_uid: Uid) -> bool {
    // Only root or file owner can change permissions
    uid == 0 || uid == file_uid
}

/// Check if user can delete a file in a directory
pub fn can_delete(
    uid: Uid,
    gid: Gid,
    dir_uid: Uid,
    dir_gid: Gid,
    dir_perms: FilePermissions,
) -> bool {
    // Need write permission on directory
    if !can_write(uid, gid, dir_uid, dir_gid, dir_perms) {
        return false;
    }

    // If sticky bit is set, only owner can delete
    if dir_perms.is_sticky() {
        return uid == 0 || uid == dir_uid;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_permissions_from_mode() {
        let perms = FilePermissions::from_mode(0o755);
        assert!(perms.owner_can_read());
        assert!(perms.owner_can_write());
        assert!(perms.owner_can_exec());
        assert!(perms.group_can_read());
        assert!(!perms.group_can_write());
        assert!(perms.group_can_exec());
        assert!(perms.other_can_read());
        assert!(!perms.other_can_write());
        assert!(perms.other_can_exec());
    }

    #[test]
    fn test_file_permissions_special_bits() {
        let perms = FilePermissions::from_mode(0o4755);
        assert!(perms.is_setuid());
        assert!(!perms.is_setgid());
        assert!(!perms.is_sticky());

        let perms = FilePermissions::from_mode(0o2755);
        assert!(!perms.is_setuid());
        assert!(perms.is_setgid());
        assert!(!perms.is_sticky());

        let perms = FilePermissions::from_mode(0o1755);
        assert!(!perms.is_setuid());
        assert!(!perms.is_setgid());
        assert!(perms.is_sticky());
    }

    #[test]
    fn test_root_access() {
        let perms = FilePermissions::from_mode(0o000);
        // Root can read/write anything
        assert!(can_read(0, 0, 1000, 1000, perms));
        assert!(can_write(0, 0, 1000, 1000, perms));
        // Root can't execute non-executable files
        assert!(!can_execute(0, 0, 1000, 1000, perms));
    }

    #[test]
    fn test_owner_permissions() {
        let perms = FilePermissions::from_mode(0o700);
        // Owner can access
        assert!(can_read(1000, 1000, 1000, 1000, perms));
        assert!(can_write(1000, 1000, 1000, 1000, perms));
        assert!(can_execute(1000, 1000, 1000, 1000, perms));

        // Group can't access
        assert!(!can_read(1001, 1000, 1000, 1000, perms));
        assert!(!can_write(1001, 1000, 1000, 1000, perms));
        assert!(!can_execute(1001, 1000, 1000, 1000, perms));
    }

    #[test]
    fn test_group_permissions() {
        let perms = FilePermissions::from_mode(0o070);
        // Group can access
        assert!(can_read(1001, 1000, 1000, 1000, perms));
        assert!(can_write(1001, 1000, 1000, 1000, perms));
        assert!(can_execute(1001, 1000, 1000, 1000, perms));

        // Other can't access
        assert!(!can_read(1001, 1001, 1000, 1000, perms));
        assert!(!can_write(1001, 1001, 1000, 1000, perms));
        assert!(!can_execute(1001, 1001, 1000, 1000, perms));
    }

    #[test]
    fn test_other_permissions() {
        let perms = FilePermissions::from_mode(0o007);
        // Other can access
        assert!(can_read(1001, 1001, 1000, 1000, perms));
        assert!(can_write(1001, 1001, 1000, 1000, perms));
        assert!(can_execute(1001, 1001, 1000, 1000, perms));
    }

    #[test]
    fn test_can_chown() {
        // Root can chown
        assert!(can_chown(0, 1000));
        // Owner can chown
        assert!(can_chown(1000, 1000));
        // Others can't chown
        assert!(!can_chown(1001, 1000));
    }

    #[test]
    fn test_can_chmod() {
        // Root can chmod
        assert!(can_chmod(0, 1000));
        // Owner can chmod
        assert!(can_chmod(1000, 1000));
        // Others can't chmod
        assert!(!can_chmod(1001, 1000));
    }

    #[test]
    fn test_sticky_bit_delete() {
        let perms = FilePermissions::from_mode(0o1777);
        // Root can delete
        assert!(can_delete(0, 0, 1000, 1000, perms));
        // Owner can delete
        assert!(can_delete(1000, 1000, 1000, 1000, perms));
        // Others with write permission can't delete with sticky bit
        assert!(!can_delete(1001, 1001, 1000, 1000, perms));
    }
}

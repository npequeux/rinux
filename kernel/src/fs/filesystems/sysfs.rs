//! Sysfs - System/Device Information Filesystem
//!
//! Virtual filesystem that exposes kernel objects (devices, drivers, etc.)

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Sysfs attribute read callback
pub type SysfsReadFn = fn(&str) -> Result<Vec<u8>, &'static str>;

/// Sysfs attribute write callback
pub type SysfsWriteFn = fn(&str, &[u8]) -> Result<(), &'static str>;

/// Sysfs entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysfsEntryType {
    Directory,
    File,
    Link,
}

/// Sysfs attribute
pub struct SysfsAttribute {
    pub name: String,
    pub read_fn: Option<SysfsReadFn>,
    pub write_fn: Option<SysfsWriteFn>,
    pub permissions: u16,
}

/// Sysfs entry
pub struct SysfsEntry {
    pub path: String,
    pub entry_type: SysfsEntryType,
    pub target: Option<String>, // For symbolic links
    pub attributes: Vec<SysfsAttribute>,
}

/// Sysfs structure
pub struct Sysfs {
    entries: Mutex<BTreeMap<String, SysfsEntry>>,
}

impl Sysfs {
    /// Create a new sysfs
    pub fn new() -> Self {
        let mut sysfs = Sysfs {
            entries: Mutex::new(BTreeMap::new()),
        };

        // Register default sysfs structure
        sysfs.register_default_entries();
        sysfs
    }

    /// Register default /sys entries
    fn register_default_entries(&mut self) {
        // Create top-level directories
        self.create_directory("/sys");
        self.create_directory("/sys/block");
        self.create_directory("/sys/bus");
        self.create_directory("/sys/class");
        self.create_directory("/sys/devices");
        self.create_directory("/sys/firmware");
        self.create_directory("/sys/fs");
        self.create_directory("/sys/kernel");
        self.create_directory("/sys/module");
        self.create_directory("/sys/power");

        // Kernel attributes
        self.create_directory("/sys/kernel");
        self.add_attribute("/sys/kernel", "version", 0o444, Some(read_kernel_version), None);
    }

    /// Create a directory entry
    pub fn create_directory(&mut self, path: &str) {
        let mut entries = self.entries.lock();
        entries.insert(
            String::from(path),
            SysfsEntry {
                path: String::from(path),
                entry_type: SysfsEntryType::Directory,
                target: None,
                attributes: Vec::new(),
            },
        );
    }

    /// Add an attribute to a directory
    pub fn add_attribute(
        &mut self,
        dir: &str,
        name: &str,
        permissions: u16,
        read_fn: Option<SysfsReadFn>,
        write_fn: Option<SysfsWriteFn>,
    ) {
        let attr_path = alloc::format!("{}/{}", dir, name);
        let mut entries = self.entries.lock();

        // Create the attribute as a file
        entries.insert(
            attr_path.clone(),
            SysfsEntry {
                path: attr_path,
                entry_type: SysfsEntryType::File,
                target: None,
                attributes: alloc::vec![SysfsAttribute {
                    name: String::from(name),
                    read_fn,
                    write_fn,
                    permissions,
                }],
            },
        );
    }

    /// Create a symbolic link
    pub fn create_link(&mut self, path: &str, target: &str) {
        let mut entries = self.entries.lock();
        entries.insert(
            String::from(path),
            SysfsEntry {
                path: String::from(path),
                entry_type: SysfsEntryType::Link,
                target: Some(String::from(target)),
                attributes: Vec::new(),
            },
        );
    }

    /// Read from a sysfs entry
    pub fn read(&self, path: &str) -> Result<Vec<u8>, &'static str> {
        let entries = self.entries.lock();
        if let Some(entry) = entries.get(path) {
            match entry.entry_type {
                SysfsEntryType::File => {
                    if let Some(attr) = entry.attributes.first() {
                        if let Some(read_fn) = attr.read_fn {
                            drop(entries); // Release lock before calling read function
                            return read_fn(path);
                        }
                    }
                    Err("No read function")
                }
                SysfsEntryType::Link => {
                    if let Some(target) = entry.target.clone() {
                        // Follow the link
                        drop(entries);
                        return self.read(&target);
                    }
                    Err("Invalid link")
                }
                SysfsEntryType::Directory => Err("Cannot read directory"),
            }
        } else {
            Err("Entry not found")
        }
    }

    /// Write to a sysfs entry
    pub fn write(&self, path: &str, data: &[u8]) -> Result<(), &'static str> {
        let entries = self.entries.lock();
        if let Some(entry) = entries.get(path) {
            if entry.entry_type == SysfsEntryType::File {
                if let Some(attr) = entry.attributes.first() {
                    if let Some(write_fn) = attr.write_fn {
                        return write_fn(path, data);
                    }
                }
            }
        }
        Err("Entry not found or not writable")
    }

    /// List entries in a directory
    pub fn list(&self, dir: &str) -> Result<Vec<String>, &'static str> {
        let entries = self.entries.lock();
        
        // Check if directory exists
        if !entries.contains_key(dir) {
            return Err("Directory not found");
        }

        let prefix = if dir.ends_with('/') {
            String::from(dir)
        } else {
            alloc::format!("{}/", dir)
        };

        let mut results = Vec::new();
        for (path, _) in entries.iter() {
            if path.starts_with(&prefix) && path != dir {
                let rest = &path[prefix.len()..];
                if !rest.contains('/') {
                    results.push(String::from(rest));
                }
            }
        }

        Ok(results)
    }
}

impl Default for Sysfs {
    fn default() -> Self {
        Self::new()
    }
}

/// Read kernel version
fn read_kernel_version(_path: &str) -> Result<Vec<u8>, &'static str> {
    let version = b"0.1.0\n";
    Ok(version.to_vec())
}

/// Global sysfs instance
static SYSFS: Mutex<Option<Sysfs>> = Mutex::new(None);

/// Initialize sysfs
pub fn init() {
    let mut fs = SYSFS.lock();
    *fs = Some(Sysfs::new());
}

/// Read from sysfs
pub fn read(path: &str) -> Result<Vec<u8>, &'static str> {
    let fs = SYSFS.lock();
    if let Some(ref sysfs) = *fs {
        sysfs.read(path)
    } else {
        Err("Sysfs not initialized")
    }
}

/// Write to sysfs
pub fn write(path: &str, data: &[u8]) -> Result<(), &'static str> {
    let fs = SYSFS.lock();
    if let Some(ref sysfs) = *fs {
        sysfs.write(path, data)
    } else {
        Err("Sysfs not initialized")
    }
}

/// List sysfs directory
pub fn list(dir: &str) -> Result<Vec<String>, &'static str> {
    let fs = SYSFS.lock();
    if let Some(ref sysfs) = *fs {
        sysfs.list(dir)
    } else {
        Err("Sysfs not initialized")
    }
}

/// Register a device in sysfs
pub fn register_device(_class: &str, _name: &str) -> Result<(), &'static str> {
    // TODO: Implement proper device registration
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysfs_creation() {
        let sysfs = Sysfs::new();
        assert!(sysfs.list("/sys").is_ok());
    }

    #[test]
    fn test_kernel_version() {
        let sysfs = Sysfs::new();
        let data = sysfs.read("/sys/kernel/version").unwrap();
        assert!(!data.is_empty());
    }
}

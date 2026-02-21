//! Procfs - Process Information Filesystem
//!
//! Virtual filesystem that exposes process and system information

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Proc entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcEntryType {
    File,
    Directory,
}

/// Proc entry read callback
pub type ProcReadFn = fn(&str) -> Result<Vec<u8>, &'static str>;

/// Proc entry
pub struct ProcEntry {
    pub name: String,
    pub entry_type: ProcEntryType,
    pub read_fn: Option<ProcReadFn>,
}

/// Procfs structure
pub struct Procfs {
    entries: Mutex<BTreeMap<String, ProcEntry>>,
}

impl Procfs {
    /// Create a new procfs
    pub fn new() -> Self {
        let mut procfs = Procfs {
            entries: Mutex::new(BTreeMap::new()),
        };

        // Register default entries
        procfs.register_default_entries();
        procfs
    }

    /// Register default /proc entries
    fn register_default_entries(&mut self) {
        self.register_file("/proc/version", read_version);
        self.register_file("/proc/cpuinfo", read_cpuinfo);
        self.register_file("/proc/meminfo", read_meminfo);
        self.register_file("/proc/uptime", read_uptime);
        self.register_file("/proc/loadavg", read_loadavg);
    }

    /// Register a proc file
    pub fn register_file(&mut self, path: &str, read_fn: ProcReadFn) {
        let mut entries = self.entries.lock();
        entries.insert(
            String::from(path),
            ProcEntry {
                name: String::from(path),
                entry_type: ProcEntryType::File,
                read_fn: Some(read_fn),
            },
        );
    }

    /// Register a proc directory
    pub fn register_directory(&mut self, path: &str) {
        let mut entries = self.entries.lock();
        entries.insert(
            String::from(path),
            ProcEntry {
                name: String::from(path),
                entry_type: ProcEntryType::Directory,
                read_fn: None,
            },
        );
    }

    /// Read from a proc entry
    pub fn read(&self, path: &str) -> Result<Vec<u8>, &'static str> {
        let entries = self.entries.lock();
        if let Some(entry) = entries.get(path) {
            if let Some(read_fn) = entry.read_fn {
                return read_fn(path);
            }
        }
        Err("Entry not found or not readable")
    }

    /// List entries in a directory
    pub fn list(&self, dir: &str) -> Result<Vec<String>, &'static str> {
        let entries = self.entries.lock();
        let prefix = if dir.ends_with('/') {
            String::from(dir)
        } else {
            alloc::format!("{}/", dir)
        };

        let mut results = Vec::new();
        for (path, _) in entries.iter() {
            if path.starts_with(&prefix) {
                let rest = &path[prefix.len()..];
                if !rest.contains('/') {
                    results.push(String::from(rest));
                }
            }
        }

        Ok(results)
    }
}

impl Default for Procfs {
    fn default() -> Self {
        Self::new()
    }
}

/// Read /proc/version
fn read_version(_path: &str) -> Result<Vec<u8>, &'static str> {
    let version = b"Rinux version 0.1.0 (x86_64)\n";
    Ok(version.to_vec())
}

/// Read /proc/cpuinfo
fn read_cpuinfo(_path: &str) -> Result<Vec<u8>, &'static str> {
    // TODO: Get actual CPU information
    let info = b"processor\t: 0\nvendor_id\t: Unknown\nmodel name\t: Unknown CPU\n";
    Ok(info.to_vec())
}

/// Read /proc/meminfo
fn read_meminfo(_path: &str) -> Result<Vec<u8>, &'static str> {
    // TODO: Get actual memory statistics from mm subsystem
    // For now, return placeholder data
    let info = b"MemTotal:       65536 kB\nMemFree:        32768 kB\nMemAvailable:   32768 kB\n";
    Ok(info.to_vec())
}

/// Read /proc/uptime
fn read_uptime(_path: &str) -> Result<Vec<u8>, &'static str> {
    // TODO: Get actual uptime
    let uptime = b"0.00 0.00\n";
    Ok(uptime.to_vec())
}

/// Read /proc/loadavg
fn read_loadavg(_path: &str) -> Result<Vec<u8>, &'static str> {
    // TODO: Calculate actual load average
    let loadavg = b"0.00 0.00 0.00 1/1 1\n";
    Ok(loadavg.to_vec())
}

/// Global procfs instance
static PROCFS: Mutex<Option<Procfs>> = Mutex::new(None);

/// Initialize procfs
pub fn init() {
    let mut fs = PROCFS.lock();
    *fs = Some(Procfs::new());
}

/// Read from procfs
pub fn read(path: &str) -> Result<Vec<u8>, &'static str> {
    let fs = PROCFS.lock();
    if let Some(ref procfs) = *fs {
        procfs.read(path)
    } else {
        Err("Procfs not initialized")
    }
}

/// List procfs directory
pub fn list(dir: &str) -> Result<Vec<String>, &'static str> {
    let fs = PROCFS.lock();
    if let Some(ref procfs) = *fs {
        procfs.list(dir)
    } else {
        Err("Procfs not initialized")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procfs_creation() {
        let procfs = Procfs::new();
        assert!(procfs.read("/proc/version").is_ok());
    }

    #[test]
    fn test_proc_version() {
        let procfs = Procfs::new();
        let data = procfs.read("/proc/version").unwrap();
        assert!(!data.is_empty());
    }
}

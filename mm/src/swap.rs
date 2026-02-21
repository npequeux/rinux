//! Swap Support
//!
//! Manages swapping pages to/from disk when memory is low.

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;
use alloc::collections::VecDeque;

/// Swap statistics
static SWAP_IN_COUNT: AtomicU64 = AtomicU64::new(0);
static SWAP_OUT_COUNT: AtomicU64 = AtomicU64::new(0);
static SWAP_ENABLED: AtomicBool = AtomicBool::new(false);

/// Swap entry - identifies a page on the swap device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapEntry {
    /// Swap device ID
    pub device: u32,
    /// Offset in swap device (in pages)
    pub offset: u64,
}

impl SwapEntry {
    pub const fn new(device: u32, offset: u64) -> Self {
        SwapEntry { device, offset }
    }

    /// Encode swap entry into a page table entry value
    pub fn encode(&self) -> u64 {
        // Swap entries use bits 1-63 (bit 0 is PRESENT=0)
        // Format: [device:8][offset:55]
        ((self.device as u64) << 55) | (self.offset << 1)
    }

    /// Decode swap entry from page table entry value
    pub fn decode(value: u64) -> Self {
        let device = ((value >> 55) & 0xFF) as u32;
        let offset = (value >> 1) & 0x7F_FFFF_FFFF_FFFF;
        SwapEntry { device, offset }
    }

    /// Check if a PTE value is a swap entry
    pub fn is_swap_entry(pte_value: u64) -> bool {
        // Swap entry if not present (bit 0 = 0) and non-zero
        (pte_value & 1) == 0 && pte_value != 0
    }
}

/// Swap device
struct SwapDevice {
    id: u32,
    total_pages: u64,
    used_pages: u64,
    free_list: VecDeque<u64>, // Free page offsets
}

impl SwapDevice {
    fn new(id: u32, size_pages: u64) -> Self {
        let mut free_list = VecDeque::new();
        for i in 0..size_pages {
            free_list.push_back(i);
        }

        SwapDevice {
            id,
            total_pages: size_pages,
            used_pages: 0,
            free_list,
        }
    }

    /// Allocate a swap slot
    fn allocate(&mut self) -> Option<SwapEntry> {
        if let Some(offset) = self.free_list.pop_front() {
            self.used_pages += 1;
            Some(SwapEntry::new(self.id, offset))
        } else {
            None
        }
    }

    /// Free a swap slot
    fn deallocate(&mut self, offset: u64) {
        if offset < self.total_pages {
            self.free_list.push_back(offset);
            self.used_pages = self.used_pages.saturating_sub(1);
        }
    }

    fn free_pages(&self) -> u64 {
        self.total_pages - self.used_pages
    }
}

/// Swap manager
struct SwapManager {
    devices: alloc::vec::Vec<SwapDevice>,
    enabled: bool,
}

impl Default for SwapManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SwapManager {
    const fn new() -> Self {
        SwapManager {
            devices: alloc::vec::Vec::new(),
            enabled: false,
        }
    }

    /// Add a swap device
    fn add_device(&mut self, device_id: u32, size_pages: u64) {
        self.devices.push(SwapDevice::new(device_id, size_pages));
    }

    /// Allocate a swap entry
    fn allocate_swap(&mut self) -> Option<SwapEntry> {
        // Try each device in order
        for device in &mut self.devices {
            if let Some(entry) = device.allocate() {
                return Some(entry);
            }
        }
        None
    }

    /// Free a swap entry
    fn free_swap(&mut self, entry: SwapEntry) {
        if let Some(device) = self.devices.iter_mut().find(|d| d.id == entry.device) {
            device.deallocate(entry.offset);
        }
    }

    /// Get total swap space
    fn total_swap(&self) -> u64 {
        self.devices.iter().map(|d| d.total_pages).sum::<u64>() * 4096
    }

    /// Get free swap space
    fn free_swap_space(&self) -> u64 {
        self.devices.iter().map(|d| d.free_pages()).sum::<u64>() * 4096
    }
}

static SWAP_MANAGER: Mutex<SwapManager> = Mutex::new(SwapManager::new());

/// Initialize swap subsystem
pub fn init() {
    // Swap is disabled by default until a swap device is added
    SWAP_ENABLED.store(false, Ordering::Release);
}

/// Add a swap device
pub fn add_swap_device(device_id: u32, size_bytes: u64) {
    let size_pages = size_bytes / 4096;
    let mut manager = SWAP_MANAGER.lock();
    manager.add_device(device_id, size_pages);
    manager.enabled = true;
    SWAP_ENABLED.store(true, Ordering::Release);
}

/// Check if swap is enabled
pub fn is_enabled() -> bool {
    SWAP_ENABLED.load(Ordering::Acquire)
}

/// Swap out a page to disk
///
/// # Arguments
///
/// * `virt_addr` - Virtual address of page to swap out
/// * `phys_addr` - Physical address of page contents
///
/// # Returns
///
/// SwapEntry on success, or error string
pub fn swap_out(virt_addr: u64, phys_addr: u64) -> Result<SwapEntry, &'static str> {
    if !is_enabled() {
        return Err("Swap not enabled");
    }

    // Allocate a swap slot
    let mut manager = SWAP_MANAGER.lock();
    let entry = manager.allocate_swap().ok_or("No swap space available")?;

    // TODO: Write page to swap device
    // This would involve:
    // 1. Get block device driver for swap device
    // 2. Write 4KB at entry.offset * 4096
    // 3. Wait for I/O completion
    
    // For now, just pretend we wrote it
    let _ = (virt_addr, phys_addr);
    
    SWAP_OUT_COUNT.fetch_add(1, Ordering::SeqCst);
    Ok(entry)
}

/// Swap in a page from disk
///
/// # Arguments
///
/// * `entry` - Swap entry identifying the page
/// * `phys_addr` - Physical address to read page into
///
/// # Returns
///
/// Ok on success, or error string
pub fn swap_in(entry: SwapEntry, phys_addr: u64) -> Result<(), &'static str> {
    if !is_enabled() {
        return Err("Swap not enabled");
    }

    // TODO: Read page from swap device
    // This would involve:
    // 1. Get block device driver for swap device
    // 2. Read 4KB from entry.offset * 4096 into phys_addr
    // 3. Wait for I/O completion
    
    // For now, just pretend we read it
    let _ = (entry, phys_addr);

    // Free the swap slot
    let mut manager = SWAP_MANAGER.lock();
    manager.free_swap(entry);

    SWAP_IN_COUNT.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

/// Get swap statistics
pub fn get_stats() -> (u64, u64, u64, u64) {
    let manager = SWAP_MANAGER.lock();
    (
        manager.total_swap(),
        manager.free_swap_space(),
        SWAP_IN_COUNT.load(Ordering::Acquire),
        SWAP_OUT_COUNT.load(Ordering::Acquire),
    )
}

/// Enable swap
pub fn enable() {
    let mut manager = SWAP_MANAGER.lock();
    if !manager.devices.is_empty() {
        manager.enabled = true;
        SWAP_ENABLED.store(true, Ordering::Release);
    }
}

/// Disable swap
pub fn disable() {
    let mut manager = SWAP_MANAGER.lock();
    manager.enabled = false;
    SWAP_ENABLED.store(false, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_entry_encode_decode() {
        let entry = SwapEntry::new(5, 12345);
        let encoded = entry.encode();
        let decoded = SwapEntry::decode(encoded);
        
        assert_eq!(entry, decoded);
        assert_eq!(decoded.device, 5);
        assert_eq!(decoded.offset, 12345);
    }

    #[test]
    fn test_swap_entry_is_swap() {
        let swap_entry = SwapEntry::new(1, 100).encode();
        assert!(SwapEntry::is_swap_entry(swap_entry));
        
        // Present page (bit 0 = 1) is not a swap entry
        let present_page = 0x1234_5001;
        assert!(!SwapEntry::is_swap_entry(present_page));
        
        // Zero is not a swap entry
        assert!(!SwapEntry::is_swap_entry(0));
    }

    #[test]
    fn test_swap_device_allocation() {
        let mut device = SwapDevice::new(0, 100);
        assert_eq!(device.free_pages(), 100);
        
        let entry = device.allocate().unwrap();
        assert_eq!(device.free_pages(), 99);
        assert_eq!(entry.device, 0);
        
        device.deallocate(entry.offset);
        assert_eq!(device.free_pages(), 100);
    }

    #[test]
    fn test_swap_manager() {
        let mut manager = SwapManager::new();
        manager.add_device(0, 100);
        
        assert_eq!(manager.total_swap(), 100 * 4096);
        assert_eq!(manager.free_swap(), 100 * 4096);
        
        let entry = manager.allocate_swap().unwrap();
        assert_eq!(manager.free_swap(), 99 * 4096);
        
        manager.free_swap(entry);
        assert_eq!(manager.free_swap(), 100 * 4096);
    }
}

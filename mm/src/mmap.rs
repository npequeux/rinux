//! Memory Mapping (mmap)
//!
//! User-space memory mapping implementation.

use crate::frame;
use crate::paging::{PageMapper, VirtAddr, PhysAddr};
use spin::Mutex;
use alloc::collections::BTreeMap;

/// Memory protection flags
pub mod prot {
    /// Page can be read
    pub const PROT_READ: i32 = 0x1;
    /// Page can be written
    pub const PROT_WRITE: i32 = 0x2;
    /// Page can be executed
    pub const PROT_EXEC: i32 = 0x4;
    /// Page cannot be accessed
    pub const PROT_NONE: i32 = 0x0;
}

/// Memory mapping flags
pub mod map {
    /// Share changes
    pub const MAP_SHARED: i32 = 0x01;
    /// Changes are private
    pub const MAP_PRIVATE: i32 = 0x02;
    /// Interpret addr exactly
    pub const MAP_FIXED: i32 = 0x10;
    /// Don't use a file
    pub const MAP_ANONYMOUS: i32 = 0x20;
}

/// Page size constant
const PAGE_SIZE: usize = 4096;

/// Memory mapping region
#[derive(Debug, Clone, Copy)]
struct MappedRegion {
    start: usize,
    size: usize,
    prot: i32,
    flags: i32,
}

/// Memory mapper
struct MemoryMapper {
    regions: BTreeMap<usize, MappedRegion>,
    next_addr: usize,
}

const USER_MMAP_START: usize = 0x0000_1000_0000_0000;
const USER_MMAP_END: usize = 0x0000_7FFF_FFFF_F000;

impl MemoryMapper {
    const fn new() -> Self {
        MemoryMapper {
            regions: BTreeMap::new(),
            next_addr: USER_MMAP_START,
        }
    }

    /// Find a free region of the specified size
    fn find_free_region(&mut self, size: usize) -> Option<usize> {
        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        
        // Try to find a gap in existing mappings
        let mut addr = self.next_addr;
        
        while addr + aligned_size < USER_MMAP_END {
            // Check if this range overlaps with any existing mapping
            let mut overlaps = false;
            for (region_start, region) in self.regions.iter() {
                if addr < region_start + region.size && addr + aligned_size > *region_start {
                    // Overlap found, try after this region
                    addr = region_start + region.size;
                    addr = (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                    overlaps = true;
                    break;
                }
            }
            
            if !overlaps {
                return Some(addr);
            }
        }
        
        None
    }

    /// Map memory region
    fn map(
        &mut self,
        addr: Option<usize>,
        size: usize,
        prot: i32,
        flags: i32,
        _fd: i32,
        _offset: usize,
    ) -> Result<usize, ()> {
        if size == 0 {
            return Err(());
        }

        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        // Find address to use
        let map_addr = if let Some(requested_addr) = addr {
            if (flags & map::MAP_FIXED) != 0 {
                // Must use exact address
                if requested_addr & (PAGE_SIZE - 1) != 0 {
                    return Err(()); // Not page-aligned
                }
                // TODO: Check if range is available or unmap existing
                requested_addr
            } else {
                // Hint - try to use it if available, otherwise find another
                if requested_addr & (PAGE_SIZE - 1) == 0 {
                    // Check if available
                    let mut available = true;
                    for (region_start, region) in self.regions.iter() {
                        if requested_addr < region_start + region.size
                            && requested_addr + aligned_size > *region_start
                        {
                            available = false;
                            break;
                        }
                    }
                    if available {
                        requested_addr
                    } else {
                        self.find_free_region(aligned_size).ok_or(())?
                    }
                } else {
                    self.find_free_region(aligned_size).ok_or(())?
                }
            }
        } else {
            // No hint, find a region
            self.find_free_region(aligned_size).ok_or(())?
        };

        // Allocate physical frames and map them
        let num_pages = aligned_size / PAGE_SIZE;
        let mut mapper = unsafe { PageMapper::new() };

        for i in 0..num_pages {
            let virt_addr = map_addr + i * PAGE_SIZE;
            
            // Allocate physical frame
            let frame = frame::allocate_frame().ok_or(())?;

            // Zero the frame before mapping
            // TODO: This assumes identity mapping or temporary mapping
            unsafe {
                let phys_ptr = frame.start_address() as *mut u8;
                core::ptr::write_bytes(phys_ptr, 0, PAGE_SIZE);
            }

            // Determine page permissions
            let writable = (prot & prot::PROT_WRITE) != 0;
            let user_accessible = true; // User-space mapping
            
            let virt = VirtAddr::new(virt_addr as u64);
            let phys = PhysAddr::new(frame.start_address());
            
            if let Err(_) = mapper.map_page(virt, phys, writable, user_accessible) {
                // Failed to map, clean up already mapped pages
                for j in 0..i {
                    let cleanup_virt = VirtAddr::new((map_addr + j * PAGE_SIZE) as u64);
                    let _ = mapper.unmap_page(cleanup_virt);
                }
                return Err(());
            }
        }

        // Record the mapping
        let region = MappedRegion {
            start: map_addr,
            size: aligned_size,
            prot,
            flags,
        };
        self.regions.insert(map_addr, region);

        // Update next_addr hint
        self.next_addr = map_addr + aligned_size;
        self.next_addr = (self.next_addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        Ok(map_addr)
    }

    /// Unmap memory region
    fn unmap(&mut self, addr: usize, size: usize) -> Result<(), ()> {
        if size == 0 {
            return Err(());
        }

        // Check alignment
        if addr & (PAGE_SIZE - 1) != 0 {
            return Err(());
        }

        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        // Find and remove the region(s)
        // For simplicity, we only handle exact matches for now
        if let Some(region) = self.regions.get(&addr) {
            if region.size == aligned_size {
                // Unmap the pages
                let num_pages = aligned_size / PAGE_SIZE;
                let mut mapper = unsafe { PageMapper::new() };

                for i in 0..num_pages {
                    let virt_addr = addr + i * PAGE_SIZE;
                    let virt = VirtAddr::new(virt_addr as u64);
                    
                    if let Ok(frame) = mapper.unmap_page(virt) {
                        frame::deallocate_frame(frame);
                    }
                }

                // Remove from regions
                self.regions.remove(&addr);
                return Ok(());
            }
        }

        // TODO: Handle partial unmaps and region splitting
        Err(())
    }
}

static MEMORY_MAPPER: Mutex<MemoryMapper> = Mutex::new(MemoryMapper::new());

/// Map memory into user address space
pub fn mmap(
    addr: Option<usize>,
    size: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: usize,
) -> Result<usize, ()> {
    MEMORY_MAPPER.lock().map(addr, size, prot, flags, fd, offset)
}

/// Unmap memory from user address space
pub fn munmap(addr: usize, size: usize) -> Result<(), ()> {
    MEMORY_MAPPER.lock().unmap(addr, size)
}

/// Change protection of memory region
pub fn mprotect(addr: usize, size: usize, prot: i32) -> Result<(), ()> {
    // TODO: Implement mprotect
    let _ = (addr, size, prot);
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_free_region() {
        let mut mapper = MemoryMapper::new();
        let addr = mapper.find_free_region(PAGE_SIZE);
        assert!(addr.is_some());
    }
}

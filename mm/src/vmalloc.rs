//! Virtual Memory Allocator
//!
//! Allocator for kernel virtual memory.
//! vmalloc allocates virtually contiguous memory that may be physically non-contiguous.

use crate::frame;
use spin::Mutex;

/// VM area start address (kernel space)
const VMALLOC_START: usize = 0xFFFF_FFC0_0000_0000;
/// VM area end address
const VMALLOC_END: usize = 0xFFFF_FFE0_0000_0000;
/// Page size
const PAGE_SIZE: usize = 4096;

/// Virtual memory region
#[derive(Debug, Clone, Copy)]
struct VMRegion {
    start: usize,
    size: usize,
    used: bool,
}

/// Virtual memory allocator
struct VMAllocator {
    regions: [VMRegion; MAX_VM_REGIONS],
    region_count: usize,
}

const MAX_VM_REGIONS: usize = 256;

impl VMAllocator {
    const fn new() -> Self {
        const EMPTY_REGION: VMRegion = VMRegion {
            start: 0,
            size: 0,
            used: false,
        };

        VMAllocator {
            regions: [EMPTY_REGION; MAX_VM_REGIONS],
            region_count: 0,
        }
    }

    fn init(&mut self) {
        // Initialize with one large free region
        self.regions[0] = VMRegion {
            start: VMALLOC_START,
            size: VMALLOC_END - VMALLOC_START,
            used: false,
        };
        self.region_count = 1;
    }

    /// Allocate virtual memory
    fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        // Align size to page boundary
        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        // Find a free region large enough
        for i in 0..self.region_count {
            if !self.regions[i].used && self.regions[i].size >= aligned_size {
                let addr = self.regions[i].start;

                // Split the region if necessary
                if self.regions[i].size > aligned_size && self.region_count < MAX_VM_REGIONS {
                    // Create new free region with remaining space
                    let new_start = addr + aligned_size;
                    let new_size = self.regions[i].size - aligned_size;

                    // Shift regions to make space
                    for j in (i + 1..self.region_count).rev() {
                        self.regions[j + 1] = self.regions[j];
                    }

                    self.regions[i + 1] = VMRegion {
                        start: new_start,
                        size: new_size,
                        used: false,
                    };
                    self.region_count += 1;
                }

                // Mark this region as used
                self.regions[i].used = true;
                self.regions[i].size = aligned_size;

                // Allocate physical frames and map them
                if let Err(_) = self.map_region(addr, aligned_size) {
                    // Failed to map, mark as free again
                    self.regions[i].used = false;
                    return None;
                }

                return Some(addr as *mut u8);
            }
        }

        None
    }

    /// Free virtual memory
    fn free(&mut self, ptr: *mut u8) {
        let addr = ptr as usize;

        // Find the region
        for i in 0..self.region_count {
            if self.regions[i].start == addr && self.regions[i].used {
                // Unmap the region
                let _ = self.unmap_region(addr, self.regions[i].size);

                // Mark as free
                self.regions[i].used = false;

                // Try to merge with adjacent free regions
                self.merge_free_regions();
                return;
            }
        }
    }

    /// Map virtual memory region to physical frames
    fn map_region(&mut self, virt_start: usize, size: usize) -> Result<(), ()> {
        let num_pages = size / PAGE_SIZE;

        for i in 0..num_pages {
            let virt_addr = virt_start + i * PAGE_SIZE;

            // Allocate a physical frame
            let frame = frame::allocate_frame().ok_or(())?;

            // Zero the frame
            unsafe {
                let ptr = frame.start_address() as *mut u8;
                core::ptr::write_bytes(ptr, 0, PAGE_SIZE);
            }

            // Map the page
            // TODO: Use proper page table mapping
            // For now, this is a stub
            let _phys_addr = frame.start_address();
            // map_page(virt_addr, phys_addr, true, false)?;
        }

        Ok(())
    }

    /// Unmap virtual memory region
    fn unmap_region(&mut self, virt_start: usize, size: usize) -> Result<(), ()> {
        let num_pages = size / PAGE_SIZE;

        for i in 0..num_pages {
            let virt_addr = virt_start + i * PAGE_SIZE;

            // Unmap the page and free the physical frame
            // TODO: Use proper page table unmapping
            // let phys_addr = unmap_page(virt_addr)?;
            // let frame = Frame::containing_address(phys_addr);
            // frame::deallocate_frame(frame);

            let _ = virt_addr; // Suppress warning
        }

        Ok(())
    }

    /// Merge adjacent free regions
    fn merge_free_regions(&mut self) {
        let mut i = 0;
        while i < self.region_count.saturating_sub(1) {
            if !self.regions[i].used && !self.regions[i + 1].used {
                // Check if regions are adjacent
                if self.regions[i].start + self.regions[i].size == self.regions[i + 1].start {
                    // Merge regions
                    self.regions[i].size += self.regions[i + 1].size;

                    // Shift remaining regions
                    for j in i + 1..self.region_count - 1 {
                        self.regions[j] = self.regions[j + 1];
                    }

                    self.region_count -= 1;
                    continue;
                }
            }
            i += 1;
        }
    }
}

static VM_ALLOCATOR: Mutex<VMAllocator> = Mutex::new(VMAllocator::new());

/// Initialize vmalloc
pub fn init() {
    VM_ALLOCATOR.lock().init();
}

/// Allocate virtual memory
///
/// Allocates virtually contiguous memory that may be physically non-contiguous.
/// This is useful for large allocations where physical contiguity is not required.
pub fn vmalloc(size: usize) -> Option<*mut u8> {
    if size == 0 {
        return None;
    }
    VM_ALLOCATOR.lock().allocate(size)
}

/// Free virtual memory
///
/// # Safety
///
/// The caller must ensure that `ptr` was allocated by vmalloc and is not used afterwards.
pub unsafe fn vfree(ptr: *mut u8) {
    if !ptr.is_null() {
        VM_ALLOCATOR.lock().free(ptr);
    }
}

/// Check if address is in vmalloc range
pub fn is_vmalloc_addr(addr: usize) -> bool {
    addr >= VMALLOC_START && addr < VMALLOC_END
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_vmalloc_addr() {
        assert!(is_vmalloc_addr(VMALLOC_START));
        assert!(is_vmalloc_addr(VMALLOC_START + 0x1000));
        assert!(!is_vmalloc_addr(VMALLOC_START - 1));
        assert!(!is_vmalloc_addr(VMALLOC_END));
    }
}

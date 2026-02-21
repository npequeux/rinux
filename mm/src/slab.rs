//! Slab Allocator
//!
//! A slab allocator for efficient allocation of fixed-size objects.
//! Inspired by the Linux kernel SLUB allocator.

use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr::{null_mut, NonNull};
use spin::Mutex;

/// Size classes for the slab allocator
/// These are common allocation sizes in the kernel
const SIZE_CLASSES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// Number of size classes
const NUM_SIZE_CLASSES: usize = SIZE_CLASSES.len();

/// Maximum size for slab allocation (larger allocations use buddy allocator)
const _MAX_SLAB_SIZE: usize = 4096;

/// Pages per slab
const PAGES_PER_SLAB: usize = 1;

/// Slab size in bytes
const SLAB_SIZE: usize = PAGES_PER_SLAB * 4096;

/// A single object in a slab
#[repr(C)]
struct SlabObject {
    next: Option<NonNull<SlabObject>>,
}

/// A slab contains multiple objects of the same size
struct Slab {
    free_list: Option<NonNull<SlabObject>>,
    object_size: usize,
    num_objects: usize,
    num_free: usize,
}

impl Slab {
    /// Create a new empty slab (not yet allocated)
    const fn new(object_size: usize) -> Self {
        Slab {
            free_list: None,
            object_size,
            num_objects: 0,
            num_free: 0,
        }
    }

    /// Initialize a slab with memory
    ///
    /// # Safety
    ///
    /// The caller must ensure that `memory` points to valid,
    /// aligned memory of at least SLAB_SIZE bytes.
    unsafe fn initialize(&mut self, memory: *mut u8) {
        let object_size = self.object_size.max(mem::size_of::<SlabObject>());
        self.num_objects = SLAB_SIZE / object_size;
        self.num_free = self.num_objects;

        // Build free list
        self.free_list = None;
        for i in (0..self.num_objects).rev() {
            let offset = i * object_size;
            let obj = memory.add(offset) as *mut SlabObject;
            (*obj).next = self.free_list;
            self.free_list = NonNull::new(obj);
        }
    }

    /// Allocate an object from this slab
    fn allocate(&mut self) -> Option<*mut u8> {
        if let Some(mut free_obj) = self.free_list {
            unsafe {
                self.free_list = free_obj.as_mut().next;
                self.num_free -= 1;
                Some(free_obj.as_ptr() as *mut u8)
            }
        } else {
            None
        }
    }

    /// Deallocate an object back to this slab
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` was allocated from this slab.
    unsafe fn deallocate(&mut self, ptr: *mut u8) {
        let obj = ptr as *mut SlabObject;
        (*obj).next = self.free_list;
        self.free_list = NonNull::new(obj);
        self.num_free += 1;
    }

    /// Check if slab is full
    #[allow(dead_code)]
    fn is_full(&self) -> bool {
        self.num_free == 0
    }

    /// Check if slab is empty
    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.num_free == self.num_objects
    }
}

/// Slab allocator
pub struct SlabAllocator {
    size_classes: [Slab; NUM_SIZE_CLASSES],
    fallback: BumpAllocator,
}

impl SlabAllocator {
    /// Create a new slab allocator
    pub const fn new() -> Self {
        // Initialize array with const values matching SIZE_CLASSES
        let size_classes = [
            Slab::new(16),
            Slab::new(32),
            Slab::new(64),
            Slab::new(128),
            Slab::new(256),
            Slab::new(512),
            Slab::new(1024),
            Slab::new(2048),
            Slab::new(4096),
            Slab::new(8192),
        ];

        SlabAllocator {
            size_classes,
            fallback: BumpAllocator::new(),
        }
    }

    /// Initialize the slab allocator
    ///
    /// # Safety
    ///
    /// The caller must ensure that the heap region is valid and not in use.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback.init(heap_start, heap_size);

        // Allocate initial slabs for each size class
        // We pre-allocate all slab memory at once to avoid borrow issues
        let mut slab_memories: [Option<*mut u8>; NUM_SIZE_CLASSES] = [None; NUM_SIZE_CLASSES];
        for i in 0..NUM_SIZE_CLASSES {
            slab_memories[i] = self.allocate_slab_memory();
        }

        // Initialize each slab with its allocated memory
        for (i, slab) in self.size_classes.iter_mut().enumerate() {
            if let Some(memory) = slab_memories[i] {
                slab.initialize(memory);
            }
        }
    }

    /// Allocate memory for a new slab
    fn allocate_slab_memory(&mut self) -> Option<*mut u8> {
        let layout = Layout::from_size_align(SLAB_SIZE, SLAB_SIZE).ok()?;
        let ptr = self.fallback.alloc(layout);
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Find the appropriate size class for a layout
    fn size_class_for(&self, layout: &Layout) -> Option<usize> {
        let size = layout.size().max(layout.align());
        SIZE_CLASSES
            .iter()
            .position(|&class_size| class_size >= size)
    }

    /// Allocate memory
    pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
        // Check if we can use a size class
        if let Some(class_idx) = self.size_class_for(&layout) {
            if let Some(ptr) = self.size_classes[class_idx].allocate() {
                return ptr;
            }

            // Slab is full, try to allocate a new slab
            // For simplicity, we'll just use the fallback for now
            // TODO: In a real implementation, we'd track multiple slabs per size class
            // and initialize them here without double-borrowing
        }

        // Fall back to bump allocator for large allocations
        self.fallback.alloc(layout)
    }

    /// Deallocate memory
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` was allocated by this allocator.
    pub unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        // For now, we only support deallocation for slab-allocated objects
        // Bump allocator doesn't support deallocation
        if let Some(class_idx) = self.size_class_for(&layout) {
            self.size_classes[class_idx].deallocate(ptr);
        }
        // Ignore deallocation for bump-allocated memory
    }
}

/// Simple bump allocator as fallback
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        const HEAP_START: usize = 0xFFFF_FF00_0000_0000;
        const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MB

        BumpAllocator {
            heap_start: HEAP_START,
            heap_end: HEAP_START + HEAP_SIZE,
            next: HEAP_START,
        }
    }

    /// Initialize the allocator
    ///
    /// # Safety
    ///
    /// The caller must ensure that the heap region is valid and not in use.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }

    /// Allocate memory using the bump allocator
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        // Align up the current position
        let alloc_start = (self.next + align - 1) & !(align - 1);
        let alloc_end = alloc_start.saturating_add(size);

        if alloc_end > self.heap_end {
            null_mut()
        } else {
            self.next = alloc_end;
            alloc_start as *mut u8
        }
    }
}

/// Global allocator wrapper
#[allow(dead_code)]
struct LockedSlabAllocator(Mutex<SlabAllocator>);

unsafe impl GlobalAlloc for LockedSlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().allocate(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().deallocate(ptr, layout)
    }
}

// Note: This is commented out because we can't have two global allocators
// Uncomment and replace the one in allocator.rs when ready to switch
// #[global_allocator]
// static ALLOCATOR: LockedSlabAllocator = LockedSlabAllocator(Mutex::new(SlabAllocator::new()));

/// Initialize the slab allocator
pub fn init() {
    // Initialization will be done by the memory subsystem
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_class_selection() {
        let allocator = SlabAllocator::new();

        // Test exact matches
        assert_eq!(
            allocator.size_class_for(&Layout::from_size_align(8, 8).unwrap()),
            Some(0)
        );
        assert_eq!(
            allocator.size_class_for(&Layout::from_size_align(16, 16).unwrap()),
            Some(1)
        );

        // Test sizes that need rounding up
        assert_eq!(
            allocator.size_class_for(&Layout::from_size_align(9, 8).unwrap()),
            Some(1)
        );
        assert_eq!(
            allocator.size_class_for(&Layout::from_size_align(100, 8).unwrap()),
            Some(4)
        );
    }
}

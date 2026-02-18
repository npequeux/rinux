//! Kernel Heap Allocator
//!
//! Global allocator for the kernel heap.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;

/// Heap start address
const HEAP_START: usize = 0xFFFF_FF00_0000_0000;

/// Heap size (1 MB initially)
const HEAP_SIZE: usize = 1024 * 1024;

/// Simple bump allocator for early boot
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: HEAP_START,
            heap_end: HEAP_START + HEAP_SIZE,
            next: HEAP_START,
        }
    }
    
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

/// Align address upwards
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Wrapper around Mutex<BumpAllocator> for the global allocator
struct LockedAllocator(Mutex<BumpAllocator>);

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.0.lock();
        
        let alloc_start = align_up(allocator.next, layout.align());
        let alloc_end = alloc_start.saturating_add(layout.size());
        
        if alloc_end > allocator.heap_end {
            null_mut()
        } else {
            allocator.next = alloc_end;
            alloc_start as *mut u8
        }
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't support deallocation
    }
}

#[global_allocator]
static ALLOCATOR: LockedAllocator = LockedAllocator(Mutex::new(BumpAllocator::new()));

/// Initialize the heap
pub fn init() {
    // The actual heap memory would be allocated during early boot
    // For now, we assume it's available at HEAP_START
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

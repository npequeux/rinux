//! Physical Frame Allocator
//!
//! Manages physical memory frames.

use spin::Mutex;

/// Frame size
pub const FRAME_SIZE: usize = 4096;

/// Physical frame
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Frame {
    number: u64,
}

impl Frame {
    /// Create a frame containing the given address
    pub fn containing_address(addr: u64) -> Frame {
        Frame {
            number: addr / FRAME_SIZE as u64,
        }
    }

    /// Get the start address of the frame
    pub fn start_address(&self) -> u64 {
        self.number * FRAME_SIZE as u64
    }
}

/// Frame allocator
pub struct FrameAllocator {
    next_free_frame: Frame,
    total_frames: u64,
    allocated_frames: u64,
}

impl Default for FrameAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameAllocator {
    pub const fn new() -> Self {
        FrameAllocator {
            next_free_frame: Frame { number: 0 },
            total_frames: 0,
            allocated_frames: 0,
        }
    }

    pub fn init(&mut self, memory_start: u64, memory_end: u64) {
        self.next_free_frame = Frame::containing_address(memory_start);
        self.total_frames = (memory_end - memory_start) / FRAME_SIZE as u64;
        self.allocated_frames = 0;
    }

    /// Allocate a frame
    pub fn allocate_frame(&mut self) -> Option<Frame> {
        if self.allocated_frames >= self.total_frames {
            None
        } else {
            let frame = self.next_free_frame;
            self.next_free_frame.number += 1;
            self.allocated_frames += 1;
            Some(frame)
        }
    }

    /// Deallocate a frame
    pub fn deallocate_frame(&mut self, _frame: Frame) {
        // TODO: Implement proper deallocation
        self.allocated_frames = self.allocated_frames.saturating_sub(1);
    }

    /// Get number of free frames
    pub fn free_frames(&self) -> u64 {
        self.total_frames - self.allocated_frames
    }
}

static FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::new());

/// Initialize frame allocator
pub fn init() {
    let mut allocator = FRAME_ALLOCATOR.lock();
    // Initialize with 128 MB of memory starting at 1 MB
    allocator.init(0x100000, 0x100000 + 128 * 1024 * 1024);
}

/// Allocate a physical frame
pub fn allocate_frame() -> Option<Frame> {
    FRAME_ALLOCATOR.lock().allocate_frame()
}

/// Deallocate a physical frame
pub fn deallocate_frame(frame: Frame) {
    FRAME_ALLOCATOR.lock().deallocate_frame(frame);
}

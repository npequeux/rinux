//! Physical Frame Allocator
//!
//! Manages physical memory frames using a bitmap allocator.

use spin::Mutex;

/// Frame size
pub const FRAME_SIZE: usize = 4096;

/// Maximum number of frames we can track (32 MB worth)
const MAX_FRAMES: usize = 8192;

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

    /// Get frame number
    pub fn number(&self) -> u64 {
        self.number
    }
}

/// Frame allocator with bitmap
pub struct FrameAllocator {
    bitmap: [u64; MAX_FRAMES / 64], // Each u64 tracks 64 frames
    start_frame: u64,
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
            bitmap: [0; MAX_FRAMES / 64],
            start_frame: 0,
            total_frames: 0,
            allocated_frames: 0,
        }
    }

    pub fn init(&mut self, memory_start: u64, memory_end: u64) {
        self.start_frame = memory_start / FRAME_SIZE as u64;
        self.total_frames = (memory_end - memory_start) / FRAME_SIZE as u64;
        self.allocated_frames = 0;

        // Limit to MAX_FRAMES
        if self.total_frames > MAX_FRAMES as u64 {
            // TODO: Log warning about truncation (needs kernel dependency)
            self.total_frames = MAX_FRAMES as u64;
        }

        // Clear bitmap
        for i in 0..self.bitmap.len() {
            self.bitmap[i] = 0;
        }
    }

    /// Check if a frame is allocated
    pub fn is_allocated(&self, frame_number: u64) -> bool {
        if frame_number < self.start_frame {
            return true;
        }

        let index = (frame_number - self.start_frame) as usize;
        if index >= self.total_frames as usize {
            return true;
        }

        let bitmap_index = index / 64;
        let bit_index = index % 64;

        (self.bitmap[bitmap_index] & (1 << bit_index)) != 0
    }

    /// Mark a frame as allocated
    pub fn mark_allocated(&mut self, frame_number: u64) {
        if frame_number < self.start_frame {
            return;
        }

        let index = (frame_number - self.start_frame) as usize;
        if index >= self.total_frames as usize {
            return;
        }

        let bitmap_index = index / 64;
        let bit_index = index % 64;

        if (self.bitmap[bitmap_index] & (1 << bit_index)) == 0 {
            self.bitmap[bitmap_index] |= 1 << bit_index;
            self.allocated_frames += 1;
        }
    }

    /// Mark a frame as free
    fn mark_free(&mut self, frame_number: u64) {
        if frame_number < self.start_frame {
            return;
        }

        let index = (frame_number - self.start_frame) as usize;
        if index >= self.total_frames as usize {
            return;
        }

        let bitmap_index = index / 64;
        let bit_index = index % 64;

        if (self.bitmap[bitmap_index] & (1 << bit_index)) != 0 {
            self.bitmap[bitmap_index] &= !(1 << bit_index);
            self.allocated_frames = self.allocated_frames.saturating_sub(1);
        }
    }

    /// Allocate a frame
    pub fn allocate_frame(&mut self) -> Option<Frame> {
        // Find first free frame
        for i in 0..self.total_frames as usize {
            let bitmap_index = i / 64;
            let bit_index = i % 64;

            if (self.bitmap[bitmap_index] & (1 << bit_index)) == 0 {
                // Found free frame
                self.bitmap[bitmap_index] |= 1 << bit_index;
                self.allocated_frames += 1;
                return Some(Frame {
                    number: self.start_frame + i as u64,
                });
            }
        }

        None
    }

    /// Deallocate a frame
    pub fn deallocate_frame(&mut self, frame: Frame) {
        self.mark_free(frame.number);
    }

    /// Get number of free frames
    pub fn free_frames(&self) -> u64 {
        self.total_frames - self.allocated_frames
    }

    /// Get number of allocated frames
    pub fn allocated_frames(&self) -> u64 {
        self.allocated_frames
    }

    /// Get total frames
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }
}

static FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::new());

/// Initialize frame allocator
pub fn init() {
    let mut allocator = FRAME_ALLOCATOR.lock();
    // Initialize with 32 MB of memory starting at 1 MB
    allocator.init(0x100000, 0x100000 + 32 * 1024 * 1024);
}

/// Allocate a physical frame
pub fn allocate_frame() -> Option<Frame> {
    FRAME_ALLOCATOR.lock().allocate_frame()
}

/// Deallocate a physical frame
pub fn deallocate_frame(frame: Frame) {
    FRAME_ALLOCATOR.lock().deallocate_frame(frame);
}

/// Get memory statistics
pub fn get_stats() -> (u64, u64, u64) {
    let allocator = FRAME_ALLOCATOR.lock();
    (
        allocator.total_frames(),
        allocator.allocated_frames(),
        allocator.free_frames(),
    )
}

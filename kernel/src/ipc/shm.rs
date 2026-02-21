//! Shared Memory
//!
//! Shared memory segments for IPC.

use alloc::vec::Vec;
use spin::Mutex;

/// Shared memory ID type
pub type ShmId = usize;

/// Shared memory segment
pub struct SharedMemorySegment {
    _id: ShmId,
    size: usize,
    data: Vec<u8>,
    attached_count: usize,
}

impl SharedMemorySegment {
    /// Create a new shared memory segment
    pub fn new(id: ShmId, size: usize) -> Self {
        SharedMemorySegment {
            _id: id,
            size,
            data: alloc::vec![0u8; size],
            attached_count: 0,
        }
    }

    /// Get segment size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Attach to segment
    pub fn attach(&mut self) {
        self.attached_count += 1;
    }

    /// Detach from segment
    pub fn detach(&mut self) {
        if self.attached_count > 0 {
            self.attached_count -= 1;
        }
    }

    /// Check if segment is attached
    pub fn is_attached(&self) -> bool {
        self.attached_count > 0
    }

    /// Read from segment
    pub fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, ()> {
        if offset >= self.size {
            return Err(());
        }

        let to_read = buf.len().min(self.size - offset);
        buf[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        Ok(to_read)
    }

    /// Write to segment
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<usize, ()> {
        if offset >= self.size {
            return Err(());
        }

        let to_write = data.len().min(self.size - offset);
        self.data[offset..offset + to_write].copy_from_slice(&data[..to_write]);
        Ok(to_write)
    }
}

/// Global shared memory registry
static SHM_SEGMENTS: Mutex<Vec<Option<SharedMemorySegment>>> = Mutex::new(Vec::new());

/// Initialize shared memory subsystem
pub fn init() {
    let mut segments = SHM_SEGMENTS.lock();
    *segments = Vec::new();
}

/// Create a new shared memory segment
pub fn create_shm(size: usize) -> Result<ShmId, ()> {
    let mut segments = SHM_SEGMENTS.lock();

    // Find empty slot or append
    for (i, slot) in segments.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(SharedMemorySegment::new(i, size));
            return Ok(i);
        }
    }

    let id = segments.len();
    segments.push(Some(SharedMemorySegment::new(id, size)));
    Ok(id)
}

/// Destroy a shared memory segment
pub fn destroy_shm(shm_id: ShmId) -> Result<(), ()> {
    let mut segments = SHM_SEGMENTS.lock();
    if let Some(slot) = segments.get_mut(shm_id) {
        if let Some(seg) = slot {
            if seg.is_attached() {
                return Err(());
            }
        }
        *slot = None;
        Ok(())
    } else {
        Err(())
    }
}

/// Attach to a shared memory segment
pub fn attach_shm(shm_id: ShmId) -> Result<(), ()> {
    let mut segments = SHM_SEGMENTS.lock();
    if let Some(Some(seg)) = segments.get_mut(shm_id) {
        seg.attach();
        Ok(())
    } else {
        Err(())
    }
}

/// Detach from a shared memory segment
pub fn detach_shm(shm_id: ShmId) -> Result<(), ()> {
    let mut segments = SHM_SEGMENTS.lock();
    if let Some(Some(seg)) = segments.get_mut(shm_id) {
        seg.detach();
        Ok(())
    } else {
        Err(())
    }
}

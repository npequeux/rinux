//! Pipe Implementation
//!
//! Unix-style pipes for IPC.

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// Pipe buffer size
const PIPE_BUF_SIZE: usize = 4096;

/// Pipe end type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeEnd {
    /// Read end
    Read,
    /// Write end
    Write,
}

/// Pipe structure
pub struct Pipe {
    buffer: Mutex<VecDeque<u8>>,
    capacity: usize,
    read_closed: AtomicBool,
    write_closed: AtomicBool,
}

impl Default for Pipe {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipe {
    /// Create a new pipe
    pub fn new() -> Self {
        Pipe {
            buffer: Mutex::new(VecDeque::with_capacity(PIPE_BUF_SIZE)),
            capacity: PIPE_BUF_SIZE,
            read_closed: AtomicBool::new(false),
            write_closed: AtomicBool::new(false),
        }
    }

    /// Write data to pipe
    pub fn write(&self, data: &[u8]) -> Result<usize, ()> {
        if self.write_closed.load(Ordering::Acquire) {
            return Err(());
        }

        let mut buffer = self.buffer.lock();
        let mut written = 0;

        for &byte in data {
            if buffer.len() >= self.capacity {
                break;
            }
            buffer.push_back(byte);
            written += 1;
        }

        Ok(written)
    }

    /// Read data from pipe
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, ()> {
        if self.read_closed.load(Ordering::Acquire) {
            return Err(());
        }

        let mut buffer = self.buffer.lock();
        let mut read = 0;

        for slot in buf.iter_mut() {
            if let Some(byte) = buffer.pop_front() {
                *slot = byte;
                read += 1;
            } else {
                break;
            }
        }

        Ok(read)
    }

    /// Close pipe end
    pub fn close(&self, end: PipeEnd) {
        match end {
            PipeEnd::Read => self.read_closed.store(true, Ordering::Release),
            PipeEnd::Write => self.write_closed.store(true, Ordering::Release),
        }
    }

    /// Check if pipe is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().is_empty()
    }

    /// Check if pipe is full
    pub fn is_full(&self) -> bool {
        self.buffer.lock().len() >= self.capacity
    }

    /// Get available data size
    pub fn available(&self) -> usize {
        self.buffer.lock().len()
    }
}

/// Global pipe registry
static PIPES: Mutex<Vec<Option<Pipe>>> = Mutex::new(Vec::new());

/// Initialize pipe subsystem
pub fn init() {
    let mut pipes = PIPES.lock();
    *pipes = Vec::new();
}

/// Create a new pipe and return its ID
pub fn create_pipe() -> Result<usize, ()> {
    let mut pipes = PIPES.lock();
    let pipe = Pipe::new();

    // Find empty slot or append
    for (i, slot) in pipes.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(pipe);
            return Ok(i);
        }
    }

    let id = pipes.len();
    pipes.push(Some(pipe));
    Ok(id)
}

/// Close a pipe
pub fn close_pipe(pipe_id: usize) {
    let mut pipes = PIPES.lock();
    if let Some(slot) = pipes.get_mut(pipe_id) {
        *slot = None;
    }
}

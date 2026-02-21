//! Block Request Queue
//!
//! Manages I/O requests to block devices

use crate::device::BlockDeviceError;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;

/// Block I/O request
#[derive(Debug, Clone)]
pub struct BlockRequest {
    /// Request type
    pub op: BlockOperation,
    /// Starting block number
    pub block: u64,
    /// Number of blocks
    pub count: u32,
    /// Buffer address (physical)
    pub buffer: usize,
    /// Request ID
    pub id: u64,
    /// Request status
    pub status: RequestStatus,
}

/// Block operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockOperation {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Flush operation
    Flush,
}

/// Request status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    /// Request is pending
    Pending,
    /// Request is in progress
    InProgress,
    /// Request completed successfully
    Completed,
    /// Request failed
    Failed(BlockDeviceError),
}

/// Block request queue
pub struct BlockRequestQueue {
    requests: VecDeque<BlockRequest>,
    next_id: u64,
}

impl BlockRequestQueue {
    /// Create a new request queue
    pub fn new() -> Self {
        BlockRequestQueue {
            requests: VecDeque::new(),
            next_id: 1,
        }
    }

    /// Add a request to the queue
    pub fn add_request(&mut self, mut request: BlockRequest) -> u64 {
        request.id = self.next_id;
        request.status = RequestStatus::Pending;
        self.next_id += 1;
        
        self.requests.push_back(request.clone());
        request.id
    }

    /// Get the next pending request
    pub fn next_request(&mut self) -> Option<BlockRequest> {
        self.requests.iter_mut()
            .find(|r| matches!(r.status, RequestStatus::Pending))
            .map(|r| {
                r.status = RequestStatus::InProgress;
                r.clone()
            })
    }

    /// Mark a request as completed
    pub fn complete_request(&mut self, id: u64, result: Result<(), BlockDeviceError>) {
        if let Some(request) = self.requests.iter_mut().find(|r| r.id == id) {
            request.status = match result {
                Ok(()) => RequestStatus::Completed,
                Err(e) => RequestStatus::Failed(e),
            };
        }
    }

    /// Remove completed requests
    pub fn cleanup(&mut self) {
        self.requests.retain(|r| !matches!(r.status, RequestStatus::Completed));
    }

    /// Get number of pending requests
    pub fn pending_count(&self) -> usize {
        self.requests.iter()
            .filter(|r| matches!(r.status, RequestStatus::Pending))
            .count()
    }
}

impl Default for BlockRequestQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_queue() {
        let mut queue = BlockRequestQueue::new();
        
        let request = BlockRequest {
            op: BlockOperation::Read,
            block: 0,
            count: 1,
            buffer: 0,
            id: 0,
            status: RequestStatus::Pending,
        };
        
        let id = queue.add_request(request);
        assert_eq!(id, 1);
        assert_eq!(queue.pending_count(), 1);
        
        let next = queue.next_request();
        assert!(next.is_some());
        assert_eq!(queue.pending_count(), 0);
    }
}

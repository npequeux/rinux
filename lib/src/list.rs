//! Linked List
//!
//! Intrusive linked list implementation.

use core::ptr::NonNull;
use core::marker::PhantomData;

/// List node
pub struct ListNode<T> {
    next: Option<NonNull<ListNode<T>>>,
    prev: Option<NonNull<ListNode<T>>>,
    _marker: PhantomData<T>,
}

impl<T> ListNode<T> {
    pub const fn new() -> Self {
        ListNode {
            next: None,
            prev: None,
            _marker: PhantomData,
        }
    }
}

/// Linked list
pub struct List<T> {
    head: Option<NonNull<ListNode<T>>>,
    tail: Option<NonNull<ListNode<T>>>,
    len: usize,
}

impl<T> List<T> {
    pub const fn new() -> Self {
        List {
            head: None,
            tail: None,
            len: 0,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
}

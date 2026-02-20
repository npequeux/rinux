//! Linked List
//!
//! Intrusive linked list implementation.

use core::marker::PhantomData;
use core::ptr::NonNull;

/// List node
pub struct ListNode<T> {
    #[allow(dead_code)]
    next: Option<NonNull<ListNode<T>>>,
    #[allow(dead_code)]
    prev: Option<NonNull<ListNode<T>>>,
    _marker: PhantomData<T>,
}

impl<T> Default for ListNode<T> {
    fn default() -> Self {
        Self::new()
    }
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
    #[allow(dead_code)]
    head: Option<NonNull<ListNode<T>>>,
    #[allow(dead_code)]
    tail: Option<NonNull<ListNode<T>>>,
    len: usize,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
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

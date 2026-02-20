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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_node_new() {
        let node: ListNode<i32> = ListNode::new();
        assert!(node.next.is_none());
        assert!(node.prev.is_none());
    }

    #[test]
    fn test_list_node_default() {
        let node: ListNode<i32> = ListNode::default();
        assert!(node.next.is_none());
        assert!(node.prev.is_none());
    }

    #[test]
    fn test_list_new() {
        let list: List<i32> = List::new();
<<<<<<< copilot/increase-linux-coverage
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
=======
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
>>>>>>> master
    }

    #[test]
    fn test_list_default() {
        let list: List<i32> = List::default();
<<<<<<< copilot/increase-linux-coverage
        assert!(list.head.is_none());
        assert!(list.tail.is_none());
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
=======
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
>>>>>>> master
    }

    #[test]
    fn test_list_is_empty() {
        let list: List<i32> = List::new();
        assert!(list.is_empty());
<<<<<<< copilot/increase-linux-coverage
        assert_eq!(list.len(), 0);
=======
>>>>>>> master
    }

    #[test]
    fn test_list_len() {
        let list: List<i32> = List::new();
        assert_eq!(list.len(), 0);
    }
<<<<<<< copilot/increase-linux-coverage

    #[test]
    fn test_list_const_new() {
        const LIST: List<i32> = List::new();
        assert_eq!(LIST.len, 0);
    }

    #[test]
    fn test_list_node_const_new() {
        const NODE: ListNode<i32> = ListNode::new();
        // Just ensure const construction works
        let _n = NODE;
    }

    #[test]
    fn test_list_different_types() {
        let list_i32: List<i32> = List::new();
        let list_u64: List<u64> = List::new();
        let list_str: List<&str> = List::new();
        
        assert!(list_i32.is_empty());
        assert!(list_u64.is_empty());
        assert!(list_str.is_empty());
    }
=======
>>>>>>> master
}

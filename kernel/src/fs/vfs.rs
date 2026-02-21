//! Virtual File System
//!
//! VFS layer for abstracting different file systems.

use crate::types::Inode;
use alloc::string::String;
use alloc::vec::Vec;

/// VFS node type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsNodeType {
    /// File node
    File,
    /// Directory node
    Directory,
    /// Device node
    Device,
}

/// VFS node structure
pub struct VfsNode {
    /// Node name
    pub name: String,
    /// Inode number
    pub inode: Inode,
    /// Node type
    pub node_type: VfsNodeType,
    /// Parent inode
    pub parent: Option<Inode>,
    /// Children inodes (for directories)
    pub children: Vec<Inode>,
}

impl VfsNode {
    /// Create a new VFS node
    pub fn new(name: String, inode: Inode, node_type: VfsNodeType) -> Self {
        VfsNode {
            name,
            inode,
            node_type,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Add a child to this node
    pub fn add_child(&mut self, child_inode: Inode) {
        if self.node_type == VfsNodeType::Directory {
            self.children.push(child_inode);
        }
    }

    /// Remove a child from this node
    pub fn remove_child(&mut self, child_inode: Inode) {
        if self.is_directory() {
            self.children.retain(|&inode| inode != child_inode);
        }
    }

    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        self.node_type == VfsNodeType::Directory
    }

    /// Check if this is a file
    pub fn is_file(&self) -> bool {
        self.node_type == VfsNodeType::File
    }
}

/// Initialize VFS subsystem
pub fn init() {
    // TODO: Initialize root filesystem
    // For now, just a placeholder
}

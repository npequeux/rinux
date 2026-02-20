//! Rinux Kernel Library
//!
//! Core kernel data structures and algorithms.

#![no_std]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub use rinux_arch_x86 as arch;
pub use rinux_drivers as drivers;
pub use rinux_kernel as kernel;
pub use rinux_mm as mm;

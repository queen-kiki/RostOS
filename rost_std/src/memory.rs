//! This module contains procedures to change the virtual address space.

use crate::syscall::*;

/// Denotes the maximal address a user process can use.
pub const MAX_ADDRESS: u64 = 0x0000_8000_0000_0000 - 1;

/// This function maps a single page of virtual memory to an arbitrary frame.
pub unsafe fn map_page(virt_addr: u64) {
    syscall!(SYS_VMAP, virt_addr, virt_addr);
}

/// This functions maps a virtual memory range to arbitrary frames.
pub unsafe fn map_range(virt_start: u64, virt_len: u64) {
    syscall!(SYS_VMAP, virt_start, virt_start + virt_len);
}

/// This function maps a virtual page to a specified physical frame.
pub unsafe fn map_to(virt_addr: u64, phys_addr: u64) {
    syscall!(SYS_PMAP, virt_addr, phys_addr);
}

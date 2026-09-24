use core::mem;

use alloc::vec::Vec;
use core::cmp::*;

use crate::disk::{Disk, DiskAddress};

use crate::disk::block::{self, BLOCK_DATA_SIZE, BLOCK_SIZE};

pub const NODE_SIZE: u64 = mem::size_of::<Node>() as u64;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub data_start: DiskAddress,
    pub data_size: u64,
    pub key: i64,
    pub left_child: DiskAddress,
    pub right_child: DiskAddress,
}

impl Node {
    pub const EMPTY: Node = Node {
        data_start: DiskAddress::NULL,
        data_size: 0,
        key: 0,
        left_child: DiskAddress::NULL,
        right_child: DiskAddress::NULL,
    };
}

pub fn clear(disk: &impl Disk, node_addr: DiskAddress) -> Option<()> {
    let node = get_node(disk, node_addr)?;

    node.data_size = 0;
    block::deallocate_data_block(disk, node.data_start);

    Some(())
}

pub fn copy_data(
    disk: &impl Disk,
    node_addr: DiskAddress,
    buffer: &mut Vec<u8>,
) -> Option<()> {
    let node = get_node(disk, node_addr)?;

    if buffer.len() > 0 {
        return None;
    }

    block::copy_from_data_block(disk, node.data_start, buffer, node.data_size)?;

    Some(())
}

pub fn copy_data_len(
    disk: &impl Disk,
    node_addr: DiskAddress,
    buffer: &mut Vec<u8>,
    len: u64,
) -> Option<()> {
    let node = get_node(disk, node_addr)?;

    if buffer.len() > 0 {
        return None;
    }

    if len > node.data_size {
        return None;
    }

    block::copy_from_data_block(disk, node.data_start, buffer, len)?;

    Some(())
}

pub fn copy_data_slice(
    disk: &impl Disk,
    node_addr: DiskAddress,
    buffer: &mut Vec<u8>,
    start: u64,
    end: u64,
) -> Option<()> {
    let node = get_node(disk, node_addr)?;

    if buffer.len() > 0 {
        buffer.clear();
        return None;
    }

    if end > node.data_size {
        return None;
    }

    block::copy_slice_from_data_block(disk, node.data_start, buffer, start, end)?;

    Some(())
}

pub fn write_data(disk: &impl Disk, node_addr: DiskAddress, data: &[u8]) -> Option<()> {
    let node = get_node(disk, node_addr)?;

    if node.data_start.is_null() {
        node.data_start = block::allocate_block(disk)?;
        block::get_data_block(disk, node.data_start)?.next_block = DiskAddress::NULL;
    }

    block::write_to_data_block(disk, node.data_start, data);

    node.data_size = data.len() as u64;

    Some(())
}

pub fn get_node(disk: &impl Disk, address: DiskAddress) -> Option<&mut Node> {
    let block = disk.get_block(address)?;

    let node: *mut Node = &mut block[address.offset()? as usize] as *mut u8 as _;

    unsafe { Some(&mut *node) }
}

pub fn allocate_node(disk: &impl Disk) -> Option<DiskAddress> {
    let address = if !block::get_root_block(disk).free_nodes_start.is_null() {
        let old_head = block::get_root_block(disk).free_nodes_start;
        let next_node = get_node(disk, old_head)?.right_child;
        block::get_root_block(disk).free_nodes_start = next_node;
        old_head
    } else {
        let mut node_block = block::get_root_block(disk).current_node_block;

        if node_block.is_null() {
            node_block = block::allocate_block(disk)?;

            block::get_node_block(disk, node_block)?.used_nodes = 0;
            block::get_node_block(disk, node_block)?.nodes = [Node::EMPTY; 96];
            block::get_root_block(disk).current_node_block = node_block;
        }

        let node = allocate_node_in_block(disk, node_block)?;

        if block::get_node_block(disk, node_block)?.used_nodes >= 96 {
            node_block = DiskAddress::NULL;
            block::get_root_block(disk).current_node_block = node_block;
        }

        node
    };

    *get_node(disk, address)? = Node::EMPTY;

    Some(address)
}

pub fn deallocate_node(disk: &impl Disk, node_addr: DiskAddress) -> Option<()> {
    let old_head = block::get_root_block(disk).free_nodes_start;

    *get_node(disk, node_addr)? = Node {
        right_child: old_head,
        ..Node::EMPTY
    };

    block::get_root_block(disk).free_nodes_start = node_addr;

    Some(())
}

pub fn allocate_node_in_block(disk: &impl Disk, address: DiskAddress) -> Option<DiskAddress> {
    let node_block = block::get_node_block(disk, address)?;
    if node_block.used_nodes < node_block.nodes.len() as u64 {
        let index = node_block.used_nodes;
        node_block.used_nodes += 1;
        Some(DiskAddress::new(
            address.index()?,
            mem::size_of::<u64>() as u64 + NODE_SIZE * index,
        ))
    } else {
        None
    }
}

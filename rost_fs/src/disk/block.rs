use crate::disk::{Disk, DiskAddress};
use crate::node::node::{self, Node};
use core::fmt::Debug;
use core::mem;

use alloc::vec::Vec;

pub const BLOCK_SIZE: u64 = 4096;
pub const MAGIC: u64 = 0xdead_cafe_beef_feed;
pub const BLOCK_DATA_SIZE: usize = BLOCK_SIZE as usize - mem::size_of::<DiskAddress>();

pub type Block = [u8; BLOCK_SIZE as usize];

#[repr(C)]
pub struct RootBlock {
    pub magic: u64,
    pub root_node: DiskAddress,
    pub free_nodes_start: DiskAddress,
    pub free_block_start: DiskAddress, // start of free linked blocks
    pub current_node_block: DiskAddress,
    pub top_block: DiskAddress, // highest block used (all blocks above are free and unlinked)
    pub name: [u8; 128],        // file system name
}

impl RootBlock {
    pub fn init(name: &[u8]) -> RootBlock {
        let mut name_arr = [0; 128];

        for (i, c) in name_arr.iter_mut().enumerate() {
            *c = *name.get(i).unwrap_or(&0);
        }

        RootBlock {
            magic: MAGIC,
            root_node: DiskAddress::NULL,
            free_nodes_start: DiskAddress::NULL,
            free_block_start: DiskAddress::NULL,
            top_block: DiskAddress::block(0),
            current_node_block: DiskAddress::NULL,
            name: name_arr,
        }
    }
}

#[repr(C)]
pub struct DataBlock {
    pub next_block: DiskAddress,
    pub data: [u8; BLOCK_DATA_SIZE],
}

#[repr(C)]
pub struct NodeBlock {
    pub used_nodes: u64,
    pub nodes: [Node; 96],
}

pub fn get_node_block(disk: &impl Disk, address: DiskAddress) -> Option<&mut NodeBlock> {
    unsafe { Some(&mut *(disk.get_block(address)? as *mut Block as *mut NodeBlock)) }
}

pub fn get_data_block(disk: &impl Disk, address: DiskAddress) -> Option<&mut DataBlock> {
    unsafe { Some(&mut *(disk.get_block(address)? as *mut Block as *mut DataBlock)) }
}

pub fn get_root_block(disk: &impl Disk) -> &mut RootBlock {
    unsafe {
        &mut *(disk
            .get_block(DiskAddress::block(0))
            .expect("Root block not found!") as *mut Block as *mut RootBlock)
    }
}

pub fn allocate_block(disk: &impl Disk) -> Option<DiskAddress> {
    let old_head = get_root_block(disk).free_block_start;

    if !old_head.is_null() {
        let next_block = get_data_block(disk, old_head).unwrap().next_block;

        get_root_block(disk).free_block_start = next_block;

        Some(old_head)
    } else {
        if get_root_block(disk).top_block.index()? < disk.block_count() - 1 {
            Some(get_root_block(disk).top_block.inc())
        } else {
            None
        }
    }
}

pub fn deallocate_block(disk: &impl Disk, address: DiskAddress) -> Option<()> {
    let old_head = get_root_block(disk).free_block_start;
    get_data_block(disk, address)?.next_block = old_head;
    get_root_block(disk).free_block_start = address;

    Some(())
}

pub fn data_blocks_required(data_size: u64) -> u64 {
    data_size / BLOCK_DATA_SIZE as u64 + 1
}

pub fn write_to_data_block(
    disk: &impl Disk,
    mut data_block_addr: DiskAddress,
    data: &[u8],
) -> Option<()> {
    for chunk in data.chunks(BLOCK_DATA_SIZE).chain(core::iter::once(&[][..])) {
        let data_block = get_data_block(disk, data_block_addr)?;

        data_block.data[..chunk.len()].copy_from_slice(chunk);

        // A partially filled block is the last one.
        if chunk.len() < BLOCK_DATA_SIZE {
            break;
        }

        if data_block.next_block.is_null() {
            data_block.next_block = allocate_block(disk)?;
            get_data_block(disk, data_block.next_block)?.next_block = DiskAddress::NULL;
        }

        data_block_addr = data_block.next_block;
    }

    Some(())
}

pub fn copy_from_data_block(
    disk: &impl Disk,
    data_block_addr: DiskAddress,
    buffer: &mut Vec<u8>,
    size: u64,
) -> Option<()> {
    copy_slice_from_data_block(disk, data_block_addr, buffer, 0, size)
}

pub fn copy_slice_from_data_block(
    disk: &impl Disk,
    mut data_block_addr: DiskAddress,
    buffer: &mut Vec<u8>,
    start: u64,
    end: u64,
) -> Option<()> {
    let (start, end) = (start as usize, end as usize);

    for block_nr in 0..=end / BLOCK_DATA_SIZE {
        let data_block = get_data_block(disk, data_block_addr)?;

        let block_start = block_nr * BLOCK_DATA_SIZE;
        let block_end = (block_start + BLOCK_DATA_SIZE).min(end);

        if start < block_end {
            buffer.extend_from_slice(&data_block.data[start.max(block_start) - block_start..block_end - block_start]);
        }

        // A partially read block is the last one.
        if block_end - block_start < BLOCK_DATA_SIZE {
            break;
        }

        if data_block.next_block.is_null() {
            buffer.clear();
            return None;
        }

        data_block_addr = data_block.next_block;
    }

    Some(())
}

pub fn deallocate_data_block(
    disk: &impl Disk,
    data_block_addr: DiskAddress,
) -> Option<()> {
    let data_block = get_data_block(disk, data_block_addr)?;
    if !data_block.next_block.is_null() {
        deallocate_data_block(disk, data_block.next_block);
    }
    deallocate_block(disk, data_block_addr);
    Some(())
}

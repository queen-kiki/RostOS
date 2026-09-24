use alloc::collections::BinaryHeap;
use alloc::vec::Vec;

use crate::disk::{self, Disk};

pub mod node;
pub mod tree;

pub type NodeID = i64;

pub trait NodeSet<'disk> {
    type Disk: Disk;
    fn insert_node(&'disk self, key: NodeID) -> NodeResult<Node<'disk, Self::Disk>>;
    fn search_node(&'disk self, key: NodeID) -> NodeResult<Node<'disk, Self::Disk>>;
}

#[derive(Clone, Copy, Debug)]
pub enum NodeError {
    InsertionError,
    SearchError,
    ReadError,
    WriteError,
    InvalidNode,
}

type NodeResult<T> = Result<T, NodeError>;

pub struct NodeTree<'a, D: disk::Disk> {
    disk: &'a mut D,
}

impl<'a, D: disk::Disk> NodeTree<'a, D> {
    pub fn new(disk: &'a mut D) -> NodeTree<'a, D> {
        NodeTree { disk }
    }
}

impl<'a, D: disk::Disk> NodeSet<'a> for NodeTree<'a, D> {
    type Disk = D;

    fn insert_node(&'a self, key: i64) -> NodeResult<Node<'a, D>> {
        if let Some(addr) = tree::insert_node(self.disk, key) {
            Ok(Node {
                disk: self.disk,
                addr,
            })
        } else {
            Err(NodeError::InsertionError)
        }
    }

    fn search_node(&'a self, key: i64) -> NodeResult<Node<'a, D>> {
        if let Some(addr) = tree::search_node(self.disk, key) {
            Ok(Node {
                disk: self.disk,
                addr,
            })
        } else {
            Err(NodeError::SearchError)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Node<'a, D: disk::Disk> {
    disk: &'a D,
    addr: disk::DiskAddress,
}

impl<'a, D: disk::Disk> Node<'a, D> {
    pub fn data_size(&self) -> NodeResult<u64> {
        match node::get_node(self.disk, self.addr) {
            None => Err(NodeError::InvalidNode),
            Some(node) => Ok(node.data_size),
        }
    }

    pub fn read_data(&self, buf: &mut Vec<u8>) -> NodeResult<()> {
        match node::copy_data(self.disk, self.addr, buf) {
            None => Err(NodeError::ReadError),
            Some(()) => Ok(()),
        }
    }

    pub fn read_data_len(&self, buf: &mut Vec<u8>, len: u64) -> NodeResult<()> {
        match node::copy_data_len(self.disk, self.addr, buf, len) {
            None => Err(NodeError::ReadError),
            Some(()) => Ok(()),
        }
    }

    pub fn read_data_slice(&self, buf: &mut Vec<u8>, start: u64, end: u64) -> NodeResult<()> {
        match node::copy_data_slice(self.disk, self.addr, buf, start, end) {
            None => Err(NodeError::ReadError),
            Some(()) => Ok(()),
        }
    }

    pub fn write_data(&self, data: &[u8]) -> NodeResult<()> {
        match node::write_data(self.disk, self.addr, data) {
            None => Err(NodeError::WriteError),
            Some(()) => Ok(()),
        }
    }
}

impl<'a, D: disk::Disk> PartialEq for Node<'a, D> {
    fn eq(&self, other: &Node<'a, D>) -> bool {
        self.addr == other.addr
    }
}

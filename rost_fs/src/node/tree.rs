use core::cmp::Ord;
use core::cmp::Ordering::*;

use super::node::{self, Node};
use crate::disk::*;

pub fn search_node(disk: &impl Disk, key: i64) -> Option<DiskAddress> {
    let mut current_node = block::get_root_block(disk).root_node;

    loop {
        if let Some(node) = node::get_node(disk, current_node) {
            match Ord::cmp(&key, &node.key) {
                Less => {
                    current_node = node.left_child;
                }
                Equal => {
                    break Some(current_node);
                }
                Greater => {
                    current_node = node.right_child;
                }
            }
        } else {
            break None;
        }
    }
}

/// Returns true if insertion occured, false if node already exists
pub fn insert_node(disk: &impl Disk, key: i64) -> Option<DiskAddress> {
    if block::get_root_block(disk).root_node.is_null() {
        let addr = node::allocate_node(disk)?;
        *node::get_node(disk, addr)? = Node { key, ..Node::EMPTY };
        block::get_root_block(disk).root_node = addr;

        return Some(addr);
    }

    let mut current_node = block::get_root_block(disk).root_node;

    loop {
        if let Some(node) = node::get_node(disk, current_node) {
            match Ord::cmp(&key, &node.key) {
                Less => {
                    if node.left_child.is_null() {
                        let addr = node::allocate_node(disk)?;
                        *node::get_node(disk, addr)? = Node { key, ..Node::EMPTY };
                        node.left_child = addr;
                        return Some(addr);
                    } else {
                        current_node = node.left_child;
                    }
                }
                Equal => {
                    return None;
                }
                Greater => {
                    if node.right_child.is_null() {
                        let addr = node::allocate_node(disk)?;
                        *node::get_node(disk, addr)? = Node { key, ..Node::EMPTY };
                        node.right_child = addr;
                        return Some(addr);
                    } else {
                        current_node = node.right_child;
                    }
                }
            }
        } else {
            return None;
        }
    }
}

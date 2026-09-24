use consts::*;
use memory;
use random;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use alloc::string::String;
use alloc::vec::Vec;
use rost_fs::disk::*;
use rost_fs::node::*;
use x86_64::structures::paging::PageTableFlags;

pub use rost_fs::fs::*;

static mut DISK: RamDisk = RamDisk::new_empty();
static NODE_TREE: Once<RwLock<NodeTree<'static, RamDisk>>> = Once::new();

fn create_tree() -> RwLock<NodeTree<'static, RamDisk>> {
    unsafe { RwLock::new(NodeTree::new(&mut *core::ptr::addr_of_mut!(DISK))) }
}

pub fn tree() -> RwLockReadGuard<'static, NodeTree<'static, RamDisk>> {
    NODE_TREE.call_once(create_tree).read()
}

pub fn tree_mut() -> RwLockWriteGuard<'static, NodeTree<'static, RamDisk>> {
    NODE_TREE.call_once(create_tree).write()
}

pub unsafe fn init() {
    memory::map_range(
        RAMDISK_START,
        RAMDISK_START + RAMDISK_SIZE,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
    (*core::ptr::addr_of_mut!(DISK)).init(RAMDISK_START, RAMDISK_SIZE);

    let disk = core::slice::from_raw_parts_mut(RAMDISK_START as *mut u8, RAMDISK_SIZE as _);

    disk.clone_from_slice(::DISK_IMAGE);
}

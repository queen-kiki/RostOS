#![allow(unused)]
#![no_std]
// `Disk::get_block` hands out `&mut Block` from `&self`, which lets callers
// alias blocks. Fixing it means reworking the trait (see todo.md, storage).
#![allow(clippy::mut_from_ref)]

#[macro_use]
extern crate alloc;

pub mod disk;
pub mod fs;
pub mod node;

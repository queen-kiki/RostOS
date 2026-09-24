#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![allow(unused)]

#[macro_use]
extern crate alloc;
extern crate bootloader;
extern crate linked_list_allocator;
extern crate rost_fs;
extern crate spin;
extern crate x86_64;
extern crate xmas_elf;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod vga_buffer;
mod boot_info;
mod consts;
mod elf;
mod fs;
mod gdt;
mod interrupt;
mod memory;
mod panic;
mod process;
mod random;
mod syscall;
mod time;

use core::ptr;

use alloc::string::String;
use alloc::vec::Vec;
use fs::*;
use memory::frame_allocator::FrameStackAllocator;
use x86_64::structures::paging::*;
use x86_64::{PhysAddr, VirtAddr};

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static DISK_IMAGE: &'static [u8] = include_bytes!("../../disk.img");

use consts::*;

core::arch::global_asm!(include_str!("routines.S"));

pub fn io_wait() {
    let mut port = x86_64::instructions::port::Port::new(0x80);
    unsafe {
        port.write(0u8);
    }
}

#[no_mangle]
pub extern "C" fn kernel_init() {
    unsafe {
        // initialize memory management
        memory::init();

        // map vga buffer to high memory
        memory::map_to_address(
            VGA_BUFFER_VADDR,
            VGA_BUFFER_PADDR,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        memory::unmap(VGA_BUFFER_PADDR);

        // map arbitrary frames to heap pages in high memory
        memory::map_range(
            KERNEL_HEAP_START,
            KERNEL_HEAP_START + KERNEL_HEAP_SIZE - 1,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        // initalize heap allocator
        ALLOCATOR
            .lock()
            .init(KERNEL_HEAP_START as *mut u8, KERNEL_HEAP_SIZE as usize);

        time::set_interval(5000);
        gdt::init();
        fs::init();
        syscall::init();
        process::init();
        // intialize interrupts (IDT, PIC)
        interrupt::init();
    }
}

use process::Process;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        // create and schedule the init process
        process::schedule(Process::create(b"/bin/init", vec![b'/']));
        // activate multiprocessing
        process::activate_scheduler();

        // use kernel process as idle spin.
        loop {}
    }
}

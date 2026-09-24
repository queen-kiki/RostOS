#![warn(clippy::all)]
#![no_std]
#![no_main]

#[macro_use]
extern crate rost_std;

use core::sync::atomic::*;
use rost_std::keyboard::{EventKind, KeyEvent, KEY_DOWN, KEY_ESCAPE, KEY_S, KEY_UP, KEY_W};
use rost_std::process;
use rost_std::signal;
use rost_std::vga;
use rost_std::vga::{Color, ColorCode, VGA_HEIGHT, VGA_WIDTH};
use rost_std::port;


#[no_mangle]
pub extern "C" fn _start() {
    unsafe {
        port::write::<u8>(0xf4, 0x00);
    }
    process::idle();
}

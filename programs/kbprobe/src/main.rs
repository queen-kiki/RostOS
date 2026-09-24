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

use rost_std::debug;

use spin::RwLock;

static CODE: RwLock<Option<KeyEvent>> = RwLock::new(None);

extern "C" fn keyboard_handler(scancode: u64, _: u64, _: u64, _: u64) {
    if let Some(event) = KeyEvent::from_scancode(scancode as u8) {
        if let Some(mut code) = CODE.try_write() {
            *code = Some(event)
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    signal::subscribe(signal::SIGNAL_KEYBOARD, keyboard_handler);

    loop {
        {
            if let Some(event) = CODE.write().take() {
                kprintln!("{}", event.keycode());
            }
        }

        process::sleep(1);
    }
}

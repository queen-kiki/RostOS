#![warn(clippy::all)]
#![no_std]
#![no_main]

#[macro_use]
extern crate rost_std;

use rost_std::keyboard::KeyEvent;
use rost_std::process;
use rost_std::signal;


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

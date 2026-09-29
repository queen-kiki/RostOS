#![warn(clippy::all)]
#![no_std]
#![no_main]

extern crate rost_std;

use rost_std::process;
use rost_std::port;


#[no_mangle]
pub extern "C" fn _start() {
    unsafe {
        port::write::<u8>(0xf4, 0x00);
    }
    process::idle();
}

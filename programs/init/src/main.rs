#![no_std]
#![no_main]


#[macro_use]
extern crate rost_std;

use rost_std::process;
use rost_std::signal;
use rost_std::vga;
use rost_std::debug;

use core::sync::atomic::*;


#[no_mangle]
pub extern "C" fn _start() {
    process::execute(b"/bin/logo").unwrap().wait();
    loop {
        process::execute(b"/bin/shell").unwrap().wait();
        kprintln!("Tried to kill init shell!");
    }
}

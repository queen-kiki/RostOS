#![no_std]

use core::panic::PanicInfo;

#[macro_use]
mod syscall;

pub mod ascii;
pub mod debug;
pub mod keyboard;
pub mod memory;
pub mod misc;
pub mod port;
pub mod process;
pub mod signal;
pub mod time;
pub mod vga;

#[panic_handler]
pub fn panic(panic_info: &PanicInfo) -> ! {
    kprintln!("Panic! \n {:?}", panic_info);

    process::exit();
}

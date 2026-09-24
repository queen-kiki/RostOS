//! This module is provided as a means to access the kernel's built-in console. It is meant solely for debugging purposes.

use core::fmt;

use crate::syscall::{self, *};

struct DebugPrinter;

impl fmt::Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        unsafe {
            syscall!(SYS_DEBUG_PRINT, bytes.as_ptr(), bytes.len());
        }
        Ok(())
    }
}

/// Writes a string of ascii bytes to the kernel console.
pub fn write_bytes(bytes: &[u8]) {
    if bytes.len() == 0 {
        return;
    }

    unsafe {
        syscall!(SYS_DEBUG_PRINT, bytes.as_ptr(), bytes.len());
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    let _ = DebugPrinter.write_fmt(args);
}

#[macro_export]
macro_rules! kprintln {
    () => (kprint!("\n"));
    ($($arg:tt)*) => (kprint!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::debug::_print(format_args!($($arg)*)));
}

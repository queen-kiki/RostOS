//! This module provides access to the kernel's signal bus.

use crate::syscall::*;
/// This signal is sent to a process when a key is pressed.
pub const SIGNAL_KEYBOARD: u64 = 1;

/// Subscribes a handler function to the given signal channel. Signal handlers take four 64-bit arguments whose meaning depends on the signal.
pub fn subscribe(channel: u64, handler_func: extern "C" fn(u64, u64, u64, u64)) {
    unsafe {
        syscall!(
            SYS_SUBSCRIBE,
            channel,
            handler_func as *const extern "C" fn(u64, u64, u64, u64)
        );
    }
}

/// Adds a new channel to the signal bus. Returns its id.
pub fn add_channel() -> u64 {
    unsafe { syscall!(SYS_ADD_CHANNEL) }
}

/// Sends a signal on the given channel.
pub fn send(channel: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) {
    unsafe {
        syscall!(SYS_SEND, channel, arg0, arg1, arg2, arg3);
    }
}

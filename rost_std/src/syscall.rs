//! This module contains the system call procedure as well as supported system call ids.

/// This system call prints a message to the kernel's debug console.
pub const SYS_DEBUG_PRINT: u64 = 0x0;

/// This system call returns the current time of the system.
pub const SYS_GET_TIME: u64 = 0x10;

/// This system call registers a handler function with a signal.
pub const SYS_SUBSCRIBE: u64 = 0x20;

/// This system call creates a new signal channel.
pub const SYS_ADD_CHANNEL: u64 = 0x21;

/// This system call sends a message on a channel.
pub const SYS_SEND: u64 = 0x22;

/// This system call returns the process' PID.
pub const SYS_PROCESS_GETPID: u64 = 0x30;

/// This system call exits the program.
pub const SYS_PROCESS_EXIT: u64 = 0x31;

/// This system call executes an elf program.
pub const SYS_PROCESS_EXECUTE: u64 = 0x32;

/// This system call waits on the given PID.
pub const SYS_PROCESS_WAIT: u64 = 0x33;

/// This system call sleeps for the specified amount of ticks.
pub const SYS_PROCESS_SLEEP: u64 = 0x34;

/// This system call kills another process.
pub const SYS_PROCESS_KILL: u64 = 0x35;

/// This system call maps virtual memory to arbitrary frames.
pub const SYS_VMAP: u64 = 0x50;

/// This system call maps virtual memory to specified frames.
pub const SYS_PMAP: u64 = 0x51;

core::arch::global_asm!(
    "
.global _syscall

_syscall:
    int 0x80
    ret
"
);

extern "C" {
    /// This function performs the actual system call.
    pub fn _syscall(_rdi: u64, _rsi: u64, _rdx: u64, _rcx: u64, _r8: u64, _r9: u64) -> u64;
}

#[macro_export]
macro_rules! syscall {
    ($rdi:expr) => {
        $crate::syscall::_syscall($rdi as _, 0, 0, 0, 0, 0)
    };
    ($rdi:expr, $rsi:expr) => {
        $crate::syscall::_syscall($rdi as _, $rsi as _, 0, 0, 0, 0)
    };
    ($rdi:expr, $rsi:expr, $rdx:expr) => {
        $crate::syscall::_syscall($rdi as _, $rsi as _, $rdx as _, 0, 0, 0)
    };
    ($rdi:expr, $rsi:expr, $rdx:expr, $rcx:expr) => {
        $crate::syscall::_syscall($rdi as _, $rsi as _, $rdx as _, $rcx as _, 0, 0)
    };
    ($rdi:expr, $rsi:expr, $rdx:expr, $rcx:expr, $r8:expr) => {
        $crate::syscall::_syscall($rdi as _, $rsi as _, $rdx as _, $rcx as _, $r8 as _, 0)
    };
    ($rdi:expr, $rsi:expr, $rdx:expr, $rcx:expr, $r8:expr, $r9:expr) => {
        $crate::syscall::_syscall(
            $rdi as _, $rsi as _, $rdx as _, $rcx as _, $r8 as _, $r9 as _,
        )
    };
}

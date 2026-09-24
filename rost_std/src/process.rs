///! This module provides access to process control functionality.

use crate::syscall::{self, *};

/// This struct represents a process.
pub struct Process {
    pid: u64,
}

impl Process {
    /// Gets the process of a struct.
    pub fn pid(&self) -> u64 {
        self.pid
    }

    /// Waits for the process to quit. If the process never quits the current process is lost.
    pub fn wait(&self) {
        unsafe {
            syscall!(SYS_PROCESS_WAIT, self.pid);
        }
    }

    /// Kills the process.
    pub fn kill(&self) {
        unsafe {
            syscall!(SYS_PROCESS_KILL, self.pid);
        }
    }
}

/// Gets the current process.
pub fn current() -> Process {
    let pid = unsafe { syscall!(SYS_PROCESS_GETPID) };

    Process { pid }
}

/// Executes an elf file located at `elf_path`, returning its PID if execution was successful
pub fn execute(elf_path: &[u8]) -> Option<Process> {
    let pid = unsafe { syscall!(SYS_PROCESS_EXECUTE, elf_path.as_ptr(), elf_path.len()) };

    if pid == -1 as _ {
        None
    } else {
        Some(Process { pid })
    }
}

/// Sleeps for the specified amount of ticks.
pub fn sleep(ticks: u64) {
    unsafe {
        syscall!(SYS_PROCESS_SLEEP, ticks);
    }
}

/// Exits the current process.
pub fn exit() -> ! {
    unsafe {
        syscall!(SYS_PROCESS_EXIT);
    }

    unreachable!();
}

/// Idles the current process, making only its signal handlers function.
pub fn idle() -> ! {
    unsafe {
        syscall!(SYS_PROCESS_WAIT, 0);
    }

    unreachable!();
}

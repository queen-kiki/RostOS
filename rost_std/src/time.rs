//! This module allows access to the time-keeping functionality of the OS. Time is not kept in real-life units, but in PIT-ticks.

use crate::syscall::*;

/// This struct represents a point in time.
#[derive(Copy, Clone)]
pub struct Time {
    tick_count: u64,
}

impl Time {
    /// Returns the current point in time.
    pub fn current() -> Self {
        Self {
            tick_count: unsafe { syscall!(SYS_GET_TIME) },
        }
    }

    /// Returns the amount of ticks elapsed since the point in time.
    pub fn elapsed(&self) -> u64 {
        Self::current().tick_count - self.tick_count
    }

    /// Returns the tick number of the point in time.
    pub fn tick_nr(&self) -> u64 {
        self.tick_count
    }
}

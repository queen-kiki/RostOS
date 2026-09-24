use core::marker::PhantomData;
use core::ptr;
///! This module provides an abstraction around the legacy VGA buffer. It uses an invisible buffer to quickly store changes and then the `vga::show` function to show them on screen.
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::syscall;
use crate::syscall::*;

use crate::memory;

use crate::port;


/// The width of the VGA buffer
pub const VGA_WIDTH: usize = 80;

/// The height of the VGA buffer
pub const VGA_HEIGHT: usize = 25;

static mut VGA_ADDRESS: *mut u16 = (memory::MAX_ADDRESS + 1 - 0x100_000 + 0xb8000) as _;
static mut VGA_BUFFER: [u16; VGA_WIDTH * VGA_HEIGHT] = [0; VGA_WIDTH * VGA_HEIGHT];

static VGA_MAPPED: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// This struct represents a VGA code.
#[derive(Clone, Copy)]
pub struct ColorCode {
    bg: Color,
    fg: Color,
}

impl ColorCode {
    pub const fn new(bg: Color, fg: Color) -> Self {
        Self { bg, fg }
    }
}

impl Into<u16> for ColorCode {
    fn into(self) -> u16 {
        (self.bg as u8 as u16) << 4 | self.fg as u8 as u16
    }
}

fn check_mapped() {
    if !VGA_MAPPED.load(Ordering::SeqCst) {
        panic!("VGA not mapped!")
    }
}

/// Maps a region of virtual memory to the VGA buffer. Operations writing to the buffer before it is mapped will panic the process.
pub fn map() {
    unsafe {
        if VGA_MAPPED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            memory::map_to(VGA_ADDRESS as _, 0xb8000);
        }
    }
}

/// Writes a single character to the invisible buffer at the given position.
pub fn write_char(x: usize, y: usize, chr: u8, color_code: ColorCode) {
    if x >= VGA_WIDTH || y >= VGA_HEIGHT {
        return;
    }

    unsafe {
        VGA_BUFFER[VGA_WIDTH * y + x] =
            ((Into::<u16>::into(color_code)) << 8) | chr as u16 & 0x00ff;
    }
}

/// Simultaneously writes a single character to the invisible and the visible buffer.
pub fn write_char_thru(x: usize, y: usize, chr: u8, color_code: ColorCode) {
    if x >= VGA_WIDTH || y >= VGA_HEIGHT {
        return;
    }
    
    let val = ((Into::<u16>::into(color_code)) << 8) | chr as u16 & 0x00ff;

    unsafe {
        VGA_BUFFER[VGA_WIDTH * y + x] = val;
        *VGA_ADDRESS.offset((VGA_WIDTH * y + x) as isize) = val;
    }
}

/// Shifts the contents of the invisible buffer up by one row.
pub fn shift_up() {
    for y in 0..VGA_HEIGHT - 1 {
        for x in 0..VGA_WIDTH {
            unsafe { VGA_BUFFER[VGA_WIDTH * y + x] = VGA_BUFFER[VGA_WIDTH * (y + 1) + x] }
        }
    }

    for x in 0..VGA_WIDTH {
        unsafe {
            VGA_BUFFER[VGA_WIDTH * 24 + x] = 0;
        }
    }
}

/// Draws a string onto the buffer.
pub fn draw_string(x: usize, y: usize, string: &[u8], color_code: ColorCode) {
    for (i, &c) in string.iter().enumerate() {
        write_char(x + i, y, c, color_code);
    }
}

/// Sets the hardware cursor to the specified position.
pub fn set_cursor(x: usize, y: usize) {
	let pos = y * VGA_WIDTH + x;
    write_char_thru(x, y, b' ', ColorCode::new(Color::Black, Color::White));
    unsafe {
        port::write(0x3D4, 0x0Au8);
        port::write(0x3D5, (port::read::<u8>(0x3D5) & 0xC0u8));
        port::write(0x3D4, 0x0Bu8);
        port::write(0x3D5, (port::read::<u8>(0x3D5) & 0xE0u8) | 0x00ff);
        port::write(0x3D4, 0x0Fu8);
        port::write(0x3D5, (pos & 0xFF) as u8);
        port::write(0x3D4, 0x0Eu8);
        port::write(0x3D5, ((pos >> 8) & 0xFF) as u8);
    }
}

/// Clears the invisible buffer.
pub fn clear() {
    unsafe {
        let buffer = (&raw mut VGA_BUFFER).cast::<u16>();
        for i in 0..VGA_WIDTH * VGA_HEIGHT {
            buffer.add(i).write(0);
        }
    }
}

/// Copies the invisible buffer to the visible VGA buffer. Will panic if the VGA buffer has not been mapped using `vga::map`.
pub fn show() {
    check_mapped();
    unsafe {
        ptr::copy(
            (&raw mut VGA_BUFFER).cast::<u16>(),
            VGA_ADDRESS,
            VGA_WIDTH * VGA_HEIGHT,
        );
    }
}

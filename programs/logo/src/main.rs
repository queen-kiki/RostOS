#![warn(clippy::all)]
#![no_std]
#![no_main]

extern crate rost_std;

use rost_std::process;
use rost_std::vga;
use rost_std::vga::{Color, ColorCode, VGA_HEIGHT, VGA_WIDTH};

const COLORS: &[Color] = &[
   Color::Red,
   Color::Red,
   Color::LightRed,
   Color::LightRed,
   Color::Yellow,
   Color::Yellow,
   Color::Yellow,
   Color::LightGreen,
   Color::LightGreen,
   Color::Green,
   Color::Green,
   Color::Cyan,
   Color::Cyan,
   Color::Blue,
   Color::Blue,
   Color::LightBlue,
   Color::LightBlue,
   Color::Magenta,
   Color::Magenta,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,
   Color::White,Color::White,
   Color::LightGray,
   Color::DarkGray,
   Color::Black,
];

const PROGRESS : &[&[u8]]= &[b"   .   ", b"   .   ", b"  ...  ", b"  ...  ", b" ..... ", b" ..... ", b"........", b"........"];

#[no_mangle]
pub extern "C" fn _start() {
    vga::map();
    for (radius, (&color, progress)) in (6..).zip(COLORS.iter().zip(PROGRESS.iter().cycle())) {
        vga::clear();
        vga::draw_string(35, 12, b"RostOS v0.1", ColorCode::new(Color::Black, color));
        vga::draw_string(37, 13, progress, ColorCode::new(Color::Black, color));

        for x in 0..VGA_WIDTH {
            for y in 0..VGA_HEIGHT {
                if (x as isize / 2 - 20).abs() + (y as isize - 12).abs() == radius {
                    vga::write_char(x, y, b' ', ColorCode::new(color, color));
                }
            }
        }

        vga::show();
        process::sleep(10);
    }

    for _i in 0..4 {
    
    }


    process::exit();
}

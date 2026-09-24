#![warn(clippy::all)]
#![no_std]
#![no_main]

extern crate rost_std;

use core::sync::atomic::*;
use rost_std::keyboard::{EventKind, KeyEvent, KEY_DOWN, KEY_ESCAPE, KEY_S, KEY_UP, KEY_W};
use rost_std::process;
use rost_std::signal;
use rost_std::vga;
use rost_std::vga::{Color, ColorCode, VGA_HEIGHT, VGA_WIDTH};
use rost_std::misc::itoa;

const PADDLE_HEIGHT: usize = 5;

static P1_AXIS: AtomicIsize = AtomicIsize::new(0);
static P2_AXIS: AtomicIsize = AtomicIsize::new(0);
static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn keyboard_handler(scancode: u64, _: u64, _: u64, _: u64) {
    if let Some(event) = KeyEvent::from_scancode(scancode as u8) {
        match event.kind() {
            EventKind::Press => {
                match event.keycode() {
                    KEY_S => P1_AXIS.store(1, Ordering::SeqCst),
                    KEY_W => P1_AXIS.store(-1, Ordering::SeqCst),
                    KEY_DOWN => P2_AXIS.store(1, Ordering::SeqCst),
                    KEY_UP => P2_AXIS.store(-1, Ordering::SeqCst),
                    KEY_ESCAPE => RUNNING.store(false, Ordering::SeqCst),
                    _ => (),
                };
            }
            EventKind::Release => {
                match event.keycode() {
                    KEY_S => P1_AXIS.store(0, Ordering::SeqCst),
                    KEY_W => P1_AXIS.store(0, Ordering::SeqCst),
                    KEY_DOWN => P2_AXIS.store(0, Ordering::SeqCst),
                    KEY_UP => P2_AXIS.store(0, Ordering::SeqCst),
                    _ => (),
                };
            }
        }
    }
}

struct Vec2 {
    x: isize,
    y: isize,
}

trait GameObject {
    fn render(&self);
    fn update(&mut self);
}

struct Paddle {
    pos: Vec2,
    color: ColorCode,
}

impl GameObject for Paddle {
    fn render(&self) {
        let r = (PADDLE_HEIGHT / 2) as isize;
        for i in -r..=r {
            vga::write_char(self.pos.x as _, (self.pos.y + i) as _, b' ', self.color);
        }
    }

    fn update(&mut self) {
        if self.pos.y < 0 {
            self.pos.y = VGA_HEIGHT as _;
        }
        if self.pos.y > VGA_HEIGHT as _ {
            self.pos.y = 0;
        }
    }
}

pub struct Ball {
    pos: Vec2,
    vel: Vec2,
    color: ColorCode,
}

impl GameObject for Ball {
    fn render(&self) {
        vga::write_char(self.pos.x as _, self.pos.y as _, 254, self.color);
    }

    fn update(&mut self) {
        if !(self.pos.y < VGA_HEIGHT as isize - 1 && self.pos.y > 0) {
            self.vel.y *= -1;
        }

        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    vga::map();
    vga::clear();

    signal::subscribe(signal::SIGNAL_KEYBOARD, keyboard_handler);

    let mut ball = Ball {
        pos: Vec2 {
            x: (VGA_WIDTH / 2) as _,
            y: (VGA_HEIGHT / 2) as _,
        },
        vel: Vec2 { x: 1, y: 1 },
        color: ColorCode::new(Color::Black, Color::White),
    };

    let mut paddle_1 = Paddle {
        pos: Vec2 {
            x: 0,
            y: (VGA_HEIGHT / 2) as _,
        },
        color: ColorCode::new(Color::Blue, Color::Blue),
    };

    let mut paddle_2 = Paddle {
        pos: Vec2 {
            x: (VGA_WIDTH - 1) as _,
            y: (VGA_HEIGHT / 2) as _,
        },
        color: ColorCode::new(Color::Red, Color::Red),
    };

    let mut score_1 = 0;
    let mut score_2 = 0;

    while RUNNING.load(Ordering::SeqCst) {
        ball.update();
        paddle_1.update();
        paddle_2.update();
        
        let r = (PADDLE_HEIGHT / 2) as isize;

        if ball.pos.x <= 0 {
            if (ball.pos.y - paddle_1.pos.y).abs() <= r + 1 {
                ball.vel.x *= -1;
            } else {
                ball.pos.x = (VGA_WIDTH / 2) as _;
                score_2 += 1;
            }
        }

        if ball.pos.x >= (VGA_WIDTH - 1) as _ {
            if (ball.pos.y - paddle_2.pos.y).abs() <= r + 1 {
                ball.vel.x *= -1;
            } else {
                ball.pos.x = (VGA_WIDTH / 2) as _;
                score_1 += 1;
            }
        }

        paddle_1.pos.y += P1_AXIS.load(Ordering::SeqCst);
        paddle_2.pos.y += P2_AXIS.load(Ordering::SeqCst);

        vga::clear();

        ball.render();
        paddle_1.render();
        paddle_2.render();
        let mut s1 = [0; 10];
        vga::draw_string(20,3,itoa(&mut s1, score_1), ColorCode::new(Color::Black, Color::White));
        vga::draw_string(17,3,b"P1", ColorCode::new(Color::Black, Color::Blue));
        let mut s2 = [0; 10];
        vga::draw_string(60,3,itoa(&mut s2, score_2), ColorCode::new(Color::Black, Color::White));
        vga::draw_string(57,3,b"P2", ColorCode::new(Color::Black, Color::Red));

        for x in 2..(VGA_WIDTH - 2) {
            vga::write_char(x, 0, b'-', ColorCode::new(Color::Black, Color::White));
            vga::write_char(x, 24, b'-', ColorCode::new(Color::Black, Color::White));
        }

        vga::show();

        process::sleep(15);
    }
    process::exit();
}

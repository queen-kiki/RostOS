//! A Kármán vortex street: flow past a cylinder, simulated with the lattice
//! Boltzmann method (D2Q9, BGK collisions).
//!
//! The target has no FPU support, so everything runs in Q24 fixed point. The
//! lattice is oversampled 2x2 relative to the screen, which shows it as 80x48
//! "pixels" using half-block characters.
//!
//! SPACE switches between vorticity and speed, ESC quits.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use rost_std::keyboard::{EventKind, KeyEvent, KEY_ESCAPE, KEY_SPACE};
use rost_std::misc::itoa;
use rost_std::vga::{self, Color, ColorCode, VGA_HEIGHT, VGA_WIDTH};
use rost_std::{port, process, signal};

/// Lattice cells per screen pixel along each axis.
const OVERSAMPLE: usize = 2;
/// Screen pixels: one text row holds two, the last row is the status line.
const PX: usize = VGA_WIDTH;
const PY: usize = (VGA_HEIGHT - 1) * 2;
/// Lattice size.
const NX: usize = PX * OVERSAMPLE;
const NY: usize = PY * OVERSAMPLE;
const N: usize = NX * NY;

/// Fixed point one (Q24).
const ONE: i64 = 1 << 24;

/// Inflow velocity, in lattice units.
const U0: i64 = ONE / 10;
/// Cylinder radius and center, slightly off-axis so that shedding starts.
const RADIUS: usize = 8;
const CX: usize = NX / 5;
const CY: usize = NY / 2 - 1;
/// Kinematic viscosity 0.017: Re = U0 * 2 * RADIUS / nu ≈ 100.
/// The BGK relaxation rate is omega = 1 / (3 nu + 0.5) = 1 / 0.551.
const OMEGA: i64 = ONE * 1000 / 551;
const REYNOLDS: &[u8] = b"100";

/// Simulation steps per rendered frame.
const STEPS_PER_FRAME: usize = 10;

/// D2Q9 velocities, weights (in 36ths) and the opposite of each direction.
const Q: usize = 9;
const EX: [i64; Q] = [0, 1, 0, -1, 0, 1, -1, -1, 1];
const EY: [i64; Q] = [0, 0, 1, 0, -1, 1, 1, -1, -1];
const W: [i64; Q] = [16, 4, 4, 4, 4, 1, 1, 1, 1];
const OPP: [usize; Q] = [0, 3, 4, 1, 2, 7, 8, 5, 6];

/// Upper half block in code page 437: the foreground colors the top half.
const HALF_BLOCK: u8 = 0xDF;

static RUNNING: AtomicBool = AtomicBool::new(true);
static SHOW_SPEED: AtomicBool = AtomicBool::new(false);

extern "C" fn keyboard_handler(scancode: u64, _: u64, _: u64, _: u64) {
    if let Some(event) = KeyEvent::from_scancode(scancode as u8) {
        if event.kind() == EventKind::Press {
            match event.keycode() {
                KEY_ESCAPE => RUNNING.store(false, Ordering::SeqCst),
                KEY_SPACE => {
                    SHOW_SPEED.fetch_xor(true, Ordering::SeqCst);
                }
                _ => (),
            }
        }
    }
}

struct Lattice {
    /// Particle distributions, double buffered for streaming.
    f: [[[i32; Q]; N]; 2],
    cur: usize,
    ux: [i32; N],
    uy: [i32; N],
    solid: [bool; N],
}

// ~1.2 MiB, far too big for the stack of the thread that initializes it.
static mut LATTICE: Lattice = Lattice {
    f: [[[0; Q]; N]; 2],
    cur: 0,
    ux: [0; N],
    uy: [0; N],
    solid: [false; N],
};

fn idx(x: usize, y: usize) -> usize {
    y * NX + x
}

/// The equilibrium distribution for density `rho` and velocity `(ux, uy)`.
fn equilibrium(rho: i64, ux: i64, uy: i64) -> [i32; Q] {
    let u2 = (ux * ux + uy * uy) >> 24;
    let mut feq = [0; Q];

    for i in 0..Q {
        let eu = EX[i] * ux + EY[i] * uy;
        // 1 + 3 e.u + 9/2 (e.u)^2 - 3/2 u^2
        let t = ONE + 3 * eu + ((9 * eu * eu) >> 25) - ((3 * u2) >> 1);
        feq[i] = (((rho * t) >> 24) * W[i] / 36) as i32;
    }

    feq
}

impl Lattice {
    fn init(&mut self) {
        let inflow = equilibrium(ONE, U0, 0);
        // A sideways kick behind the cylinder breaks the symmetry, so the
        // vortices start shedding within a few periods rather than eventually.
        let kicked = equilibrium(ONE, U0, U0 / 4);

        for y in 0..NY {
            for x in 0..NX {
                let (dx, dy) = (x as isize - CX as isize, y as isize - CY as isize);
                let i = idx(x, y);

                self.solid[i] = dx * dx + dy * dy <= (RADIUS * RADIUS) as isize;
                self.f[0][i] = if dx > 0 && dx < 4 * RADIUS as isize && dy.abs() < RADIUS as isize {
                    kicked
                } else {
                    inflow
                };
                self.ux[i] = 0;
                self.uy[i] = 0;
            }
        }

        self.cur = 0;
    }

    fn step(&mut self) {
        let (src, dst) = (self.cur, 1 - self.cur);

        // Collide.
        for i in 0..N {
            if self.solid[i] {
                continue;
            }

            let f = &mut self.f[src][i];
            let (mut rho, mut jx, mut jy) = (0, 0, 0);
            for q in 0..Q {
                let fq = f[q] as i64;
                rho += fq;
                jx += EX[q] * fq;
                jy += EY[q] * fq;
            }

            let ux = (jx << 24) / rho;
            let uy = (jy << 24) / rho;
            self.ux[i] = ux as i32;
            self.uy[i] = uy as i32;

            let feq = equilibrium(rho, ux, uy);
            for q in 0..Q {
                f[q] += (((feq[q] - f[q]) as i64 * OMEGA) >> 24) as i32;
            }
        }

        // Stream: periodic top to bottom, bounce back off the cylinder.
        for y in 0..NY {
            for x in 0..NX {
                let i = idx(x, y);
                if self.solid[i] {
                    continue;
                }

                for q in 0..Q {
                    let nx = x as i64 + EX[q];
                    if nx < 0 || nx >= NX as i64 {
                        continue;
                    }
                    let ny = (y as i64 + EY[q]).rem_euclid(NY as i64);
                    let j = idx(nx as usize, ny as usize);

                    if self.solid[j] {
                        self.f[dst][i][OPP[q]] = self.f[src][i][q];
                    } else {
                        self.f[dst][j][q] = self.f[src][i][q];
                    }
                }
            }
        }

        // Fixed inflow on the left, zero-gradient outflow on the right.
        let inflow = equilibrium(ONE, U0, 0);
        for y in 0..NY {
            self.f[dst][idx(0, y)] = inflow;
            self.f[dst][idx(NX - 1, y)] = self.f[dst][idx(NX - 2, y)];
        }

        self.cur = dst;
    }

    /// Vorticity at a lattice cell, by central differences.
    fn curl(&self, x: usize, y: usize) -> i64 {
        let (xl, xr) = (x.saturating_sub(1), (x + 1).min(NX - 1));
        let (yu, yd) = ((y + NY - 1) % NY, (y + 1) % NY);

        (self.uy[idx(xr, y)] - self.uy[idx(xl, y)]) as i64
            - (self.ux[idx(x, yd)] - self.ux[idx(x, yu)]) as i64
    }

    fn speed2(&self, i: usize) -> i64 {
        let (ux, uy) = (self.ux[i] as i64, self.uy[i] as i64);
        (ux * ux + uy * uy) >> 24
    }

    /// The color of screen pixel `(px, py)`, averaged over its lattice cells.
    fn pixel(&self, px: usize, py: usize, show_speed: bool) -> Color {
        let (mut sum, mut solid) = (0, 0);

        for y in py * OVERSAMPLE..(py + 1) * OVERSAMPLE {
            for x in px * OVERSAMPLE..(px + 1) * OVERSAMPLE {
                let i = idx(x, y);
                if self.solid[i] {
                    solid += 1;
                } else if show_speed {
                    sum += self.speed2(i);
                } else {
                    sum += self.curl(x, y);
                }
            }
        }

        if solid * 2 >= OVERSAMPLE * OVERSAMPLE {
            return Color::White;
        }
        let avg = sum / (OVERSAMPLE * OVERSAMPLE) as i64;

        if show_speed {
            speed_color(avg)
        } else {
            vorticity_color(avg)
        }
    }
}

/// Diverging palette: blue for clockwise, red for counter-clockwise rotation.
fn vorticity_color(curl: i64) -> Color {
    const SAT: i64 = U0 / 2;
    const LEVELS: [Color; 7] = [
        Color::LightCyan,
        Color::LightBlue,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::LightRed,
        Color::Yellow,
    ];

    let level = match curl.abs() {
        c if c < SAT / 8 => 0,
        c if c < SAT / 3 => 1,
        c if c < SAT => 2,
        _ => 3,
    };
    LEVELS[(3 + curl.signum() * level) as usize]
}

/// Sequential palette over |u|^2, up to twice the inflow speed.
fn speed_color(speed2: i64) -> Color {
    const LEVELS: [Color; 8] = [
        Color::Black,
        Color::Blue,
        Color::LightBlue,
        Color::Cyan,
        Color::LightGreen,
        Color::Yellow,
        Color::LightRed,
        Color::Red,
    ];

    let max = ((2 * U0) * (2 * U0)) >> 24;
    let level = (speed2 * LEVELS.len() as i64 / max) as usize;
    LEVELS[level.min(LEVELS.len() - 1)]
}

/// Bit 3 of the attribute mode control register selects blinking over bright
/// background colors, which the half-block rendering needs.
fn set_blink(enabled: bool) {
    unsafe {
        // Reading the input status register resets the attribute controller's
        // index/data flip-flop; 0x20 keeps the display enabled.
        port::read::<u8>(0x3DA);
        port::write(0x3C0, 0x10u8 | 0x20);
        let mode = port::read::<u8>(0x3C1);
        port::write(0x3C0, if enabled { mode | 0x08 } else { mode & !0x08 });
    }
}

fn draw(lattice: &Lattice, step: usize) {
    let show_speed = SHOW_SPEED.load(Ordering::SeqCst);

    for row in 0..VGA_HEIGHT - 1 {
        for px in 0..PX {
            let top = lattice.pixel(px, 2 * row, show_speed);
            let bottom = lattice.pixel(px, 2 * row + 1, show_speed);
            vga::write_char(px, row, HALF_BLOCK, ColorCode::new(bottom, top));
        }
    }

    let status = ColorCode::new(Color::Black, Color::LightGray);
    let highlight = ColorCode::new(Color::Black, Color::White);
    for x in 0..VGA_WIDTH {
        vga::write_char(x, VGA_HEIGHT - 1, b' ', status);
    }

    let mut x = 1;
    let mut text = |s: &[u8], color| {
        vga::draw_string(x, VGA_HEIGHT - 1, s, color);
        x += s.len();
    };
    let mut digits = [0; 12];

    // 0xA0 is an a with an acute accent in code page 437.
    text(b"K\xA0rm\xA0n vortex street", highlight);
    text(b"  Re ", status);
    text(REYNOLDS, status);
    text(b"  step ", status);
    text(itoa(&mut digits, step as i32), highlight);
    text(if show_speed { b"  speed    " } else { b"  vorticity" }, status);
    text(b"  SPACE view  ESC quit", status);

    vga::show();
}

#[no_mangle]
pub extern "C" fn _start() {
    let lattice = unsafe { &mut *core::ptr::addr_of_mut!(LATTICE) };

    vga::map();
    set_blink(false);
    signal::subscribe(signal::SIGNAL_KEYBOARD, keyboard_handler);

    lattice.init();

    let mut step = 0;
    while RUNNING.load(Ordering::SeqCst) {
        for _ in 0..STEPS_PER_FRAME {
            lattice.step();
        }
        step += STEPS_PER_FRAME;
        draw(lattice, step);
    }

    set_blink(true);
    vga::clear();
    vga::show();
    process::exit();
}

#![no_std]
#![no_main]

extern crate rost_std;

use rost_std::vga;
use rost_std::vga::{ColorCode, Color, VGA_WIDTH};
use rost_std::signal;
use rost_std::keyboard;
use rost_std::keyboard::{KeyEvent, EventKind, KeyCase};
use rost_std::process;
use rost_std::process::Process;
use rost_std::misc::atoi;

use rost_std::ascii::BACKSPACE;

use core::sync::atomic::*;

use spin::Mutex;

const COLOR : ColorCode = ColorCode::new(Color::Black, Color::White);

static TERMINAL_BUFFER: TerminalBuf = TerminalBuf::new();
static SHIFT: AtomicBool = AtomicBool::new(false);

extern "C" fn keyboard_handler(scancode: u64, _: u64, _:u64,_:u64) {
    
    if let Some(event) = KeyEvent::from_scancode(scancode as _) {

       if event.keycode() == keyboard::KEY_LEFT_SHIFT || event.keycode() == keyboard::KEY_RIGHT_SHIFT {
            match event.kind() {
                EventKind::Press => SHIFT.fetch_or(true, Ordering::SeqCst),
                EventKind::Release => SHIFT.fetch_and(false, Ordering::SeqCst),
            };
        } else if event.kind() == EventKind::Press {
            match event.keycode() {
                keyboard::KEY_ENTER => {
                    if TERMINAL_BUFFER.cursor.load(Ordering::SeqCst) <= TERMINAL_BUFFER.read_start.load(Ordering::SeqCst) {
                        TERMINAL_BUFFER.input_char(b' ');
                        TERMINAL_BUFFER.del_char();
                    }
                    TERMINAL_BUFFER.new_line()
                },
                keyboard::KEY_BACKSPACE => TERMINAL_BUFFER.del_char(),
                _c =>  {
                    let c = event.get_ascii(KeyCase::new(SHIFT.load(Ordering::SeqCst)));
                    TERMINAL_BUFFER.input_char(c);
                }
            } 
        }
    }
    update_cursor();
}

pub struct TerminalBuf {
    line: Mutex<[u8; 80]>,
    buffer: Mutex<[u8; 80]>,
    reading: AtomicBool,
    read_start: AtomicUsize,
    cursor: AtomicUsize,
    buffer_ready: AtomicBool,
    buffer_len: AtomicUsize,
}

impl TerminalBuf {
    pub const fn new()-> Self {
        Self {
            line: Mutex::new([0;80]),
            buffer: Mutex::new([0;80]),
            reading: AtomicBool::new(false),
            read_start: AtomicUsize::new(0),
            cursor: AtomicUsize::new(0),
            buffer_ready: AtomicBool::new(false),
            buffer_len: AtomicUsize::new(0),
        }
    }

    pub fn print_char(&self, c: u8) {
        let cursor = self.cursor.fetch_add(1, Ordering::SeqCst); 

        vga::write_char(cursor, 24, c, COLOR);
        vga::show();
        if self.cursor.load(Ordering::SeqCst) >= VGA_WIDTH {
            self.new_line();
        }
    }

    pub fn del_char(&self) {        
        if self.cursor.load(Ordering::SeqCst) > self.read_start.load(Ordering::SeqCst) {
            let cursor = self.cursor.fetch_sub(1, Ordering::SeqCst) - 1;
            (*self.line.lock())[cursor] = b' ';
            vga::write_char(cursor, 24, b' ', COLOR);
            vga::show();
        }
    }

    pub fn print_ascii(&self, s: &[u8]) {
        for byte in s {
            match *byte {
                b'\n' => self.new_line(),
                BACKSPACE => self.del_char(),
                b => self.print_char(b),
            }
        }
    }

    pub fn input_char(&self, c: u8) {
        if self.buffer_ready.load(Ordering::SeqCst) {
            return;
        }

        let cursor = self.cursor.fetch_add(1, Ordering::SeqCst); 
        {
            let mut line = self.line.lock();

            (*line)[cursor] = c;
        }
        vga::write_char(cursor, 24, c, COLOR);
        vga::show();


        if !self.reading.fetch_or(true, Ordering::SeqCst) {
            self.read_start.store(cursor, Ordering::SeqCst);
        }

        if self.cursor.load(Ordering::SeqCst) >= VGA_WIDTH {
            self.new_line();
        }
    }

    pub fn new_line(&self) {
        if !self.buffer_ready.load(Ordering::SeqCst) && self.reading.fetch_and(false, Ordering::SeqCst) {
            self.buffer_len.store(self.cursor.load(Ordering::SeqCst) - self.read_start.load(Ordering::SeqCst), Ordering::SeqCst);
            (self.buffer.lock())[..VGA_WIDTH - self.read_start.load(Ordering::SeqCst)].copy_from_slice(&(*self.line.lock())[self.read_start.load(Ordering::SeqCst)..]);
            self.buffer_ready.store(true, Ordering::SeqCst);
            self.read_start.store(0, Ordering::SeqCst)
        }

        self.cursor.store(0, Ordering::SeqCst);
        vga::shift_up();
        vga::show();
    }

    pub fn get_line(&self, buf: &mut [u8]) -> Option<usize> {
        if self
            .buffer_ready
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            buf.copy_from_slice(&(*self.buffer.lock())[0..buf.len()]);
            Some(self.buffer_len.load(Ordering::SeqCst))
        } else {
            None
        }
    }
}

impl Default for TerminalBuf {
    fn default() -> Self {
        Self::new()
    }
}

enum ShellError {
    CommandNotFound,
}

type ShellResult<T> = Result<T, ShellError>;

fn execute_command(cmd: &[u8]) -> ShellResult<Process> {
    let mut line = [0; 84];
    
    line[..4].copy_from_slice(b"bin/");
    line[4..cmd.len() + 4].copy_from_slice(cmd);

    process::execute(&line[0..cmd.len() + 4]).ok_or(ShellError::CommandNotFound)
}

fn poll_line(line: &mut [u8]) -> &[u8] {
    loop {
        if let Some(len) = TERMINAL_BUFFER.get_line(line) { break &line[0..len] }
    }
}

fn update_cursor() {
    vga::set_cursor(TERMINAL_BUFFER.cursor.load(Ordering::SeqCst).min(VGA_WIDTH - 1), 24);
}

fn clear() {
    TERMINAL_BUFFER.cursor.store(0, Ordering::SeqCst);
    vga::clear();
    update_cursor();
}

fn prompt() {
    TERMINAL_BUFFER.print_ascii(b"> ");
    update_cursor();
}

#[no_mangle]
pub extern "C" fn _start() {
    vga::map();
    signal::subscribe(signal::SIGNAL_KEYBOARD, keyboard_handler);
    loop {
        prompt();
        let mut line = [0; 80];
        let line = poll_line(&mut line);

        let split = line.iter().position(|b| *b==b' ');

        let mut cmd : &[u8];
        let arg = if let Some(split) = split {
            cmd = &line[..split];
            Some(&line[split + 1..])
        } else {
            cmd = line;
            None
        };

        let mut bg = false;

        if cmd.len() > 1 && *cmd.first().unwrap() == b'&' {
            bg = true;
            cmd = &cmd[1..]
        }

        match cmd {
            b"exit" => process::exit(),
            b"clear" => clear(),
            b"sleep" => {
                if let Some(arg) = arg {
                    process::sleep(atoi(arg) as u64);
                } else {
                    TERMINAL_BUFFER.print_ascii(b"Command requires argument!");
                } 
            },
            _ => match execute_command(cmd) {
                Ok(proc) => {
                    if !bg {
                         proc.wait()
                    }
                },
                Err(ShellError::CommandNotFound) => {
                    TERMINAL_BUFFER.print_ascii(b"Command not found: ");
                    TERMINAL_BUFFER.print_ascii(cmd);
                    TERMINAL_BUFFER.new_line();  
                },
            },
        }
    }
    
}

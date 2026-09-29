//! This module contains procedures to interpret the data supplied by the keyboard signal.
use crate::ascii;

const KEYMAP_LOWER: [u8; 87] = [
    0,
    ascii::ESCAPE,
    b'1',
    b'2',
    b'3',
    b'4',
    b'5',
    b'6',
    b'7',
    b'8',
    b'9',
    b'0',
    b'\'',
    b'^',
    ascii::BACKSPACE,
    b'\t',
    b'q',
    b'w',
    b'e',
    b'r',
    b't',
    b'z',
    b'u',
    b'i',
    b'o',
    b'p',
    b'[',
    b']',
    b'\n',
    0,
    b'a',
    b's',
    b'd',
    b'f',
    b'g',
    b'h',
    b'j',
    b'k',
    b'l',
    0,
    b'{',
    0,
    0,
    b'$',
    b'y',
    b'x',
    b'c',
    b'v',
    b'b',
    b'n',
    b'm',
    b',',
    b'.',
    b'-',
    0,
    0,
    0,
    b' ',
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

const KEYMAP_UPPER: [u8; 87] = [
    0, 0, b'+', b'"', b'*', 0, b'%', b'&', b'/', b'(', b')', b'=', b'?', b'`', 0, b'\t', b'Q',
    b'W', b'E', b'R', b'T', b'Z', b'U', b'I', b'O', b'P', 0, b'!', b'\n', 0, b'A', b'S', b'D',
    b'F', b'G', b'H', b'J', b'K', b'L', 0, b'{', 0, 0, b'}', b'Y', b'X', b'C', b'V', b'B', b'N',
    b'M', b';', b':', b'_', 0, 0, 0, b' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const PRESS_START: u8 = 0;
const PRESS_END: u8 = PRESS_START + 86;

const RELEASE_START: u8 = 0x80;
const RELEASE_END: u8 = RELEASE_START + 86;

pub use self::scancodes::*;

mod scancodes {
    pub const KEY_A: u8 = 0x1E;
    pub const KEY_B: u8 = 0x30;
    pub const KEY_C: u8 = 0x2E;
    pub const KEY_D: u8 = 0x20;
    pub const KEY_E: u8 = 0x12;
    pub const KEY_F: u8 = 0x21;
    pub const KEY_G: u8 = 0x22;
    pub const KEY_H: u8 = 0x23;
    pub const KEY_I: u8 = 0x17;
    pub const KEY_J: u8 = 0x24;
    pub const KEY_K: u8 = 0x25;
    pub const KEY_L: u8 = 0x26;
    pub const KEY_M: u8 = 0x32;
    pub const KEY_N: u8 = 0x31;
    pub const KEY_O: u8 = 0x18;
    pub const KEY_P: u8 = 0x19;
    pub const KEY_Q: u8 = 0x10;
    pub const KEY_R: u8 = 0x13;
    pub const KEY_S: u8 = 0x1F;
    pub const KEY_T: u8 = 0x14;
    pub const KEY_U: u8 = 0x16;
    pub const KEY_V: u8 = 0x2F;
    pub const KEY_W: u8 = 0x11;
    pub const KEY_X: u8 = 0x2D;
    pub const KEY_Y: u8 = 0x2C;
    pub const KEY_Z: u8 = 0x15;
    pub const KEY_1: u8 = 0x02;
    pub const KEY_2: u8 = 0x03;
    pub const KEY_3: u8 = 0x04;
    pub const KEY_4: u8 = 0x05;
    pub const KEY_5: u8 = 0x06;
    pub const KEY_6: u8 = 0x07;
    pub const KEY_7: u8 = 0x08;
    pub const KEY_8: u8 = 0x09;
    pub const KEY_9: u8 = 0x0A;
    pub const KEY_0: u8 = 0x0B;
    pub const KEY_F1: u8 = 0x3B;
    pub const KEY_F2: u8 = 0x3C;
    pub const KEY_F3: u8 = 0x3D;
    pub const KEY_F4: u8 = 0x3E;
    pub const KEY_F5: u8 = 0x3F;
    pub const KEY_F6: u8 = 0x40;
    pub const KEY_F7: u8 = 0x41;
    pub const KEY_F8: u8 = 0x42;
    pub const KEY_F9: u8 = 0x43;
    pub const KEY_F10: u8 = 0x44;
    pub const KEY_F11: u8 = 0x57;
    pub const KEY_F12: u8 = 0x58;
    pub const KEY_ESC: u8 = 0x01;
    pub const KEY_ENTER: u8 = 0x1C;
    pub const KEY_SPACE: u8 = 0x39;
    pub const KEY_TAB: u8 = 0x0F;
    pub const KEY_BACKSPACE: u8 = 0x0E;
    pub const KEY_CAPSLOCK: u8 = 0x3A;
    pub const KEY_ESCAPE: u8 = 0x01;
    pub const KEY_UP: u8 = 0x48;
    pub const KEY_DOWN: u8 = 0x50;
    pub const KEY_LEFT: u8 = 0x4B;
    pub const KEY_RIGHT: u8 = 0x4D;

    pub const KEY_LEFT_SHIFT: u8 = 0x2A;
    pub const KEY_LEFT_CTRL: u8 = 0x1D;
    pub const KEY_RIGHT_SHIFT: u8 = 0x36;
    pub const KEY_NUMBER_LOCK: u8 = 0x45;
    pub const KEY_SCROLL_LOCK: u8 = 0x46;
}

/// This enum represents kind of a keyboard event: A `Press` event is sent when a key is pressed and a `Release` event is sent when a key is released.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EventKind {
    Press,
    Release,
}

/// This enum represents the case of a letter.
#[derive(Copy, Clone)]
pub enum KeyCase {
    Upper,
    Lower,
}

impl KeyCase {
    /// Constructs a new `KeyCase`.
    pub fn new(upper: bool) -> KeyCase {
        if upper {
            KeyCase::Upper
        } else {
            KeyCase::Lower
        }
    }
}

/// This struct represents a keyboard event.
pub struct KeyEvent {
    keycode: u8,
    kind: EventKind,
}

impl KeyEvent {
    /// Gets the keycode of the event.
    pub fn keycode(&self) -> u8 {
        self.keycode
    }

    /// Gets the kind of the event.
    pub fn kind(&self) -> EventKind {
        self.kind
    }

    /// Converts a scancode to a keyboard event.
    pub fn from_scancode(scancode: u8) -> Option<Self> {
        match scancode {
            PRESS_START..=PRESS_END => Some(KeyEvent {
                keycode: scancode,
                kind: EventKind::Press,
            }),
            RELEASE_START..=RELEASE_END => Some(KeyEvent {
                keycode: scancode - 0x80,
                kind: EventKind::Release,
            }),
            _ => None,
        }
    }

    /// Gets the ascii letter associated with the event's keycode.
    pub fn get_ascii(&self, case: KeyCase) -> u8 {
        match case {
            KeyCase::Upper => KEYMAP_UPPER[self.keycode as usize],
            KeyCase::Lower => KEYMAP_LOWER[self.keycode as usize],
        }
    }
}

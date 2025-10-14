// kernel/src/drivers/keyboard.rs
use spin::Mutex;
use crate::{kprintln, log_debug, log_info};

/// Keyboard scancode to ASCII mapping table (US keyboard layout)
static SCANCODE_TO_ASCII: [u8; 128] = [
    0,    27,  b'1', b'2', b'3', b'4', b'5', b'6',  // 0x00-0x07
    b'7', b'8', b'9', b'0', b'-', b'=', 8,   b'\t', // 0x08-0x0F (8=Backspace)
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', // 0x10-0x17
    b'o', b'p', b'[', b']', b'\n', 0,   b'a', b's', // 0x18-0x1F (Ctrl)
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', // 0x20-0x27
    b'\'',b'`', 0,   b'\\',b'z', b'x', b'c', b'v', // 0x28-0x2F (LShift)
    b'b', b'n', b'm', b',', b'.', b'/', 0,   b'*', // 0x30-0x37 (RShift)
    0,    b' ', 0,   0,   0,   0,   0,   0,         // 0x38-0x3F (Alt, CapsLock, F1-F5)
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x40-0x47 (F6-F10, NumLock, ScrollLock)
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x48-0x4F (Home, Up, PgUp, -, Left, ...)
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x50-0x57
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x58-0x5F
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x60-0x67
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x68-0x6F
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x70-0x77
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x78-0x7F
];

/// Character mapping when Shift key is pressed
static SCANCODE_TO_ASCII_SHIFT: [u8; 128] = [
    0,    27,  b'!', b'@', b'#', b'$', b'%', b'^',  // 0x00-0x07
    b'&', b'*', b'(', b')', b'_', b'+', 8,   b'\t', // 0x08-0x0F
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', // 0x10-0x17
    b'O', b'P', b'{', b'}', b'\n', 0,   b'A', b'S', // 0x18-0x1F
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', // 0x20-0x27
    b'"', b'~', 0,   b'|', b'Z', b'X', b'C', b'V',  // 0x28-0x2F
    b'B', b'N', b'M', b'<', b'>', b'?', 0,   b'*',  // 0x30-0x37
    0,    b' ', 0,   0,   0,   0,   0,   0,         // 0x38-0x3F
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x40-0x47
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x48-0x4F
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x50-0x57
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x58-0x5F
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x60-0x67
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x68-0x6F
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x70-0x77
    0,    0,   0,   0,   0,   0,   0,   0,         // 0x78-0x7F
];

/// Numpad scancode mapping
static NUMPAD_SCANCODE_TO_ASCII: [u8; 128] = {
    let mut map = [0u8; 128];
    map[0x47] = b'7';
    map[0x48] = b'8';
    map[0x49] = b'9';
    map[0x4B] = b'4';
    map[0x4C] = b'5';
    map[0x4D] = b'6';
    map[0x4F] = b'1';
    map[0x50] = b'2';
    map[0x51] = b'3';
    map[0x52] = b'0';
    map[0x53] = b'.';
    map[0x4A] = b'-';
    map[0x4E] = b'+';
    map[0x37] = b'*';
    map[0x35] = b'/';
    map
};

/// Keyboard state
struct KeyboardState {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
    num_lock: bool,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            caps_lock: false,
            num_lock: true, // Usually enabled by default
        }
    }
}

static KEYBOARD_STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

/// Handle keyboard scancode
pub fn handle_scancode(raw: u8) {
    let mut state = KEYBOARD_STATE.lock();

    let key_released = (raw & 0x80) != 0;
    let scancode = raw & 0x7F;

    // Handle modifier keys
    match scancode {
        0x2A | 0x36 => { state.shift_pressed = !key_released; return; } // Shift
        0x1D => { state.ctrl_pressed = !key_released; return; }          // Ctrl
        0x38 => { state.alt_pressed = !key_released; return; }           // Alt
        0x3A => { if !key_released { state.caps_lock = !state.caps_lock; } return; } // Caps Lock
        0x45 => { if !key_released { state.num_lock = !state.num_lock; } return; }   // Num Lock
        _ => {}
    }

    if key_released { return; }

    // When NumLock is off: Numpad outputs arrow or control keys
    if !state.num_lock {
        match scancode {
            0x47 => { kprintln!("[Home]"); return; }
            0x48 => { kprintln!("[Up]"); return; }
            0x49 => { kprintln!("[PgUp]"); return; }
            0x4B => { kprintln!("[Left]"); return; }
            0x4C => { kprintln!("[Center]"); return; }
            0x4D => { kprintln!("[Right]"); return; }
            0x4F => { kprintln!("[End]"); return; }
            0x50 => { kprintln!("[Down]"); return; }
            0x51 => { kprintln!("[PgDn]"); return; }
            0x52 => { kprintln!("[Insert]"); return; }
            0x53 => { kprintln!("[Delete]"); return; }
            _ => {}
        }
    }

    // Handle numpad output based on NumLock state
    let ascii = if state.num_lock && NUMPAD_SCANCODE_TO_ASCII[scancode as usize] != 0 {
        NUMPAD_SCANCODE_TO_ASCII[scancode as usize]
    } else if state.shift_pressed {
        SCANCODE_TO_ASCII_SHIFT[scancode as usize]
    } else {
        SCANCODE_TO_ASCII[scancode as usize]
    };

    if ascii == 0 {
        return;  // Unmapped key
    }

    // Handle Caps Lock (affects letters only)
    let ascii = if state.caps_lock && ascii.is_ascii_alphabetic() {
        if state.shift_pressed {
            ascii.to_ascii_lowercase()
        } else {
            ascii.to_ascii_uppercase()
        }
    } else {
        ascii
    };

    // Handle Ctrl combinations
    if state.ctrl_pressed {
        match ascii {
            b'c' | b'C' => { kprintln!("^C"); return; }
            b'd' | b'D' => { kprintln!("^D"); return; }
            b'l' | b'L' => {
                crate::tty::tty::clear(0x000000);
                return;
            }
            _ => {}
        }
    }

    // Output character
    print_char(ascii);
}

/// Print a character to the screen
fn print_char(c: u8) {
    use crate::kprint;

    if c == b'\n' {
        kprintln!();
    } else if c == 8 {
        // TODO: Implement backspace functionality
        kprint!("\x08");
    } else if c.is_ascii_graphic() || c == b' ' {
        kprint!("{}", c as char);
    }
}

/// Initialize keyboard driver
pub fn init() {
    log_info!("Keyboard driver initialized");
}

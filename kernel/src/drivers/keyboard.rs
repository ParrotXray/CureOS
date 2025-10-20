// kernel/src/drivers/keyboard.rs
use spin::Mutex;
use crate::{kprintln, log_debug, log_info, shell, tty};

/// Keyboard scancode to ASCII mapping table (US keyboard layout)
static SCANCODE_TO_ASCII: [u8; 128] = [
    0,    27,  b'1', b'2', b'3', b'4', b'5', b'6',  // 0x00-0x07
    b'7', b'8', b'9', b'0', b'-', b'=', 8,   b'\t', // 0x08-0x0F (8=Backspace)
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', // 0x10-0x17
    b'o', b'p', b'[', b']', b'\n', 0,   b'a', b's', // 0x18-0x1F (Ctrl)
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', // 0x20-0x27
    b'\'',b'`', 0,   b'\\',b'z', b'x', b'c', b'v',  // 0x28-0x2F (LShift)
    b'b', b'n', b'm', b',', b'.', b'/', 0,   b'*',  // 0x30-0x37 (RShift, Numpad *)
    0,    b' ', 0,   0,   0,   0,   0,   0,          // 0x38-0x3F
    0,    0,   0,   0,   0,   0,   0,   b'7',       // 0x40-0x47 (Numpad 7)
    b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', // 0x48-0x4F (Numpad)
    b'2', b'3', b'0', b'.', 0,   0,   0,   0,       // 0x50-0x57 (Numpad)
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x58-0x5F
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x60-0x67
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x68-0x6F
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x70-0x77
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x78-0x7F
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
    0,    b' ', 0,   0,   0,   0,   0,   0,          // 0x38-0x3F
    0,    0,   0,   0,   0,   0,   0,   b'7',       // 0x40-0x47
    b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', // 0x48-0x4F
    b'2', b'3', b'0', b'.', 0,   0,   0,   0,       // 0x50-0x57
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x58-0x5F
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x60-0x67
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x68-0x6F
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x70-0x77
    0,    0,   0,   0,   0,   0,   0,   0,          // 0x78-0x7F
];

/// Keyboard state
struct KeyboardState {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
    num_lock: bool,
    e0_prefix: bool,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            caps_lock: false,
            num_lock: true,
            e0_prefix: false,
        }
    }
}

static KEYBOARD_STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

/// Handle keyboard scancode
pub fn handle_scancode(raw: u8) {
    let mut state = KEYBOARD_STATE.lock();

    if raw == 0xE0 {
        state.e0_prefix = true;
        return;
    }

    let key_released = (raw & 0x80) != 0;
    let scancode = raw & 0x7F;
    let is_extended = state.e0_prefix;
    state.e0_prefix = false;

    // Process the extended key
    if is_extended {
        if key_released {
            return;
        }

        match scancode {
            0x1C => tty::device::receive_char(b'\n'),    // Numpad Enter
            0x35 => tty::device::receive_char(b'/'),     // Numpad /
            0x47 => kprintln!("[Home]"),
            0x48 => kprintln!("[Up]"),
            0x49 => kprintln!("[PgUp]"),
            0x4B => kprintln!("[Left]"),
            0x4D => kprintln!("[Right]"),
            0x4F => kprintln!("[End]"),
            0x50 => kprintln!("[Down]"),
            0x51 => kprintln!("[PgDn]"),
            0x52 => kprintln!("[Insert]"),
            0x53 => kprintln!("[Delete]"),

            // 0x19 => { kprintln!("[Next Track]");}
            // 0x10 => { kprintln!("[Prev Track]");}
            // 0x24 => { kprintln!("[Stop]");}
            // 0x22 => { kprintln!("[Play/Pause]");}
            // 0x20 => { kprintln!("[Mute]");}
            // 0x30 => { kprintln!("[Volume Up]");}
            // 0x2E => { kprintln!("[Volume Down]");}
            _ => {}
        }
        return;
    }

    // Modifier keys
    match scancode {
        0x2A | 0x36 => { state.shift_pressed = !key_released; return; }
        0x1D => { state.ctrl_pressed = !key_released; return; }
        0x38 => { state.alt_pressed = !key_released; return; }
        0x3A => { if !key_released { state.caps_lock = !state.caps_lock; } return; }
        0x45 => { if !key_released { state.num_lock = !state.num_lock; } return; }
        _ => {}
    }

    if key_released {
        return;
    }

    // F1-F12
    match scancode {
        0x3B => { kprintln!("[F1]"); return; }
        0x3C => { kprintln!("[F2]"); return; }
        0x3D => { kprintln!("[F3]"); return; }
        0x3E => { kprintln!("[F4]"); return; }
        0x3F => { kprintln!("[F5]"); return; }
        0x40 => { kprintln!("[F6]"); return; }
        0x41 => { kprintln!("[F7]"); return; }
        0x42 => { kprintln!("[F8]"); return; }
        0x43 => { kprintln!("[F9]"); return; }
        0x44 => { kprintln!("[F10]"); return; }
        0x57 => { kprintln!("[F11]"); return; }
        0x58 => { kprintln!("[F12]"); return; }
        _ => {}
    }

    // Esc 鍵
    if scancode == 0x01 {
        kprintln!("[Esc]");
        return;
    }

    // Arrow keys when NumLock is OFF (0x47-0x53)
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

    let ascii = if state.shift_pressed {
        SCANCODE_TO_ASCII_SHIFT[scancode as usize]
    } else {
        SCANCODE_TO_ASCII[scancode as usize]
    };

    if ascii == 0 {
        return;
    }

    let ascii = if state.caps_lock && ascii.is_ascii_alphabetic() {
        if state.shift_pressed {
            ascii.to_ascii_lowercase()
        } else {
            ascii.to_ascii_uppercase()
        }
    } else {
        ascii
    };

    if state.ctrl_pressed {
        match ascii {
            b'c' | b'C' => {
                crate::tty::device::receive_char(3);  // Ctrl+C
                return;
            }
            b'l' | b'L' => {
                tty::device::receive_char(12);  // Ctrl+L
                return;
            }
            _ => return,
        }
    }

    tty::device::receive_char(ascii);
}


/// Initialize keyboard driver
pub fn init() {
    log_info!("Keyboard driver initialized");
}
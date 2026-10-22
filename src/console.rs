use core::fmt::{self, Write};
use core::time::Duration;
use spin::Mutex;
use uefi::proto::console::text::{Color, Key, ScanCode};

pub static CONSOLE: Mutex<Console> = Mutex::new(Console);

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uefi::system::with_stdout(|stdout| {
            let _ = stdout.write_str(s);
        });
        Ok(())
    }
}

pub fn set_color(foreground: Color, background: Color) {
    uefi::system::with_stdout(|stdout| {
        let _ = stdout.set_color(foreground, background);
    });
}

pub fn clear_screen() {
    uefi::system::with_stdout(|stdout| {
        let _ = stdout.clear();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut console = CONSOLE.lock();
    let _ = console.write_fmt(args);
}

/// Polls for a key from UEFI stdin without blocking.
pub fn read_key() -> Option<Key> {
    uefi::system::with_stdin(|stdin| {
        stdin.read_key().unwrap_or(None)
    })
}

/// Waits until a key is pressed from UEFI stdin.
pub fn get_key() -> Key {
    loop {
        if let Some(key) = read_key() {
            return key;
        }
        // Yield to firmware / VM timer with a 10ms stall to avoid pegging CPU
        uefi::boot::stall(Duration::from_millis(10));
    }
}

/// Erases the character preceding the cursor on the screen.
pub fn backspace() {
    uefi::system::with_stdout(|stdout| {
        let (col, row) = stdout.cursor_position();
        if col > 0 {
            let _ = stdout.set_cursor_position(col - 1, row);
            let _ = stdout.write_str(" ");
            let _ = stdout.set_cursor_position(col - 1, row);
        }
    });
}

/// Reads a line of ASCII text interactively from the console into `buf`.
pub fn read_line(prompt: &str, buf: &mut [u8]) -> usize {
    crate::print!("{}", prompt);
    let mut len = 0;
    loop {
        let key = get_key();
        match key {
            Key::Printable(ch) => {
                let c = char::from(ch);
                if c == '\r' || c == '\n' {
                    crate::println!();
                    break;
                } else if c == '\x08' || u16::from(ch) == 8 {
                    if len > 0 {
                        len -= 1;
                        backspace();
                    }
                } else if len < buf.len() && (c as u32) <= 127 {
                    let b = c as u8;
                    buf[len] = b;
                    len += 1;
                    crate::print!("{}", c);
                }
            }
            Key::Special(ScanCode::DELETE) => {
                if len > 0 {
                    len -= 1;
                    backspace();
                }
            }
            _ => {}
        }
    }
    len
}


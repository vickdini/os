use core::fmt::{self, Write};
use core::time::Duration;
use spin::Mutex;
use uefi::proto::console::text::{Color, Key, ScanCode};

const SCROLLBACK_MAX_LINES: usize = 512;
const SCROLLBACK_LINE_LEN: usize = 80;

pub struct ScrollbackBuffer {
    lines: [[u8; SCROLLBACK_LINE_LEN]; SCROLLBACK_MAX_LINES],
    lens: [usize; SCROLLBACK_MAX_LINES],
    head: usize,
    count: usize,
    cur_len: usize,
}

impl ScrollbackBuffer {
    pub const fn new() -> Self {
        Self {
            lines: [[0u8; SCROLLBACK_LINE_LEN]; SCROLLBACK_MAX_LINES],
            lens: [0; SCROLLBACK_MAX_LINES],
            head: 0,
            count: 0,
            cur_len: 0,
        }
    }

    pub fn push_str(&mut self, s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                self.lens[self.head] = self.cur_len;
                self.head = (self.head + 1) % SCROLLBACK_MAX_LINES;
                if self.count < SCROLLBACK_MAX_LINES {
                    self.count += 1;
                }
                self.cur_len = 0;
            } else if b == b'\r' {
                // Ignore carriage return
            } else if self.cur_len < SCROLLBACK_LINE_LEN {
                let h = self.head;
                let c = self.cur_len;
                self.lines[h][c] = b;
                self.cur_len += 1;
            }
        }
    }

    pub fn total_lines(&self) -> usize {
        self.count + if self.cur_len > 0 { 1 } else { 0 }
    }

    pub fn get_line<'a>(&self, idx: usize, buf: &'a mut [u8; SCROLLBACK_LINE_LEN]) -> Option<&'a str> {
        let total = self.total_lines();
        if idx >= total {
            return None;
        }

        if idx == self.count && self.cur_len > 0 {
            let head = self.head;
            let len = self.cur_len;
            buf[..len].copy_from_slice(&self.lines[head][..len]);
            return core::str::from_utf8(&buf[..len]).ok();
        }

        let ring_idx = if self.count < SCROLLBACK_MAX_LINES {
            idx
        } else {
            (self.head + idx) % SCROLLBACK_MAX_LINES
        };

        let len = self.lens[ring_idx];
        buf[..len].copy_from_slice(&self.lines[ring_idx][..len]);
        core::str::from_utf8(&buf[..len]).ok()
    }
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

pub struct Console {
    pub scrollback: ScrollbackBuffer,
}

impl Console {
    pub const fn new() -> Self {
        Self {
            scrollback: ScrollbackBuffer::new(),
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.scrollback.push_str(s);
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

pub fn probe_scancode(s: ScanCode) -> u16 {
    s.0
}

pub fn screen_size() -> (usize, usize) {
    uefi::system::with_stdout(|stdout| {
        if let Ok(Some(mode)) = stdout.current_mode() {
            (mode.columns(), mode.rows())
        } else {
            (80, 25)
        }
    })
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

/// Checks non-blockingly whether Ctrl+C (ASCII 3) or Ctrl+\ (ASCII 28) was pressed to interrupt execution.
pub fn check_interrupt() -> bool {
    if let Some(key) = read_key() {
        match key {
            Key::Printable(ch) if u16::from(ch) == 3 || u16::from(ch) == 28 => true,
            _ => false,
        }
    } else {
        false
    }
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
                let code = u16::from(ch);
                let c = char::from(ch);
                if code == 3 {
                    // Ctrl+C
                    crate::println!("^C");
                    crate::print!("{}", prompt);
                    len = 0;
                } else if c == '\r' || c == '\n' {
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
            Key::Special(ScanCode::UP) => {}
            Key::Special(ScanCode::DOWN) => {}
            Key::Special(ScanCode::LEFT) => {}
            Key::Special(ScanCode::RIGHT) => {}
            Key::Special(ScanCode::HOME) => {}
            Key::Special(ScanCode::END) => {}
            _ => {}
        }
    }
    len
}

use core::sync::atomic::{AtomicBool, Ordering};

pub static SCROLL_LOCK: AtomicBool = AtomicBool::new(false);

/// Checks whether Scroll Lock is active (via BDA 0x417 bit 0x10 or software toggle).
pub fn is_scroll_lock_active() -> bool {
    if SCROLL_LOCK.load(Ordering::Relaxed) {
        return true;
    }
    unsafe {
        let ptr = 0x417 as *const u8;
        (*ptr & 0x10) != 0
    }
}

/// Toggles the software Scroll Lock state and returns the new state.
pub fn toggle_scroll_lock() -> bool {
    let current = SCROLL_LOCK.load(Ordering::Relaxed);
    let new_val = !current;
    SCROLL_LOCK.store(new_val, Ordering::Relaxed);
    new_val
}

/// Sets the software Scroll Lock state.
pub fn set_scroll_lock(active: bool) {
    SCROLL_LOCK.store(active, Ordering::Relaxed);
}

/// Returns the maximum scroll offset available in the scrollback buffer.
pub fn max_scroll_offset() -> usize {
    let (_cols, rows) = screen_size();
    let console = CONSOLE.lock();
    let total = console.scrollback.total_lines();
    total.saturating_sub(rows.saturating_sub(1))
}

fn format_scroll_status<'a>(buf: &'a mut [u8], top: usize, bottom: usize, total: usize, offset: usize) -> &'a str {
    struct BufWriter<'b> {
        buf: &'b mut [u8],
        pos: usize,
    }
    impl<'b> Write for BufWriter<'b> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.buf.len().saturating_sub(self.pos);
            let to_copy = rem.min(bytes.len());
            self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
            self.pos += to_copy;
            Ok(())
        }
    }

    let mut writer = BufWriter { buf, pos: 0 };
    let _ = write!(
        writer,
        "-- SCROLL LOCK (scrolled up {} lines, {}-{} of {}) [↑/↓ to scroll] --",
        offset, top, bottom, total
    );
    let len = writer.pos;
    core::str::from_utf8(&buf[..len]).unwrap_or("-- SCROLL LOCK ACTIVE [↑/↓ to scroll] --")
}

/// Draws the terminal screen shifted up by `offset` lines.
/// When `offset == 0`, restores the live view with prompt and active line buffer.
pub fn draw_scroll_viewport(
    offset: usize,
    prompt: &str,
    line_text: &str,
    cursor_pos: usize,
    prompt_color: Color,
) {
    let (_cols, rows) = screen_size();
    let console = CONSOLE.lock();
    let total = console.scrollback.total_lines();
    let visible_rows = rows.saturating_sub(1);

    clear_screen();

    if offset == 0 {
        // Live bottom view
        let top = total.saturating_sub(visible_rows);
        let mut line_buf = [0u8; SCROLLBACK_LINE_LEN];

        for i in top..total {
            if let Some(line) = console.scrollback.get_line(i, &mut line_buf) {
                uefi::system::with_stdout(|stdout| {
                    let _ = stdout.write_str(line);
                    let _ = stdout.write_str("\r\n");
                });
            } else {
                uefi::system::with_stdout(|stdout| {
                    let _ = stdout.write_str("\r\n");
                });
            }
        }

        // Draw active prompt on bottom line
        uefi::system::with_stdout(|stdout| {
            let _ = stdout.set_color(prompt_color, Color::Black);
            let _ = stdout.write_str(prompt);
            let _ = stdout.set_color(Color::White, Color::Black);
            let _ = stdout.write_str(line_text);
            let (cur_col, cur_row) = stdout.cursor_position();
            let prompt_col = cur_col.saturating_sub(line_text.len());
            let _ = stdout.set_cursor_position(prompt_col + cursor_pos, cur_row);
        });
    } else {
        // Scrolled up into history
        let bottom = total.saturating_sub(offset);
        let top = bottom.saturating_sub(visible_rows);
        let mut line_buf = [0u8; SCROLLBACK_LINE_LEN];

        for i in top..bottom {
            if let Some(line) = console.scrollback.get_line(i, &mut line_buf) {
                uefi::system::with_stdout(|stdout| {
                    let _ = stdout.write_str(line);
                    let _ = stdout.write_str("\r\n");
                });
            } else {
                uefi::system::with_stdout(|stdout| {
                    let _ = stdout.write_str("\r\n");
                });
            }
        }

        for _ in (bottom - top)..visible_rows {
            uefi::system::with_stdout(|stdout| {
                let _ = stdout.write_str("\r\n");
            });
        }

        // Status line on the bottom row
        uefi::system::with_stdout(|stdout| {
            let _ = stdout.set_color(Color::Black, Color::Cyan);
            let mut num_buf = [0u8; 96];
            let status = format_scroll_status(&mut num_buf, top + 1, bottom, total, offset);
            let _ = stdout.write_str(status);
            let _ = stdout.set_color(Color::White, Color::Black);
        });
    }
}



use core::fmt::{self, Write};
use core::time::Duration;
use spin::Mutex;
use uefi::proto::console::text::{Color, Key, ScanCode};

const SCROLLBACK_MAX_LINES: usize = 512;
const SCROLLBACK_LINE_LEN: usize = 128;

pub fn color_to_u8(c: Color) -> u8 {
    match c {
        Color::Black => 0,
        Color::Blue => 1,
        Color::Green => 2,
        Color::Cyan => 3,
        Color::Red => 4,
        Color::Magenta => 5,
        Color::Brown => 6,
        Color::LightGray => 7,
        Color::DarkGray => 8,
        Color::LightBlue => 9,
        Color::LightGreen => 10,
        Color::LightCyan => 11,
        Color::LightRed => 12,
        Color::LightMagenta => 13,
        Color::Yellow => 14,
        Color::White => 15,
    }
}

pub fn u8_to_color(val: u8) -> Color {
    match val & 0x0F {
        0 => Color::Black,
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Cyan,
        4 => Color::Red,
        5 => Color::Magenta,
        6 => Color::Brown,
        7 => Color::LightGray,
        8 => Color::DarkGray,
        9 => Color::LightBlue,
        10 => Color::LightGreen,
        11 => Color::LightCyan,
        12 => Color::LightRed,
        13 => Color::LightMagenta,
        14 => Color::Yellow,
        _ => Color::White,
    }
}

pub fn u8_to_bg_color(val: u8) -> Color {
    match (val >> 4) & 0x07 {
        0 => Color::Black,
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Cyan,
        4 => Color::Red,
        5 => Color::Magenta,
        6 => Color::Brown,
        _ => Color::LightGray,
    }
}

pub struct ScrollbackBuffer {
    lines: [[u8; SCROLLBACK_LINE_LEN]; SCROLLBACK_MAX_LINES],
    attrs: [[u8; SCROLLBACK_LINE_LEN]; SCROLLBACK_MAX_LINES],
    lens: [usize; SCROLLBACK_MAX_LINES],
    cur_line: [u8; SCROLLBACK_LINE_LEN],
    cur_attrs: [u8; SCROLLBACK_LINE_LEN],
    head: usize,
    count: usize,
    cur_len: usize,
}

impl ScrollbackBuffer {
    pub const fn new() -> Self {
        Self {
            lines: [[0u8; SCROLLBACK_LINE_LEN]; SCROLLBACK_MAX_LINES],
            attrs: [[0x0Fu8; SCROLLBACK_LINE_LEN]; SCROLLBACK_MAX_LINES],
            lens: [0; SCROLLBACK_MAX_LINES],
            cur_line: [0u8; SCROLLBACK_LINE_LEN],
            cur_attrs: [0x0Fu8; SCROLLBACK_LINE_LEN],
            head: 0,
            count: 0,
            cur_len: 0,
        }
    }

    pub fn push_str(&mut self, s: &str, attr: u8) {
        for b in s.bytes() {
            if b == b'\n' {
                self.commit_cur_line();
            } else if b == b'\r' {
                // Ignore carriage return
            } else {
                if self.cur_len == SCROLLBACK_LINE_LEN {
                    self.commit_cur_line();
                }
                let c = self.cur_len;
                self.cur_line[c] = b;
                self.cur_attrs[c] = attr;
                self.cur_len += 1;
            }
        }
    }

    fn commit_cur_line(&mut self) {
        let h = self.head;
        let len = self.cur_len;
        self.lines[h][..len].copy_from_slice(&self.cur_line[..len]);
        self.attrs[h][..len].copy_from_slice(&self.cur_attrs[..len]);
        self.lens[h] = len;
        self.head = (self.head + 1) % SCROLLBACK_MAX_LINES;
        if self.count < SCROLLBACK_MAX_LINES {
            self.count += 1;
        }
        self.cur_len = 0;
    }

    pub fn total_lines(&self) -> usize {
        self.count + if self.cur_len > 0 { 1 } else { 0 }
    }

    pub fn get_line_with_attrs(
        &self,
        idx: usize,
        line_buf: &mut [u8; SCROLLBACK_LINE_LEN],
        attr_buf: &mut [u8; SCROLLBACK_LINE_LEN],
    ) -> Option<usize> {
        let total = self.total_lines();
        if idx >= total {
            return None;
        }

        if idx == self.count && self.cur_len > 0 {
            let len = self.cur_len;
            line_buf[..len].copy_from_slice(&self.cur_line[..len]);
            attr_buf[..len].copy_from_slice(&self.cur_attrs[..len]);
            return Some(len);
        }

        let ring_idx = if self.count < SCROLLBACK_MAX_LINES {
            idx
        } else {
            (self.head + idx) % SCROLLBACK_MAX_LINES
        };

        let len = self.lens[ring_idx];
        line_buf[..len].copy_from_slice(&self.lines[ring_idx][..len]);
        attr_buf[..len].copy_from_slice(&self.attrs[ring_idx][..len]);
        Some(len)
    }
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

pub struct Console {
    pub scrollback: ScrollbackBuffer,
    pub current_attr: u8,
}

impl Console {
    pub const fn new() -> Self {
        Self {
            scrollback: ScrollbackBuffer::new(),
            current_attr: 0x0F, // White on Black
        }
    }
}

pub fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Black => (0, 0, 0),
        Color::Blue => (0, 0, 170),
        Color::Green => (0, 170, 0),
        Color::Cyan => (0, 170, 170),
        Color::Red => (170, 0, 0),
        Color::Magenta => (170, 0, 170),
        Color::Brown => (170, 85, 0),
        Color::LightGray => (170, 170, 170),
        Color::DarkGray => (85, 85, 85),
        Color::LightBlue => (85, 85, 255),
        Color::LightGreen => (85, 255, 85),
        Color::LightCyan => (85, 255, 255),
        Color::LightRed => (255, 85, 85),
        Color::LightMagenta => (255, 85, 255),
        Color::Yellow => (255, 255, 85),
        Color::White => (255, 255, 255),
    }
}

static USE_STANDALONE_VIDEO: Mutex<bool> = Mutex::new(false);

pub fn enable_standalone_video(enabled: bool) {
    *USE_STANDALONE_VIDEO.lock() = enabled;
}

pub fn is_standalone_video() -> bool {
    *USE_STANDALONE_VIDEO.lock()
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let attr = self.current_attr;
        self.scrollback.push_str(s, attr);
        if is_standalone_video() {
            crate::video::console::write_str(s);
        } else {
            uefi::system::with_stdout(|stdout| {
                let _ = stdout.write_str(s);
            });
        }
        Ok(())
    }
}

pub fn set_color(foreground: Color, background: Color) {
    {
        let mut console = CONSOLE.lock();
        console.current_attr = color_to_u8(foreground) | (color_to_u8(background) << 4);
    }
    if is_standalone_video() {
        crate::video::console::set_colors(color_to_rgb(foreground), color_to_rgb(background));
    } else {
        uefi::system::with_stdout(|stdout| {
            let _ = stdout.set_color(foreground, background);
        });
    }
}

pub fn clear_screen() {
    if is_standalone_video() {
        crate::video::console::clear_screen();
    } else {
        uefi::system::with_stdout(|stdout| {
            let _ = stdout.clear();
        });
    }
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

fn render_formatted_line(line: &[u8], attrs: &[u8]) {
    let len = line.len();
    if len == 0 {
        return;
    }

    let mut start = 0;
    while start < len {
        let current_attr = attrs[start];
        let mut end = start + 1;
        while end < len && attrs[end] == current_attr {
            end += 1;
        }

        let fg = u8_to_color(current_attr);
        let bg = u8_to_bg_color(current_attr);
        uefi::system::with_stdout(|stdout| {
            let _ = stdout.set_color(fg, bg);
            if let Ok(segment) = core::str::from_utf8(&line[start..end]) {
                let _ = stdout.write_str(segment);
            }
        });

        start = end;
    }
    // Restore default text colors
    uefi::system::with_stdout(|stdout| {
        let _ = stdout.set_color(Color::White, Color::Black);
    });
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
        "-- SCROLL LOCK (up {} lines, lines {}-{} of {}) [UP/DOWN or PgUp/PgDn, ESC to exit] --",
        offset, top, bottom, total
    );
    let len = writer.pos;
    core::str::from_utf8(&buf[..len]).unwrap_or("-- SCROLL LOCK ACTIVE [UP/DOWN to scroll, ESC to exit] --")
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

    let mut line_buf = [0u8; SCROLLBACK_LINE_LEN];
    let mut attr_buf = [0u8; SCROLLBACK_LINE_LEN];

    if offset == 0 {
        // Live bottom view
        let top = total.saturating_sub(visible_rows);

        for i in top..total {
            if let Some(len) = console.scrollback.get_line_with_attrs(i, &mut line_buf, &mut attr_buf) {
                render_formatted_line(&line_buf[..len], &attr_buf[..len]);
                uefi::system::with_stdout(|stdout| {
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

        for i in top..bottom {
            if let Some(len) = console.scrollback.get_line_with_attrs(i, &mut line_buf, &mut attr_buf) {
                render_formatted_line(&line_buf[..len], &attr_buf[..len]);
                uefi::system::with_stdout(|stdout| {
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
        let display_top = if bottom == 0 { 0 } else { top + 1 };
        uefi::system::with_stdout(|stdout| {
            let _ = stdout.set_color(Color::Black, Color::Cyan);
            let mut num_buf = [0u8; 96];
            let status = format_scroll_status(&mut num_buf, display_top, bottom, total, offset);
            let _ = stdout.write_str(status);
            let _ = stdout.set_color(Color::White, Color::Black);
        });
    }
}



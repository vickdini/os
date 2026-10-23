use core::fmt::{self, Write};
use spin::Mutex;

use super::font::{FONT_HEIGHT, FONT_WIDTH};
use super::framebuffer::Framebuffer;

/// Global autonomous video console singleton.
pub static VIDEO_CONSOLE: Mutex<Option<VideoConsole>> = Mutex::new(None);

/// High-performance text console running directly on the hardware framebuffer.
pub struct VideoConsole {
    framebuffer: Framebuffer,
    cursor_col: usize,
    cursor_row: usize,
    max_cols: usize,
    max_rows: usize,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
}

impl VideoConsole {
    /// Creates a new VideoConsole wrapping the given Framebuffer.
    pub fn new(framebuffer: Framebuffer) -> Self {
        let max_cols = framebuffer.width() / FONT_WIDTH;
        let max_rows = framebuffer.height() / FONT_HEIGHT;

        Self {
            framebuffer,
            cursor_col: 0,
            cursor_row: 0,
            max_cols,
            max_rows,
            fg: (220, 220, 220), // Light Gray default text
            bg: (0, 0, 0),        // Black default background
        }
    }

    pub fn cols(&self) -> usize {
        self.max_cols
    }

    pub fn rows(&self) -> usize {
        self.max_rows
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_row)
    }

    pub fn set_color(&mut self, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        self.fg = fg;
        self.bg = bg;
    }

    pub fn set_cursor_pos(&mut self, col: usize, row: usize) {
        self.cursor_col = col.min(self.max_cols.saturating_sub(1));
        self.cursor_row = row.min(self.max_rows.saturating_sub(1));
    }

    pub fn clear(&mut self) {
        self.framebuffer.clear(self.bg.0, self.bg.1, self.bg.2);
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    pub fn new_line(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 < self.max_rows {
            self.cursor_row += 1;
        } else {
            self.framebuffer.scroll_up(FONT_HEIGHT, self.bg);
        }
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\r' => {
                self.cursor_col = 0;
            }
            '\n' => {
                self.new_line();
            }
            '\t' => {
                let tab_target = (self.cursor_col + 4) & !3;
                let next_col = tab_target.min(self.max_cols);
                while self.cursor_col < next_col {
                    self.framebuffer.draw_char(
                        self.cursor_col * FONT_WIDTH,
                        self.cursor_row * FONT_HEIGHT,
                        ' ',
                        self.fg,
                        self.bg,
                    );
                    self.cursor_col += 1;
                }
            }
            '\x08' => {
                // Backspace
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                    self.framebuffer.draw_char(
                        self.cursor_col * FONT_WIDTH,
                        self.cursor_row * FONT_HEIGHT,
                        ' ',
                        self.fg,
                        self.bg,
                    );
                }
            }
            printable => {
                if self.cursor_col >= self.max_cols {
                    self.new_line();
                }

                self.framebuffer.draw_char(
                    self.cursor_col * FONT_WIDTH,
                    self.cursor_row * FONT_HEIGHT,
                    printable,
                    self.fg,
                    self.bg,
                );
                self.cursor_col += 1;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn framebuffer_mut(&mut self) -> &mut Framebuffer {
        &mut self.framebuffer
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }
}

impl Write for VideoConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

/// Returns true if the autonomous video console has been initialized.
pub fn is_active() -> bool {
    VIDEO_CONSOLE.lock().is_some()
}

/// Writes a string to the autonomous video console if active.
pub fn write_str(s: &str) {
    let mut guard = VIDEO_CONSOLE.lock();
    if let Some(ref mut console) = *guard {
        console.write_str(s);
    }
}

/// Sets the current foreground and background colors for the autonomous video console.
pub fn set_colors(fg: (u8, u8, u8), bg: (u8, u8, u8)) {
    let mut guard = VIDEO_CONSOLE.lock();
    if let Some(ref mut console) = *guard {
        console.set_color(fg, bg);
    }
}

/// Clears the autonomous video console screen.
pub fn clear_screen() {
    let mut guard = VIDEO_CONSOLE.lock();
    if let Some(ref mut console) = *guard {
        console.clear();
    }
}

/// Returns the current (col, row) of the video console cursor.
pub fn cursor_position() -> (usize, usize) {
    let guard = VIDEO_CONSOLE.lock();
    if let Some(ref console) = *guard {
        console.cursor_pos()
    } else {
        (0, 0)
    }
}

/// Sets the current (col, row) of the video console cursor.
pub fn set_cursor_position(col: usize, row: usize) {
    let mut guard = VIDEO_CONSOLE.lock();
    if let Some(ref mut console) = *guard {
        console.set_cursor_pos(col, row);
    }
}

/// Returns the (cols, rows) dimensions of the video console matrix.
pub fn dimensions() -> (usize, usize) {
    let guard = VIDEO_CONSOLE.lock();
    if let Some(ref console) = *guard {
        (console.cols(), console.rows())
    } else {
        (80, 25)
    }
}

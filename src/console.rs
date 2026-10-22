use core::fmt::{self, Write};
use spin::Mutex;
use uefi::proto::console::text::Color;

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

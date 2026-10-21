#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::proto::console::text::Color;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn print_clear() {
    uefi::system::with_stdout(|stdout| {
        let _ = stdout.clear();
    });
}

pub fn print_set_color(foreground: Color, background: Color) {
    uefi::system::with_stdout(|stdout| {
        let _ = stdout.set_color(foreground, background);
    });
}

pub fn print_str(text: &str) {
    uefi::system::with_stdout(|stdout| {
        let _ = stdout.write_str(text);
    });
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    print_clear();
    print_set_color(Color::Yellow, Color::Black);
    print_str("Welcome to Hydra in Rust!\nBy Vick Dini\n");

    loop {}
}

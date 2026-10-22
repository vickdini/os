pub mod command;
pub mod commands;
pub mod fs;
pub mod state;

use core::str;
use uefi::proto::console::text::Color;

use crate::console::{read_line, set_color};
pub use command::Command;
pub use state::ShellState;

const LINE_BUF_LEN: usize = 128;
const MAX_ARGS: usize = 16;

pub fn print_welcome_banner() {
    set_color(Color::LightCyan, Color::Black);
    println!("\n+--------------------------------------------------------------+");
    println!("|             HYDRA MICROKERNEL INTERACTIVE SHELL              |");
    println!("|  Type 'help' for available commands, or 'info' for details.  |");
    println!("+--------------------------------------------------------------+\n");
}

pub fn execute_command(cmd_line: &str, state: &mut ShellState) {
    let mut args_buf = [""; MAX_ARGS];
    let mut arg_count = 0;

    for word in cmd_line.split_whitespace() {
        if arg_count < MAX_ARGS {
            args_buf[arg_count] = word;
            arg_count += 1;
        }
    }

    if arg_count == 0 {
        return;
    }

    let command_name = args_buf[0];
    let args = &args_buf[1..arg_count];

    if let Some(cmd) = commands::find_command(command_name) {
        cmd.execute(args, state);
    } else if fs::search_and_execute(command_name, args, state) {
        // Successfully found and executed external program via $PATH
    } else {
        set_color(Color::LightRed, Color::Black);
        println!("Unknown command: '{}'. Type 'help' for built-ins or 'path' for executable search dirs.", command_name);
    }
}

pub fn run() -> ! {
    let mut state = ShellState::new();
    let mut line_buf = [0u8; LINE_BUF_LEN];

    print_welcome_banner();

    loop {
        set_color(state.prompt_color, Color::Black);
        let len = read_line("hydra> ", &mut line_buf);
        set_color(Color::White, Color::Black);

        if len == 0 {
            continue;
        }

        let input_str = match str::from_utf8(&line_buf[..len]) {
            Ok(s) => s.trim(),
            Err(_) => {
                set_color(Color::LightRed, Color::Black);
                println!("[!] Invalid input encoding.");
                continue;
            }
        };

        if input_str.is_empty() {
            continue;
        }

        execute_command(input_str, &mut state);
    }
}

use uefi::proto::console::text::Color;
use crate::console::{clear_screen, set_color};
use crate::shell::command::Command;
use crate::shell::print_welcome_banner;
use crate::shell::state::ShellState;

pub struct ColorCommand;

impl Command for ColorCommand {
    fn name(&self) -> &'static str {
        "color"
    }

    fn description(&self) -> &'static str {
        "Change shell prompt and text color (cyan, green, yellow, red, white, magenta, blue)"
    }

    fn usage(&self) -> &'static str {
        "color <cyan|green|yellow|red|white|magenta|blue>"
    }

    fn execute(&self, args: &[&str], state: &mut ShellState) {
        let name = match args.first() {
            Some(n) => *n,
            None => {
                println!("Usage: {}", self.usage());
                return;
            }
        };

        let color = match name {
            "cyan" => Color::LightCyan,
            "green" => Color::LightGreen,
            "yellow" => Color::Yellow,
            "red" => Color::LightRed,
            "white" => Color::White,
            "magenta" => Color::LightMagenta,
            "blue" => Color::LightBlue,
            _ => {
                println!("Available colors: cyan, green, yellow, red, white, magenta, blue");
                return;
            }
        };

        state.prompt_color = color;
        set_color(color, Color::Black);
        println!("[+] Shell prompt color changed to {}", name);
    }
}

pub struct EchoCommand;

impl Command for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echo arguments back to the screen"
    }

    fn usage(&self) -> &'static str {
        "echo <text...>"
    }

    fn execute(&self, args: &[&str], _state: &mut ShellState) {
        for (i, word) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", word);
        }
        println!();
    }
}

pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["cls"]
    }

    fn description(&self) -> &'static str {
        "Clear the active screen buffer"
    }

    fn usage(&self) -> &'static str {
        "clear"
    }

    fn execute(&self, _args: &[&str], _state: &mut ShellState) {
        clear_screen();
        print_welcome_banner();
    }
}

pub struct PanicCommand;

impl Command for PanicCommand {
    fn name(&self) -> &'static str {
        "panic"
    }

    fn description(&self) -> &'static str {
        "Manually trigger a kernel panic to test diagnostic handler"
    }

    fn usage(&self) -> &'static str {
        "panic"
    }

    fn execute(&self, _args: &[&str], _state: &mut ShellState) {
        panic!("Manual kernel panic triggered by user from shell.");
    }
}

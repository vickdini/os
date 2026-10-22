use uefi::proto::console::text::Color;
use crate::console::set_color;
use crate::shell::command::Command;
use crate::shell::commands::all_commands;
use crate::shell::state::ShellState;

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["?"]
    }

    fn description(&self) -> &'static str {
        "Show this command manual and usage summary"
    }

    fn usage(&self) -> &'static str {
        "help [command]"
    }

    fn execute(&self, args: &[&str], _state: &mut ShellState) {
        if let Some(target) = args.first() {
            for cmd in all_commands() {
                if cmd.name() == *target || cmd.aliases().contains(target) {
                    set_color(Color::LightCyan, Color::Black);
                    println!("\nCommand: {}", cmd.name());
                    if !cmd.aliases().is_empty() {
                        print!("Aliases: ");
                        for (i, alias) in cmd.aliases().iter().enumerate() {
                            if i > 0 { print!(", "); }
                            print!("{}", alias);
                        }
                        println!();
                    }
                    println!("Description: {}", cmd.description());
                    if !cmd.usage().is_empty() {
                        println!("Usage:       {}", cmd.usage());
                    }
                    println!();
                    return;
                }
            }
            set_color(Color::LightRed, Color::Black);
            println!("No help entry found for '{}'.", target);
            return;
        }

        set_color(Color::Yellow, Color::Black);
        println!("\nHydra Shell Available Commands:");
        for cmd in all_commands() {
            set_color(Color::LightGreen, Color::Black);
            print!("  {:14}", cmd.name());
            set_color(Color::White, Color::Black);
            println!("- {}", cmd.description());
        }
        set_color(Color::LightCyan, Color::Black);
        println!("\nDefault Filesystem (ZFS Microkernel File Server):");
        set_color(Color::White, Color::Black);
        println!("  - Use 'ls', 'cat', 'write' to manage files in the ZFS pool ('hydrapool').");
        println!("  - Use 'zpool' and 'zfs' for pool and dataset/snapshot administration.");
        println!("  - Use 'efifs' for low-level UEFI bootloader media diagnostics.");
        println!("\nType 'help <command>' for specific syntax and options.\n");
    }
}

pub mod caps;
pub mod cpu;
pub mod help;
pub mod info;
pub mod ipc;
pub mod mem;
pub mod misc;
pub mod power;
pub mod status;
pub mod time;
pub mod translate;

use crate::shell::command::Command;

pub use caps::{CapInsertCommand, CapRevokeCommand, CapsCommand};
pub use cpu::CpuCommand;
pub use help::HelpCommand;
pub use info::InfoCommand;
pub use ipc::IpcCommand;
pub use mem::{AllocCommand, MemCommand};
pub use misc::{ClearCommand, ColorCommand, EchoCommand, PanicCommand};
pub use power::{HaltCommand, RebootCommand, ShutdownCommand};
pub use status::StatusCommand;
pub use time::TimeCommand;
pub use translate::TranslateCommand;

/// Static registry of all built-in commands and integrated programs.
/// To integrate a new command or program, instantiate it in this slice!
pub static COMMANDS: &[&dyn Command] = &[
    &HelpCommand,
    &InfoCommand,
    &StatusCommand,
    &CpuCommand,
    &MemCommand,
    &AllocCommand,
    &TranslateCommand,
    &CapsCommand,
    &CapInsertCommand,
    &CapRevokeCommand,
    &IpcCommand,
    &TimeCommand,
    &ColorCommand,
    &EchoCommand,
    &ClearCommand,
    &PanicCommand,
    &RebootCommand,
    &ShutdownCommand,
    &HaltCommand,
];

/// Returns all registered commands in the system.
pub fn all_commands() -> &'static [&'static dyn Command] {
    COMMANDS
}

/// Finds a command by name or alias.
pub fn find_command(name: &str) -> Option<&'static dyn Command> {
    for &cmd in COMMANDS {
        if cmd.name() == name || cmd.aliases().contains(&name) {
            return Some(cmd);
        }
    }
    None
}

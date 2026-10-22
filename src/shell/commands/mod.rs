pub mod caps;
pub mod cpu;
pub mod efifs;
pub mod fs_cmds;
pub mod gnu_fs;
pub mod gnu_sys;
pub mod gnu_text;
pub mod help;
pub mod info;
pub mod ipc;
pub mod mem;
pub mod misc;
pub mod power;
pub mod status;
pub mod time;
pub mod translate;
pub mod zfs_cmds;

use crate::shell::command::Command;

pub use caps::{CapInsertCommand, CapRevokeCommand, CapsCommand};
pub use cpu::CpuCommand;
pub use efifs::EfifsCommand;
pub use fs_cmds::{CatCommand, LsCommand, WriteCommand};
pub use gnu_fs::{
    CdCommand, CpCommand, DfCommand, DuCommand, MkdirCommand, MvCommand, PwdCommand, RmCommand,
    RmdirCommand, StatCommand, TouchCommand,
};
pub use gnu_sys::{
    FalseCommand, SeqCommand, SleepCommand, TrueCommand, UnameCommand, WhoamiCommand, YesCommand,
};
pub use gnu_text::{
    BasenameCommand, DirnameCommand, GrepCommand, HeadCommand, TailCommand, WcCommand,
};
pub use help::HelpCommand;
pub use info::InfoCommand;
pub use ipc::IpcCommand;
pub use mem::{AllocCommand, MemCommand};
pub use misc::{ClearCommand, ColorCommand, EchoCommand, PanicCommand};
pub use power::{HaltCommand, RebootCommand, ShutdownCommand};
pub use status::StatusCommand;
pub use time::TimeCommand;
pub use translate::TranslateCommand;
pub use zfs_cmds::{ZfsCommand, ZpoolCommand};

/// Static registry of all built-in commands and integrated programs.
/// To integrate a new command or program, instantiate it in this slice!
pub static COMMANDS: &[&dyn Command] = &[
    // Help and Shell State
    &HelpCommand,
    &InfoCommand,
    &StatusCommand,
    &ClearCommand,
    &ColorCommand,
    &EchoCommand,
    // GNU Coreutils - Filesystem
    &PwdCommand,
    &CdCommand,
    &LsCommand,
    &CatCommand,
    &WriteCommand,
    &TouchCommand,
    &MkdirCommand,
    &RmdirCommand,
    &RmCommand,
    &CpCommand,
    &MvCommand,
    &StatCommand,
    &DfCommand,
    &DuCommand,
    // GNU Coreutils - Text & Inspection
    &HeadCommand,
    &TailCommand,
    &WcCommand,
    &GrepCommand,
    &BasenameCommand,
    &DirnameCommand,
    // GNU Coreutils - System & Scripting
    &UnameCommand,
    &WhoamiCommand,
    &SleepCommand,
    &SeqCommand,
    &YesCommand,
    &TrueCommand,
    &FalseCommand,
    // Kernel & Hardware Diagnostics
    &CpuCommand,
    &MemCommand,
    &AllocCommand,
    &TranslateCommand,
    &CapsCommand,
    &CapInsertCommand,
    &CapRevokeCommand,
    &IpcCommand,
    &TimeCommand,
    &ZpoolCommand,
    &ZfsCommand,
    &EfifsCommand,
    &PanicCommand,
    // Power Management
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

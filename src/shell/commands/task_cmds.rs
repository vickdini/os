use core::sync::atomic::{AtomicU64, Ordering};
use uefi::proto::console::text::Color;

use crate::console::set_color;
use crate::shell::command::{parse_u64, Command};
use crate::shell::state::ShellState;
use crate::task::{self, TaskId, TaskPriority, TaskState};

static WORKER_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Background demo worker task executing concurrently with the interactive shell.
fn demo_background_worker() {
    for _ in 0..500 {
        WORKER_COUNTER.fetch_add(1, Ordering::Relaxed);
        task::sleep_ms(100);
    }
}

/// Lists all registered tasks, thread states, priority levels, and CPU tick statistics.
pub struct PsCommand;

impl Command for PsCommand {
    fn name(&self) -> &'static str {
        "ps"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["tasks", "threads"]
    }

    fn description(&self) -> &'static str {
        "List all active and sleeping kernel tasks in the preemptive scheduler"
    }

    fn usage(&self) -> &'static str {
        "ps"
    }

    fn execute(&self, _args: &[&str], _state: &mut ShellState) {
        if !task::is_active() {
            set_color(Color::Yellow, Color::Black);
            println!("[-] Multitasking scheduler is not yet active.");
            return;
        }

        let snapshots = task::get_task_snapshots();

        set_color(Color::LightCyan, Color::Black);
        println!(
            "{:<5} {:<16} {:<12} {:<10} {:>10} {:>18}",
            "PID", "NAME", "STATE", "PRIORITY", "TICKS", "STACK (RSP)"
        );
        println!("{:-<75}", "");

        for s in &snapshots {
            match s.state {
                TaskState::Running => set_color(Color::LightGreen, Color::Black),
                TaskState::Ready => set_color(Color::White, Color::Black),
                TaskState::Sleeping { .. } => set_color(Color::Yellow, Color::Black),
                TaskState::Blocked => set_color(Color::LightMagenta, Color::Black),
                TaskState::Dead => set_color(Color::DarkGray, Color::Black),
            }

            println!(
                "{:<5} {:<16} {:<12} {:<10} {:>10} {:#018x}",
                s.id.0,
                s.name,
                s.state.as_str(),
                s.priority.as_str(),
                s.ticks_run,
                s.rsp
            );
        }

        set_color(Color::LightCyan, Color::Black);
        println!("{:-<75}", "");
        let worker_val = WORKER_COUNTER.load(Ordering::Relaxed);
        println!(
            "Total tasks: {} | Timeslice: {} ms | Worker counter: {}",
            snapshots.len(),
            task::scheduler::DEFAULT_TIMESLICE_TICKS * 10,
            worker_val
        );
    }
}

/// Spawns a new concurrent background kernel thread.
pub struct SpawnCommand;

impl Command for SpawnCommand {
    fn name(&self) -> &'static str {
        "spawn"
    }

    fn description(&self) -> &'static str {
        "Spawn a new concurrent background task (e.g. 'spawn worker')"
    }

    fn usage(&self) -> &'static str {
        "spawn [worker]"
    }

    fn execute(&self, args: &[&str], _state: &mut ShellState) {
        if !task::is_active() {
            set_color(Color::LightRed, Color::Black);
            println!("[!] Multitasking scheduler is not active.");
            return;
        }

        let target = args.first().copied().unwrap_or("worker");

        match target {
            "worker" | "counter" | "demo" => {
                let id = task::spawn("bg_worker", demo_background_worker, TaskPriority::Normal);
                set_color(Color::LightGreen, Color::Black);
                println!(
                    "[+] Spawned background worker task #{} ('bg_worker').",
                    id.0
                );
                println!("    Type 'ps' to observe the task executing concurrently.");
            }
            unknown => {
                set_color(Color::LightRed, Color::Black);
                println!("Unknown task template '{}'. Available templates: worker, counter, demo", unknown);
            }
        }
    }
}

/// Terminates an active background kernel thread by its Task ID.
pub struct KillCommand;

impl Command for KillCommand {
    fn name(&self) -> &'static str {
        "kill"
    }

    fn description(&self) -> &'static str {
        "Terminate an active task by its PID (e.g. 'kill 2')"
    }

    fn usage(&self) -> &'static str {
        "kill <pid>"
    }

    fn execute(&self, args: &[&str], _state: &mut ShellState) {
        let arg = match args.first() {
            Some(a) => *a,
            None => {
                set_color(Color::LightRed, Color::Black);
                println!("Usage: kill <pid>");
                return;
            }
        };

        let pid = match parse_u64(arg) {
            Some(p) => p,
            None => {
                set_color(Color::LightRed, Color::Black);
                println!("Invalid PID '{}'", arg);
                return;
            }
        };

        if pid == 0 {
            set_color(Color::LightRed, Color::Black);
            println!("[!] Cannot kill PID 0 (Main shell thread).");
            return;
        }

        if pid == 1 {
            set_color(Color::LightRed, Color::Black);
            println!("[!] Cannot kill PID 1 (Kernel idle thread).");
            return;
        }

        if task::kill(TaskId(pid)) {
            set_color(Color::LightGreen, Color::Black);
            println!("[+] Task #{} terminated.", pid);
        } else {
            set_color(Color::LightRed, Color::Black);
            println!("[!] Task #{} not found or already dead.", pid);
        }
    }
}

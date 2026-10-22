use uefi::proto::console::text::Color;
use crate::console::set_color;
use crate::memory;
use crate::shell::command::Command;
use crate::shell::state::ShellState;

pub struct MemCommand;

impl Command for MemCommand {
    fn name(&self) -> &'static str {
        "mem"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["memory"]
    }

    fn description(&self) -> &'static str {
        "Show physical memory allocator statistics"
    }

    fn usage(&self) -> &'static str {
        "mem"
    }

    fn execute(&self, _args: &[&str], _state: &mut ShellState) {
        let alloc = memory::frame_allocator::FRAME_ALLOCATOR.lock();
        let total_bytes = alloc.total_usable_memory();
        let alloc_bytes = alloc.allocated_memory();
        let free_bytes = alloc.free_memory();

        set_color(Color::LightCyan, Color::Black);
        println!("\nPhysical Memory Manager Statistics:");
        set_color(Color::White, Color::Black);
        println!("  Total Usable RAM:   {} bytes ({} MiB)", total_bytes, total_bytes / (1024 * 1024));
        println!("  Allocated Frames:   {} bytes ({} KiB, {} frames)",
            alloc_bytes, alloc_bytes / 1024, alloc_bytes / memory::frame_allocator::PAGE_SIZE);
        println!("  Free Memory:        {} bytes ({} MiB)", free_bytes, free_bytes / (1024 * 1024));
        println!("  Base Page Size:     {} bytes (4 KiB)\n", memory::frame_allocator::PAGE_SIZE);
    }
}

pub struct AllocCommand;

impl Command for AllocCommand {
    fn name(&self) -> &'static str {
        "alloc"
    }

    fn description(&self) -> &'static str {
        "Allocate a physical 4 KiB memory frame on demand"
    }

    fn usage(&self) -> &'static str {
        "alloc"
    }

    fn execute(&self, _args: &[&str], _state: &mut ShellState) {
        let frame_opt = {
            let mut alloc = memory::frame_allocator::FRAME_ALLOCATOR.lock();
            alloc.allocate_frame()
        };

        match frame_opt {
            Some(frame) => {
                set_color(Color::LightGreen, Color::Black);
                println!("[+] Successfully allocated 4 KiB frame at physical address: {:#x}",
                    frame.start_address().as_u64());
            }
            None => {
                set_color(Color::LightRed, Color::Black);
                println!("[!] Error: Out of Physical Memory (OOM).");
            }
        }
    }
}

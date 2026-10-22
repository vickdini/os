use uefi::proto::console::text::Color;
use crate::arch;
use crate::console::set_color;
use crate::shell::command::Command;
use crate::shell::state::ShellState;

pub struct CpuCommand;

impl Command for CpuCommand {
    fn name(&self) -> &'static str {
        "cpu"
    }

    fn description(&self) -> &'static str {
        "Display CPU control registers (CR0, CR2, CR3, CR4)"
    }

    fn usage(&self) -> &'static str {
        "cpu"
    }

    fn execute(&self, _args: &[&str], _state: &mut ShellState) {
        let cpu = arch::cpu::read_state();
        set_color(Color::LightCyan, Color::Black);
        println!("\nCPU State & Control Registers:");
        set_color(Color::White, Color::Black);
        println!("  CR0:                {:?} (Paging & Protection flags)", cpu.cr0);
        println!("  CR2:                {:#x} (Last Page Fault linear address)", cpu.cr2);
        println!("  CR3 (PML4 Root):    {:#x} (Base physical address of 4-level page table)", cpu.cr3_frame.as_u64());
        println!("  CR4:                {:?} (Architecture Extensions & PAE)\n", cpu.cr4);
    }
}

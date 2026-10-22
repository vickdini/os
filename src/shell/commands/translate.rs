use uefi::proto::console::text::Color;
use x86_64::VirtAddr;
use crate::console::set_color;
use crate::memory;
use crate::shell::command::{parse_hex, Command};
use crate::shell::state::ShellState;

pub struct TranslateCommand;

impl Command for TranslateCommand {
    fn name(&self) -> &'static str {
        "translate"
    }

    fn description(&self) -> &'static str {
        "Walk PML4 4-level paging to translate a virtual address to physical"
    }

    fn usage(&self) -> &'static str {
        "translate <virtual_address_hex> (e.g. translate 0x400000)"
    }

    fn execute(&self, args: &[&str], _state: &mut ShellState) {
        let s = match args.first() {
            Some(val) => *val,
            None => {
                set_color(Color::LightRed, Color::Black);
                println!("Usage: {}", self.usage());
                return;
            }
        };

        let addr_u64 = match parse_hex(s) {
            Some(val) => val,
            None => {
                set_color(Color::LightRed, Color::Black);
                println!("[!] Invalid hexadecimal address: '{}'", s);
                return;
            }
        };

        let virt = VirtAddr::new(addr_u64);
        let address_space = memory::paging::AddressSpace::active();
        let phys_opt = unsafe { address_space.translate_addr(virt) };

        set_color(Color::LightCyan, Color::Black);
        print!("[*] Translating Virtual Address {:#x} via PML4 tree: ", virt.as_u64());
        match phys_opt {
            Some(phys) => {
                set_color(Color::LightGreen, Color::Black);
                println!("Mapped -> Physical Address {:#x}", phys.as_u64());
            }
            None => {
                set_color(Color::Yellow, Color::Black);
                println!("Not Mapped / Page Fault");
            }
        }
    }
}

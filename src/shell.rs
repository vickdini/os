use core::str;
use uefi::proto::console::text::Color;
use uefi::runtime::ResetType;
use uefi::Status;
use x86_64::VirtAddr;

use crate::arch;
use crate::capability::table::CapabilityTable;
use crate::capability::{CapType, Capability, Rights};
use crate::console::{clear_screen, read_line, set_color};
use crate::ipc::{Endpoint, Message};
use crate::memory;

const LINE_BUF_LEN: usize = 128;

pub struct ShellState {
    pub cap_table: CapabilityTable,
    pub endpoint: Endpoint,
    pub prompt_color: Color,
}

impl ShellState {
    pub fn new() -> Self {
        let mut cap_table = CapabilityTable::new();

        // Initialize root capabilities
        let cpu = arch::cpu::read_state();
        let _ = cap_table.insert(Capability::new(
            CapType::AddressSpace {
                root_cr3: cpu.cr3_frame.as_u64(),
            },
            Rights::ALL,
        ));
        let _ = cap_table.insert(Capability::new(
            CapType::Endpoint { endpoint_id: 1 },
            Rights(Rights::READ.0 | Rights::WRITE.0 | Rights::CALL.0),
        ));

        Self {
            cap_table,
            endpoint: Endpoint::new(1),
            prompt_color: Color::LightCyan,
        }
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

fn print_welcome_banner() {
    set_color(Color::LightCyan, Color::Black);
    println!("\n+--------------------------------------------------------------+");
    println!("|             HYDRA MICROKERNEL INTERACTIVE SHELL              |");
    println!("|  Type 'help' for available commands, or 'info' for details.  |");
    println!("+--------------------------------------------------------------+\n");
}

fn execute_command(cmd_line: &str, state: &mut ShellState) {
    let mut parts = cmd_line.split_whitespace();
    let command = match parts.next() {
        Some(cmd) => cmd,
        None => return,
    };

    match command {
        "help" | "?" => cmd_help(),
        "info" | "uname" => cmd_info(),
        "status" => cmd_status(state),
        "cpu" => cmd_cpu(),
        "mem" | "memory" => cmd_mem(),
        "alloc" => cmd_alloc(),
        "translate" => cmd_translate(parts.next()),
        "caps" | "cspace" => cmd_caps(&state.cap_table),
        "cap_insert" => cmd_cap_insert(parts.next(), state),
        "cap_revoke" => cmd_cap_revoke(parts.next(), state),
        "ipc" => cmd_ipc(parts, state),
        "time" | "date" => cmd_time(),
        "color" => cmd_color(parts.next(), state),
        "echo" => cmd_echo(parts),
        "clear" | "cls" => {
            clear_screen();
            print_welcome_banner();
        }
        "panic" => {
            panic!("Manual kernel panic triggered by user from shell.");
        }
        "reboot" => {
            set_color(Color::Yellow, Color::Black);
            println!("[*] Rebooting system...");

            // 1. Standard x86 8042 Keyboard Controller pulse reset line (Port 0x64, 0xFE)
            unsafe {
                use x86_64::instructions::port::Port;
                let mut p64: Port<u8> = Port::new(0x64);
                p64.write(0xFE);

                // 2. Standard x86 PCI Reset Control Register (Port 0xCF9, 0x06)
                let mut pcf9: Port<u8> = Port::new(0xCF9);
                pcf9.write(0x06);
            }

            // 3. Fallback to UEFI cold reset
            uefi::runtime::reset(ResetType::COLD, Status::SUCCESS, None);
        }
        "shutdown" => {
            set_color(Color::Yellow, Color::Black);
            println!("[*] Powering down system...");

            unsafe {
                use x86_64::instructions::port::Port;

                // 1. VirtualBox PIIX3/4 ACPI PM1a_CNT (Port 0x4004)
                let mut p4004: Port<u16> = Port::new(0x4004);
                p4004.write(0x3400); // SLP_EN | (5 << 10)
                p4004.write(0x2000); // SLP_EN | (0 << 10)

                // 2. QEMU PIIX4 ACPI PM1a_CNT (Port 0x604)
                let mut p604: Port<u16> = Port::new(0x604);
                p604.write(0x2000);
                p604.write(0x3400);

                // 3. Bochs / Cloud / Q35 ACPI PM1a_CNT (Port 0xb004)
                let mut pb004: Port<u16> = Port::new(0xb004);
                pb004.write(0x2000);
                pb004.write(0x3400);

                // 4. QEMU debug exit (Port 0x501)
                let mut p501: Port<u8> = Port::new(0x501);
                p501.write(0x00);

                // 5. APM Power Off (Port 0xb2)
                let mut pb2: Port<u8> = Port::new(0xb2);
                pb2.write(0x00);
            }

            // 6. UEFI Runtime Services Shutdown
            uefi::runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
        }
        "halt" => {
            set_color(Color::Yellow, Color::Black);
            println!("[*] Halting CPU. Hardware interrupts disabled.");
            x86_64::instructions::interrupts::disable();
            loop {
                x86_64::instructions::hlt();
            }
        }
        unknown => {
            set_color(Color::LightRed, Color::Black);
            println!("Unknown command: '{}'. Type 'help' for available commands.", unknown);
        }
    }
}

fn cmd_help() {
    set_color(Color::Yellow, Color::Black);
    println!("\nHydra Shell Commands:");
    println!("  help, ?                      Show this command manual");
    println!("  info, uname                  Display OS version, architecture, and vision");
    println!("  status                       Inspect health of all microkernel subsystems");
    println!("  cpu                          Display CPU control registers (CR0, CR2, CR3, CR4)");
    println!("  mem, memory                  Show physical memory allocator statistics");
    println!("  alloc                        Allocate a physical 4 KiB frame on demand");
    println!("  translate <virt_addr>        Walk PML4 4-level paging to translate virtual address");
    println!("  caps, cspace                 List active task capability slots and rights");
    println!("  cap_insert <ep_id>           Insert a new Endpoint capability into the table");
    println!("  cap_revoke <slot>            Revoke a capability from the table");
    println!("  ipc <label> [a0] [a1] [a2]   Send & receive a microkernel IPC message on endpoint #1");
    println!("  time, date                   Query UEFI hardware Real-Time Clock (RTC)");
    println!("  color <name>                 Set shell prompt color (cyan, green, yellow, red, white)");
    println!("  echo <args...>               Print text back to the terminal");
    println!("  clear, cls                   Clear screen buffer");
    println!("  panic                        Trigger a test kernel panic");
    println!("  reboot                       Reboot machine via UEFI runtime reset");
    println!("  shutdown                     Power off machine via UEFI runtime services");
    println!("  halt                         Disable interrupts and halt the CPU\n");
}

fn cmd_info() {
    set_color(Color::LightCyan, Color::Black);
    println!("\n============================================================");
    println!("                HYDRA MICROKERNEL v0.1.0                     ");
    println!("     UNIX-like Distributed Cloud/Fog/Edge Operating System  ");
    println!("============================================================");
    set_color(Color::White, Color::Black);
    println!("Target:        x86_64-unknown-uefi (Native 64-bit Long Mode)");
    println!("Security:      Object-Capability Security Model (seL4-inspired)");
    println!("IPC:           Synchronous Rendezvous & Fast Inline Register Passing");
    println!("Memory:        4-Level PML4 Paging & Physical Frame Allocator");
    println!("Firmware:      UEFI 64-bit (SystemTable, Direct Console I/O)");
    println!("Distributed:   Self-* Autonomous Healing, Edge/Cloud Resource Virtualization\n");
}

fn cmd_status(state: &ShellState) {
    set_color(Color::LightGreen, Color::Black);
    println!("\nSubsystem Status:");
    println!("  [+] GDT & TSS:           ONLINE (Ring 0 & Ring 3 segments loaded)");
    println!("  [+] CPU Registers:       ONLINE (Paging and Protection enabled)");
    
    let (usable_mb, allocated_kb, free_mb) = {
        let alloc = memory::frame_allocator::FRAME_ALLOCATOR.lock();
        (
            alloc.total_usable_memory() / (1024 * 1024),
            alloc.allocated_memory() / 1024,
            alloc.free_memory() / (1024 * 1024),
        )
    };
    println!("  [+] Frame Allocator:     ONLINE (Usable: {} MiB, Alloc: {} KiB, Free: {} MiB)",
        usable_mb, allocated_kb, free_mb);

    let address_space = memory::paging::AddressSpace::active();
    println!("  [+] Virtual Memory:      ONLINE (Active PML4: {:#x})",
        address_space.pml4_physical_address().as_u64());

    let mut cap_count = 0;
    for slot in 0..crate::capability::table::MAX_CAPABILITIES {
        if state.cap_table.get(slot).is_some() {
            cap_count += 1;
        }
    }
    println!("  [+] Capability Table:    ONLINE ({} active capabilities in CSpace)", cap_count);
    println!("  [+] IPC Rendezvous:      ONLINE (Endpoint #{} active, {} pending messages)\n",
        state.endpoint.id(), state.endpoint.pending_count());
}

fn cmd_cpu() {
    let cpu = arch::cpu::read_state();
    set_color(Color::LightCyan, Color::Black);
    println!("\nCPU State & Control Registers:");
    set_color(Color::White, Color::Black);
    println!("  CR0:                {:?} (Paging & Protection flags)", cpu.cr0);
    println!("  CR2:                {:#x} (Last Page Fault linear address)", cpu.cr2);
    println!("  CR3 (PML4 Root):    {:#x} (Base physical address of 4-level page table)", cpu.cr3_frame.as_u64());
    println!("  CR4:                {:?} (Architecture Extensions & PAE)\n", cpu.cr4);
}

fn cmd_mem() {
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

fn cmd_alloc() {
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

fn cmd_translate(addr_str: Option<&str>) {
    let s = match addr_str {
        Some(val) => val,
        None => {
            set_color(Color::LightRed, Color::Black);
            println!("Usage: translate <virtual_address_hex>");
            println!("Example: translate 0x400000");
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

fn cmd_caps(cap_table: &CapabilityTable) {
    set_color(Color::LightCyan, Color::Black);
    println!("\nActive Task Capability Space (CSpace):");
    println!("+------+------------------------------------+-----------------------------+");
    println!("| Slot | Capability Type                    | Rights                      |");
    println!("+------+------------------------------------+-----------------------------+");

    let mut found = 0;
    for slot in 0..crate::capability::table::MAX_CAPABILITIES {
        if let Some(cap) = cap_table.get(slot) {
            found += 1;
            let type_str = match cap.cap_type {
                CapType::AddressSpace { root_cr3 } => {
                    print!("| {:4} | AddressSpace (CR3={:#x})", slot, root_cr3);
                    1
                }
                CapType::Endpoint { endpoint_id } => {
                    print!("| {:4} | Endpoint (ID={})", slot, endpoint_id);
                    2
                }
                CapType::MemoryFrame { physical_addr } => {
                    print!("| {:4} | MemoryFrame ({:#x})", slot, physical_addr);
                    3
                }
                CapType::Thread { thread_id } => {
                    print!("| {:4} | Thread (ID={})", slot, thread_id);
                    4
                }
                CapType::HardwareDevice { device_id } => {
                    print!("| {:4} | HardwareDevice (ID={})", slot, device_id);
                    5
                }
                CapType::UntypedMemory { size_bytes } => {
                    print!("| {:4} | UntypedMemory ({} bytes)", slot, size_bytes);
                    6
                }
            };
            let _ = type_str;

            // Pad column
            print!(" | ");
            let mut rights_printed = false;
            if cap.rights.contains(Rights::READ) { print!("READ "); rights_printed = true; }
            if cap.rights.contains(Rights::WRITE) { print!("WRITE "); rights_printed = true; }
            if cap.rights.contains(Rights::EXECUTE) { print!("EXEC "); rights_printed = true; }
            if cap.rights.contains(Rights::GRANT) { print!("GRANT "); rights_printed = true; }
            if cap.rights.contains(Rights::CALL) { print!("CALL "); rights_printed = true; }
            if !rights_printed { print!("NONE"); }
            println!(" |");
        }
    }

    if found == 0 {
        println!("| (empty capability table)                                                |");
    }
    println!("+------+------------------------------------+-----------------------------+\n");
}

fn cmd_cap_insert(ep_str: Option<&str>, state: &mut ShellState) {
    let ep_id = match ep_str.and_then(parse_u32) {
        Some(id) => id,
        None => {
            set_color(Color::LightRed, Color::Black);
            println!("Usage: cap_insert <endpoint_id>");
            return;
        }
    };

    let cap = Capability::new(
        CapType::Endpoint { endpoint_id: ep_id },
        Rights(Rights::READ.0 | Rights::WRITE.0 | Rights::CALL.0),
    );

    match state.cap_table.insert(cap) {
        Ok(slot) => {
            set_color(Color::LightGreen, Color::Black);
            println!("[+] Inserted Endpoint #{} capability into CSpace slot #{}", ep_id, slot);
        }
        Err(err) => {
            set_color(Color::LightRed, Color::Black);
            println!("[!] Failed to insert capability: {}", err);
        }
    }
}

fn cmd_cap_revoke(slot_str: Option<&str>, state: &mut ShellState) {
    let slot = match slot_str.and_then(parse_usize) {
        Some(s) => s,
        None => {
            set_color(Color::LightRed, Color::Black);
            println!("Usage: cap_revoke <slot_number>");
            return;
        }
    };

    match state.cap_table.revoke(slot) {
        Some(cap) => {
            set_color(Color::LightGreen, Color::Black);
            println!("[+] Revoked capability at slot #{}: {:?}", slot, cap.cap_type);
        }
        None => {
            set_color(Color::Yellow, Color::Black);
            println!("[*] Slot #{} was already empty or out of bounds.", slot);
        }
    }
}

fn cmd_ipc<'a>(mut parts: impl Iterator<Item = &'a str>, state: &mut ShellState) {
    let label = match parts.next().and_then(parse_hex) {
        Some(l) => l as u32,
        None => {
            set_color(Color::LightRed, Color::Black);
            println!("Usage: ipc <hex_label> [arg0] [arg1] [arg2]");
            println!("Example: ipc 0x1001 42 1337");
            return;
        }
    };

    let arg0 = parts.next().and_then(parse_u64).unwrap_or(0);
    let arg1 = parts.next().and_then(parse_u64).unwrap_or(0);
    let arg2 = parts.next().and_then(parse_u64).unwrap_or(0);
    let arg3 = parts.next().and_then(parse_u64).unwrap_or(0);

    let msg = Message::with_args(label, [arg0, arg1, arg2, arg3]);

    match state.endpoint.send(msg) {
        Ok(()) => {
            set_color(Color::LightGreen, Color::Black);
            println!("[+] Message queued to Endpoint #{}: label={:#x}, args=[{}, {}, {}, {}]",
                state.endpoint.id(), msg.label, msg.args[0], msg.args[1], msg.args[2], msg.args[3]);
        }
        Err(e) => {
            set_color(Color::LightRed, Color::Black);
            println!("[!] Failed to send message: {}", e);
            return;
        }
    }

    // Immediately consume to complete rendezvous demonstration
    if let Some(received) = state.endpoint.receive() {
        set_color(Color::LightCyan, Color::Black);
        println!("[+] Rendezvous complete! Received message from Endpoint #{}:", state.endpoint.id());
        println!("    - Label: {:#x}", received.label);
        println!("    - Args:  {:?}", received.args);
    }
}

fn cmd_time() {
    match uefi::runtime::get_time() {
        Ok(t) => {
            set_color(Color::LightGreen, Color::Black);
            println!("UEFI Hardware RTC: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                t.year(), t.month(), t.day(), t.hour(), t.minute(), t.second());
        }
        Err(err) => {
            set_color(Color::LightRed, Color::Black);
            println!("[!] Could not read UEFI RTC time: {:?}", err);
        }
    }
}

fn cmd_color(color_name: Option<&str>, state: &mut ShellState) {
    let name = match color_name {
        Some(n) => n,
        None => {
            println!("Usage: color <cyan|green|yellow|red|white|magenta|blue>");
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

fn cmd_echo<'a>(parts: impl Iterator<Item = &'a str>) {
    let mut first = true;
    for word in parts {
        if !first {
            print!(" ");
        }
        print!("{}", word);
        first = false;
    }
    println!();
}

fn parse_hex(s: &str) -> Option<u64> {
    let s = s.trim();
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    if s.is_empty() {
        return None;
    }
    let mut val: u64 = 0;
    for &b in s.as_bytes() {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as u64,
            b'a'..=b'f' => (b - b'a' + 10) as u64,
            b'A'..=b'F' => (b - b'A' + 10) as u64,
            _ => return None,
        };
        val = val.checked_mul(16)?.checked_add(digit)?;
    }
    Some(val)
}

fn parse_u64(s: &str) -> Option<u64> {
    if s.starts_with("0x") || s.starts_with("0X") {
        parse_hex(s)
    } else {
        let mut val: u64 = 0;
        let s = s.trim();
        if s.is_empty() { return None; }
        for &b in s.as_bytes() {
            if !(b'0'..=b'9').contains(&b) { return None; }
            val = val.checked_mul(10)?.checked_add((b - b'0') as u64)?;
        }
        Some(val)
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    parse_u64(s).and_then(|v| u32::try_from(v).ok())
}

fn parse_usize(s: &str) -> Option<usize> {
    parse_u64(s).and_then(|v| usize::try_from(v).ok())
}

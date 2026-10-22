# Interactive Microkernel Shell Documentation

- **Source File**: [`src/shell.rs`](file:///d:/os/src/shell.rs)
- **Subsystem**: Microkernel User Interface & Command REPL

---

## 1. Conceptual Overview

Operating system development often starts with non-interactive boot routines that print diagnostic logs and halt. To enable active exploration, debugging, and experimentation with kernel primitives on live hardware or virtual machines, Hydra provides an **Interactive Microkernel Shell**.

The shell runs in UEFI text mode, accepting keyboard input via UEFI's `SimpleTextInput` protocol through [`src/console.rs`](file:///d:/os/src/console.rs), parsing user commands, and dispatching operations to kernel subsystems:

```
+-------------------------------------------------------------+
|                 Hydra Interactive Shell                     |
|                   Prompt: `hydra> `                         |
+-------------------------------------------------------------+
   |          |           |            |           |
   v          v           v            v           v
[ CPU ]    [ Memory ]  [ Paging ]   [ CSpace ]   [ IPC ]
Diagnostics  Allocator  Translation  Keyring    Rendezvous
```

---

## 2. Command Reference

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `help`, `?` | `help` | Displays the interactive command manual and usage summary. |
| `info`, `uname` | `info` | Prints Hydra Microkernel OS version, build target, and architecture overview. |
| `status` | `status` | Reports operational health across GDT, CPU, Frame Allocator, Paging, CSpace, and IPC. |
| `cpu` | `cpu` | Displays CPU control registers (`CR0`, `CR2`, `CR3` PML4 root, `CR4`). |
| `mem`, `memory` | `mem` | Displays physical memory manager statistics (Total Usable RAM, Allocated, Free). |
| `alloc` | `alloc` | Allocates a 4 KiB physical memory frame from the bump allocator and returns its physical address. |
| `translate` | `translate <hex_virt_addr>` | Walks the active 4-level PML4 page table hierarchy to resolve virtual to physical addresses. |
| `caps`, `cspace` | `caps` | Inspects the active task's capability table (slot numbers, resource types, and rights bitflags). |
| `cap_insert` | `cap_insert <ep_id>` | Inserts a new Endpoint capability into the task's CSpace. |
| `cap_revoke` | `cap_revoke <slot>` | Revokes an existing capability from the specified CSpace slot. |
| `ipc` | `ipc <label> [a0] [a1] [a2]` | Sends and receives a fast register IPC message through rendezvous Endpoint #1. |
| `time`, `date` | `time` | Queries the hardware Real-Time Clock (RTC) via UEFI runtime services. |
| `color` | `color <name>` | Customizes shell prompt and text color (`cyan`, `green`, `yellow`, `red`, `white`, `magenta`, `blue`). |
| `echo` | `echo <text...>` | Echoes input arguments back to the screen. |
| `clear`, `cls` | `clear` | Clears the active screen buffer and resets the cursor. |
| `panic` | `panic` | Manually triggers the kernel panic handler to verify diagnostic output. |
| `reboot` | `reboot` | Reboots the computer using UEFI cold reset runtime services. |
| `shutdown` | `shutdown` | Powers down the computer using UEFI shutdown runtime services. |
| `halt` | `halt` | Disables CPU interrupts and enters a low-power `hlt` loop. |

---

## 3. Architecture & Key Features

### Thread-Safe Interactive Input
Interactive input is handled by `console::read_line`:
- Characters are received from UEFI stdin with a 10 ms sleep interval via `uefi::boot::stall` to ensure low VM CPU utilization.
- Real-time backspace support erases characters from screen and buffer seamlessly via cursor positioning.
- Enter (`\r` or `\n`) submits the command line.

### Multi-Tier Power Management (`shutdown` & `reboot`)
Virtual machine firmware (such as VirtualBox's EFI and QEMU/EDK2) frequently stubs or hangs on generic UEFI runtime `ResetSystem(EfiResetShutdown)` prior to ACPI table activation. Hydra solves this by implementing direct hardware I/O power transitions:
1. **VirtualBox ACPI**: Writes sleep state S5 bits to PIIX3/4 ACPI `PM1a_CNT` (I/O Port `0x4004`).
2. **QEMU ACPI**: Writes to standard PIIX4 ACPI `PM1a_CNT` (I/O Port `0x604`) and debug exit (`0x501`).
3. **Cloud / Bochs ACPI**: Writes to I/O Port `0xb004` and APM (`0xb2`).
4. **Reboot Sequencing**: Pulses the 8042 keyboard controller reset line (`0x64`, `0xFE`) and the PCI reset control register (`0xCF9`, `0x06`) before falling back to UEFI cold reset.


### Virtual Address Translation Inspection
The `translate` command allows live debugging of paging:
```
hydra> translate 0x400000
[*] Translating Virtual Address 0x400000 via PML4 tree: Mapped -> Physical Address 0x400000
```

### On-Demand Memory Allocation
The `alloc` command demonstrates the physical memory frame allocator in real-time:
```
hydra> alloc
[+] Successfully allocated 4 KiB frame at physical address: 0x101000
hydra> mem
Physical Memory Manager Statistics:
  Total Usable RAM:   ...
  Allocated Frames:   ...
```

### Live Microkernel IPC
The `ipc` command sends a message through a rendezvous endpoint and reads the delivered payload:
```
hydra> ipc 0x2001 100 200 300
[+] Message queued to Endpoint #1: label=0x2001, args=[100, 200, 300, 0]
[+] Rendezvous complete! Received message from Endpoint #1:
    - Label: 0x2001
    - Args:  [100, 200, 300, 0]
```

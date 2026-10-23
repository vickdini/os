# Hardware Takeover & ExitBootServices

- **Source File**: [`src/arch/boot_services.rs`](file:///d:/os/src/arch/boot_services.rs)
- **Subsystem**: Architecture / Hardware Ownership & UEFI Termination

---

## 1. Conceptual Overview

Operating systems that boot via UEFI start their existence in a hybrid environment: the processor is in 64-bit Long Mode, but the firmware retains partial control over memory allocation, display devices, and the system timer via **UEFI Boot Services**.

To become a true, self-governing microkernel, Hydra OS must execute the **Hardware Takeover Sequence** by calling the UEFI specification's terminal function: `ExitBootServices`.

Once `ExitBootServices` succeeds:
1. **Firmware Eviction**: All UEFI boot-time protocols, memory allocators, and event timers cease execution immediately.
2. **Hardware Reconfiguration**: The kernel assumes exclusive control over the Global Descriptor Table (GDTR), Task State Segment (TR), Interrupt Descriptor Table (IDTR), and Page Table Root (CR3).
3. **Autonomous Driver Transition**: Console output routes directly to Hydra's hardware GOP linear framebuffer (`VideoConsole`), and keyboard input routes directly to the physical 8042 PS/2 keyboard controller.

```
+---------------------------------------------------------------------------------+
|                         PHASE A: UEFI BOOT SERVICES                             |
|  - Firmware owns GDTR, IDTR, and CR3 PML4 Root                                  |
|  - Console output routed through UEFI SimpleTextOutput                          |
|  - Keyboard input polled through UEFI SimpleTextInput                           |
+---------------------------------------------------------------------------------+
                                         |
                                         |  arch::boot_services::take_hardware_ownership()
                                         v
+---------------------------------------------------------------------------------+
|                         PHASE B: HARDWARE TAKEOVER                              |
|  1. Enable Standalone Linear Framebuffer Console (VideoConsole)                 |
|  2. Switch Input to 8042 PS/2 Controller Driver (Ports 0x60/0x64)               |
|  3. Call ExitBootServices(None) -> Capture Final UEFI Memory Map                |
|  4. Load Hydra GDT into GDTR -> Far return (retfq) into Ring 0 CS (0x08)        |
|  5. Reload SS, DS, ES, FS, GS (0x10) and load TSS into TR via ltr (0x28)        |
|  6. Enable TSS IST 1 on IDT Double Fault (#DF) Gate                             |
|  7. Switch CPU CR3 to Hydra Autonomous PML4 Table (conventional RAM)            |
+---------------------------------------------------------------------------------+
                                         |
                                         v
+---------------------------------------------------------------------------------+
|                         PHASE C: BARE-METAL HYDRA OS                            |
|  - 100% Autonomous Microkernel Execution with Zero Firmware Dependencies        |
+---------------------------------------------------------------------------------+
```

---

## 2. Hardware Takeover Pipeline

### Step 1: Autonomous Video & Keyboard Activation
Before firmware protocols are severed, Hydra switches console routing:
```rust
enable_standalone_video(true);
set_hardware_mode(true);
```
All subsequent `print!` and `println!` calls draw bitmap glyphs directly into physical video memory (`0x80000000`), and `get_key()` reads scancodes from port `0x60`.

### Step 2: Calling `ExitBootServices`
```rust
let memory_map = unsafe { uefi::boot::exit_boot_services(None) };
let entries_count = memory_map.entries().count();
```
The firmware validates the memory map key, frees internal boot-service structures, and hands unconditional hardware control to Hydra.

### Step 3: GDT & Segment Reload (`load_and_switch`)
In 64-bit mode, the Code Segment (`CS`) cannot be reloaded using `mov cs, ax`. Hydra uses a far return (`retfq`) trampoline to reload `CS` with `0x08`:
```nasm
    push {sel}              // Kernel Code Selector (0x08)
    lea {tmp}, [2f + rip]   // Return RIP
    push {tmp}
    retfq                   // Far return: pops RIP then CS
2:
    mov ds, 0x10
    mov ss, 0x10
    ltr {tss_sel}           // Load Task Register with 0x28
```

### Step 4: TSS IST Attachment on Double Fault
With the Task Register (`TR`) loaded with Hydra's TSS, the IDT entry for Double Fault (`#DF`, vector 8) is linked to IST index 1:
```rust
idt.entries[8].set_ist(1);
```
Any catastrophic kernel stack overflow will now switch immediately to Hydra's dedicated 20 KiB emergency stack.

### Step 5: Autonomous CR3 Switch
Hydra switches the processor's Page Map Level 4 root from the firmware's PML4 table to Hydra's autonomous PML4 table:
```rust
KERNEL_SPACE.lock().switch();
```

---

## 3. The 8042 PS/2 Keyboard Driver

In bare-metal mode, keystrokes are processed by [`src/arch/keyboard.rs`](file:///d:/os/src/arch/keyboard.rs):
- **Status Port `0x64`**: Polled for bit 0 (`Output Buffer Full`).
- **Data Port `0x60`**: Reads Scancode Set 1 make/break codes.
- **Modifier Tracking**: Tracks Shift (`0x2A`/`0x36`) and Ctrl (`0x1D`) state to generate uppercase letters, punctuation, and control characters (such as `Ctrl+C`).
- **Extended Scancodes (`0xE0`)**: Decodes arrow keys (`Up`, `Down`, `Left`, `Right`), `PageUp`, `PageDown`, `Home`, and `End` for terminal line editing and history navigation.

---

## 4. Interactive Verification & Shell Command

The microkernel includes an interactive shell command `exitboot` (aliased to `takeover`, `baremetal`, and `hwmode`):

```bash
hydra:/bin> exitboot status
[-] Hardware Mode: INACTIVE (Running under UEFI Boot Services)
    Type 'exitboot' to terminate boot services and take full hardware control.

hydra:/bin> exitboot
[*] Initiating ExitBootServices and Bare-Metal Hardware Takeover...

============================================================
 [+] HYDRA MICROKERNEL: FULL HARDWARE TAKEOVER COMPLETE!    
============================================================
  - UEFI Boot Services : TERMINATED (ExitBootServices invoked)
  - Memory Map Entries : 61 descriptors captured
  - GDT & TSS Status   : Hydra Ring 0 CS=0x8, DS=0x10, TR=0x28
  - Interrupts & IDT   : Hydra 256 gates (Double Fault on TSS IST1)
  - Virtual Memory     : Autonomous PML4 Root at PhysAddr 0x901000
  - Display & Input    : 1024x768 Linear Framebuffer & 8042 PS/2 Keyboard
============================================================
```

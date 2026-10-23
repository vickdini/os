# Interrupt Descriptor Table (IDT) & Hardware Exceptions

- **Source File**: [`src/arch/idt.rs`](file:///d:/os/src/arch/idt.rs)
- **Subsystem**: Architecture / CPU Exception Handling & Interrupt Routing

---

## 1. Conceptual Overview

The **Interrupt Descriptor Table (IDT)** is the x86_64 CPU hardware mechanism that dictates how the processor responds to asynchronous interrupts and synchronous software/hardware exceptions. 

Each entry in the table is a 16-byte gate descriptor defining:
1. **Target Handler Address**: The 64-bit instruction pointer (`RIP`) to jump to when the vector fires.
2. **Segment Selector**: The GDT code segment selector (Hydra Ring 0 Kernel Code Segment).
3. **Gate Type & DPL**: Descriptor Privilege Level (DPL 0 for kernel-only hardware exceptions; DPL 3 for userland-callable traps like `int3`).
4. **Interrupt Stack Table (IST)**: Optional alternate stack pointer index (1..=7) in the TSS, ensuring critical faults execute on guaranteed uncorrupted stack frames.

```
+----------------------------------------------------------------------------+
|                   CPU Interrupt Descriptor Table (IDT)                     |
+----------------------------------------------------------------------------+
| Vector 0..31  : CPU Hardware Exceptions (#DE, #DB, #BP, #UD, #DF, #GP, #PF)|
| Vector 32..47 : Legacy Hardware IRQs (PIC / IO-APIC)                       |
| Vector 48..255: APIC Timers, IPIs, Inter-Core Signaling, System Traps      |
+----------------------------------------------------------------------------+
                                     |
                                     v
                       [Assembly Trampoline Layer]
                                     |
                                     v
                   [Rust Dispatcher: hydra_exception_handler]
```

---

## 2. 64-Bit Interrupt Gate Structure

In Long Mode (64-bit), each IDT descriptor is 16 bytes:

```
 127                                      96 95                     64
+-------------------------------------------+-------------------------+
|                  Reserved                 |     Offset 63..32       |
+-------------------------------------------+-------------------------+
 63       48 47   46 45 44 40 39  35 34  32 31                      0
+-----------+---+------+-----+------+------+-------------------------+
| Offset    | P | DPL  |  0  | Type | 000  | IST  | GDT Selector | Offset 15..0 |
|  31..16   |   |      |     | 1110 |      |      |              |              |
+-----------+---+------+-----+------+------+-------------------------+
```

Hydra models this with `#[repr(C, packed)] pub struct IdtEntry`:
```rust
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}
```

---

## 3. Trampoline Layer & Calling Convention

### Hardware Exception Frame Variations
When a CPU exception occurs:
- Some exceptions push an **Error Code** automatically (`#DF`, `#TS`, `#NP`, `#SS`, `#GP`, `#PF`, `#AC`).
- Other exceptions push **no Error Code** (`#DE`, `#DB`, `#BP`, `#OF`, `#UD`, `#NM`).

To provide a uniform C-compatible context to Rust, assembly macros dynamically push a dummy zero for exceptions that lack hardware error codes:

```nasm
.macro TRAP_NO_ERRCODE name, vec
.global \name
\name:
    push 0                  // Dummy error code
    push \vec               // Vector number
    jmp common_trap_handler
.endm

.macro TRAP_ERRCODE name, vec
.global \name
\name:
    push \vec               // Vector number (error code already pushed by CPU)
    jmp common_trap_handler
.endm
```

### Stack Frame (`ExceptionContext`)
The assembly trampoline pushes all 15 general-purpose registers, creating a complete `ExceptionContext`:

```rust
#[repr(C)]
pub struct ExceptionContext {
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub rbp: u64, pub rbx: u64, pub r11: u64, pub r10: u64,
    pub r9:  u64, pub r8:  u64, pub rdi: u64, pub rsi: u64,
    pub rdx: u64, pub rcx: u64, pub rax: u64,
    pub vector: u64,
    pub error_code: u64,
    pub rip: u64, pub cs: u64, pub rflags: u64, pub rsp: u64, pub ss: u64,
}
```

### UEFI & SysV Calling Convention Compatibility
Because Hydra targets UEFI (`x86_64-unknown-uefi`) while maintaining Unix microkernel foundations, the dispatcher assembly sets up argument passing for both conventions simultaneously:
```nasm
    mov rcx, rsp            // 1st argument in Microsoft x64 / UEFI ABI
    mov rdi, rsp            // 1st argument in System V AMD64 ABI
    sub rsp, 32             // Allocate 32-byte shadow store (Win64) & align stack
    call hydra_exception_handler
    add rsp, 32             // Deallocate shadow store
    pop r15..rax            // Restore registers
    add rsp, 16             // Clean up vector and error code
    iretq                   // Return from interrupt
```

---

## 4. Exception Handlers

### Vector 3: Breakpoint (`#BP`)
- **DPL**: 3 (Permits Ring 3 userland processes to trigger software breakpoints with `int3`).
- **Behavior**: Non-fatal diagnostic trap. Prints RIP, RFLAGS, and RSP, then resumes execution via `iretq`.

### Vector 8: Double Fault (`#DF`)
- **IST**: Linked to TSS IST index 1 (`DOUBLE_FAULT_STACK`).
- **Behavior**: Even if the kernel stack overflows or suffers memory corruption, the hardware forcibly switches to a dedicated 20 KiB emergency stack, preventing immediate triple-fault reboot loops.

### Vector 13: General Protection Fault (`#GP`)
- **Behavior**: Decodes selector indices and privilege boundary violations.

### Vector 14: Page Fault (`#PF`)
- **Behavior**: Reads the faulting linear memory address from hardware control register `CR2` (`Cr2::read_raw()`), and decodes the error code flags:
  - **P (Bit 0)**: Non-present page (0) vs Page-level protection violation (1).
  - **W/R (Bit 1)**: Read access (0) vs Write access (1).
  - **U/S (Bit 2)**: Supervisor mode (0) vs User mode (1).
  - **I/D (Bit 4)**: Data access (0) vs Instruction fetch (1).

---

## 5. Shell Integration & Diagnostics

Hydra includes a built-in shell diagnostic command `trap` (aliased to `int3` and `breakpoint`):

```bash
hydra:/> trap
[*] Executing INT3 instruction to test IDT breakpoint trap...

[i] CPU BREAKPOINT TRAP (#BP) at RIP: 0x1f07f439
    RFLAGS: 0x246, RSP: 0x1ee15bf0
[+] Resumed successfully from #BP exception handler!
```

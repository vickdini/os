# Legacy Assembly Bootstrapper Documentation

- **Source Directory**: [`src/arch/x86_64/boot/`](file:///d:/os/src/arch/x86_64/boot)
- **Files**:
  - [`header.asm`](file:///d:/os/src/arch/x86_64/boot/header.asm)
  - [`main.asm`](file:///d:/os/src/arch/x86_64/boot/main.asm)
  - [`main64.asm`](file:///d:/os/src/arch/x86_64/boot/main64.asm)
- **Subsystem**: Architecture / Legacy BIOS Boot Trampoline

---

## 1. Overview

Before migrating to native **UEFI**, x86 PCs booted via legacy **BIOS**. Under BIOS:
1. The CPU boots in **16-bit Real Mode**.
2. A bootloader (like GRUB) loads the kernel into **32-bit Protected Mode**.
3. The kernel itself is responsible for setting up initial page tables, enabling PAE, switching to **64-bit Long Mode**, and jumping to 64-bit code.

These assembly files implement that transition for reference or hybrid BIOS/GRUB boot media.

---

## 2. File Breakdown

### 1. `header.asm` (Multiboot2 Header)
- Defines the magic numbers and architecture tags required by Multiboot2-compliant bootloaders (such as GRUB).
- Allows GRUB to recognize the kernel file as bootable.

### 2. `main.asm` (32-Bit to 64-Bit Transition)
- **Stack Initialization**: Allocates an initial boot stack.
- **CPU Checks**:
  - `check_multiboot`: Verifies that the bootloader passed the expected Multiboot magic value in `EAX`.
  - `check_cpuid`: Verifies that the CPU supports the `cpuid` instruction.
  - `check_long_mode`: Queries CPUID to verify 64-bit Long Mode hardware support.
- **Identity Paging**: Configures an initial 2 MiB identity-mapped page table so memory addresses map 1:1 during the switch.
- **Enabling Long Mode**: Sets the Physical Address Extension bit (`PAE`) in `CR4`, enables Long Mode in the Extended Feature Enable Register (`EFER` MSR), and enables Paging in `CR0`.
- **GDT Setup**: Loads a temporary 64-bit code segment and performs a far jump to `long_mode_start`.

### 3. `main64.asm` (64-Bit Trampoline)
- Re-zeros data segment registers (`DS`, `ES`, `FS`, `GS`, `SS`).
- Invokes the Rust 64-bit entry point (`call kernel_main`).
- Halts the CPU if the function ever returns.

# Build Script Documentation

- **Source File**: [`build.rs`](file:///d:/os/build.rs)
- **Subsystem**: Toolchain & Assembly Build Automation

---

## 1. Purpose & Context

In earlier phases or legacy configurations, Hydra supported a hybrid BIOS/Multiboot bootloader mode that required compiling 16-bit and 32-bit assembly files (`header.asm`, `main.asm`, `main64.asm`) with the `nasm` assembler before linking into ELF objects.

In current native UEFI builds, Cargo compiles Rust directly targeting `x86_64-unknown-uefi` without requiring external assembly compilation, with `build = false` in [`Cargo.toml`](file:///d:/os/Cargo.toml).

---

## 2. Key Functions

### `assemble_file(file: &str, output_dir: &str)`
1. Checks the `DEBUG` environment variable to determine whether to emit DWARF debug symbols (`-g`).
2. Invokes `nasm` to compile an `.asm` source file into an ELF64 object (`.o`).
3. Instructs Cargo to re-run the build script if the source assembly file changes (`cargo:rerun-if-changed`).
4. Tells the Rust linker to link the resulting object file (`cargo:rustc-link-object`).

### `assemble_all(output_dir: &str)`
Loops through the standard assembly boot files:
- `boot/header.asm`: Multiboot2 header definition.
- `boot/main.asm`: 32-bit protected mode entry point, page table setup, and long mode switch.
- `boot/main64.asm`: 64-bit trampoline to Rust entry point.

# CPU Control & Register Diagnostics Documentation

- **Source File**: [`src/arch/cpu.rs`](file:///d:/os/src/arch/cpu.rs)
- **Subsystem**: Architecture / CPU Control Registers

---

## 1. Conceptual Overview

On x86_64 processors, the processor's operating mode, memory translation mechanisms, and security features are governed by specialized **Control Registers** (`CR0`, `CR2`, `CR3`, `CR4`):

- **CR0**: Controls basic execution features: Protected Mode, Paging enable, Write Protect (prevents kernel from inadvertently overwriting read-only user pages).
- **CR2**: Stores the linear virtual address that triggered the most recent **Page Fault** exception. Vital for demand paging and memory debugging.
- **CR3**: Points to the base physical address of the Level 4 Page Table (PML4). Loading a new value into CR3 immediately switches active virtual address spaces.
- **CR4**: Controls advanced architecture features: Physical Address Extension (PAE), Page Global Enable (PGE), OS support for SSE/AVX floating point, and User-Mode Instruction Prevention (UMIP).

---

## 2. The `CpuState` Struct

```rust
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    pub cr0: Cr0Flags,
    pub cr2: u64,
    pub cr3_frame: PhysAddr,
    pub cr4: Cr4Flags,
}
```

Captures an instantaneous snapshot of the machine's hardware configuration registers.

---

## 3. API Reference

### `arch::cpu::read_state() -> CpuState`
Queries the CPU hardware registers and packages them into a `CpuState` diagnostic struct.

```rust
use crate::arch::cpu;

let cpu = cpu::read_state();
println!("Paging root physical address: {:#x}", cpu.cr3_frame.as_u64());
```

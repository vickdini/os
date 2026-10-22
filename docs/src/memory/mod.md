# Memory Subsystem Overview

- **Source File**: [`src/memory/mod.rs`](file:///d:/os/src/memory/mod.rs)
- **Subsystem**: Memory Management

---

## 1. Role in Hydra OS

The `memory` module coordinates all physical and virtual memory resources required by the kernel and user processes.

It abstracts low-level hardware memory mechanisms (UEFI memory descriptors, x86_64 paging tables, MMU control registers) into safe, modular Rust interfaces.

---

## 2. Submodules

- [`frame_allocator.rs`](file:///d:/os/docs/src/memory/frame_allocator.md): Manages physical memory frames (4 KiB) discovered from the UEFI firmware memory map.
- [`paging.rs`](file:///d:/os/docs/src/memory/paging.md): Manages virtual memory address spaces and implements 4-level PML4 address translation.

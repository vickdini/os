# Hydra OS - Codebase Documentation Index

Welcome to the technical documentation for **Hydra OS**. Every source code file in the repository is documented with real-world analogies, conceptual explanations, architectural diagrams, and API breakdowns designed to be accessible to anyone with general programming knowledge.

---

## 🗺️ Documentation Directory Map

```
docs/
├── README.md                      # This Index
├── build.md                       # Build automation script (build.rs)
└── src/
    ├── main.md                    # Kernel entry point & bootstrap orchestrator (src/main.rs)
    ├── console.md                 # Thread-safe console & formatting macros (src/console.rs)
    ├── arch/
    │   ├── mod.md                 # Architecture-specific module root (src/arch/mod.rs)
    │   ├── gdt.md                 # Global Descriptor Table, TSS & Rings (src/arch/gdt.rs)
    │   ├── cpu.md                 # Control registers & CPU diagnostics (src/arch/cpu.rs)
    │   └── x86_64/
    │       └── boot.md            # Legacy BIOS boot assembly (header.asm, main.asm, main64.asm)
    ├── memory/
    │   ├── mod.md                 # Memory management module root (src/memory/mod.rs)
    │   ├── frame_allocator.md     # Physical memory manager & 4 KiB frames (src/memory/frame_allocator.rs)
    │   └── paging.md              # Virtual memory & 4-level PML4 translation (src/memory/paging.rs)
    ├── capability/
    │   ├── mod.md                 # Capability types & Rights bitflags (src/capability/mod.rs)
    │   └── table.md               # Task capability table & CSpace (src/capability/table.rs)
    └── ipc/
        ├── mod.md                 # IPC subsystem root (src/ipc/mod.rs)
        ├── message.md             # Fast register payload & transferred caps (src/ipc/message.rs)
        └── endpoint.md            # Message rendezvous channels (src/ipc/endpoint.rs)
```

---

## 📚 Module Reference Table

| Subsystem | Source File | Documentation | Key Concepts & Analogies |
| :--- | :--- | :--- | :--- |
| **Boot & Main** | [`src/main.rs`](file:///d:/os/src/main.rs) | [docs/src/main.md](file:///d:/os/docs/src/main.md) | Entry point, boot sequence, hardware diagnostics, and panic handler. |
| **Console I/O** | [`src/console.rs`](file:///d:/os/src/console.rs) | [docs/src/console.md](file:///d:/os/docs/src/console.md) | Thread-safe `print!` / `println!` macros, UEFI screen color formatting. |
| **GDT & TSS** | [`src/arch/gdt.rs`](file:///d:/os/src/arch/gdt.rs) | [docs/src/arch/gdt.md](file:///d:/os/docs/src/arch/gdt.md) | Ring 0 / Ring 3 privilege levels, Task State Segment, double-fault stack. |
| **CPU Control** | [`src/arch/cpu.rs`](file:///d:/os/src/arch/cpu.rs) | [docs/src/arch/cpu.md](file:///d:/os/docs/src/arch/cpu.md) | Control registers (`CR0`, `CR2`, `CR3`, `CR4`), paging roots. |
| **Arch Root** | [`src/arch/mod.rs`](file:///d:/os/src/arch/mod.rs) | [docs/src/arch/mod.md](file:///d:/os/docs/src/arch/mod.md) | Architecture abstraction layer for hardware independence. |
| **Physical Memory**| [`src/memory/frame_allocator.rs`](file:///d:/os/src/memory/frame_allocator.rs) | [docs/src/memory/frame_allocator.md](file:///d:/os/docs/src/memory/frame_allocator.md) | UEFI memory map parsing, 4 KiB frame allocator ("parking lot attendant"). |
| **Virtual Memory** | [`src/memory/paging.rs`](file:///d:/os/src/memory/paging.rs) | [docs/src/memory/paging.md](file:///d:/os/docs/src/memory/paging.md) | 4-level x86_64 paging tree (PML4), address translation, page protection. |
| **Memory Root** | [`src/memory/mod.rs`](file:///d:/os/src/memory/mod.rs) | [docs/src/memory/mod.md](file:///d:/os/docs/src/memory/mod.md) | Physical and virtual memory coordinator. |
| **Capability Core**| [`src/capability/mod.rs`](file:///d:/os/src/capability/mod.rs) | [docs/src/capability/mod.md](file:///d:/os/docs/src/capability/mod.md) | Security keycards (`Rights::READ`, `WRITE`, `EXECUTE`, `GRANT`, `CALL`). |
| **Capability Table**| [`src/capability/table.rs`](file:///d:/os/src/capability/table.rs) | [docs/src/capability/table.md](file:///d:/os/docs/src/capability/table.md) | Per-task CSpace keyring, permission checking, access revocation. |
| **IPC Message** | [`src/ipc/message.rs`](file:///d:/os/src/ipc/message.rs) | [docs/src/ipc/message.md](file:///d:/os/docs/src/ipc/message.md) | Registered courier package: 4 fast inline register arguments + security token. |
| **IPC Endpoint** | [`src/ipc/endpoint.rs`](file:///d:/os/src/ipc/endpoint.rs) | [docs/src/ipc/endpoint.md](file:///d:/os/docs/src/ipc/endpoint.md) | Postal mailbox rendezvous point, FIFO circular ring buffer. |
| **IPC Root** | [`src/ipc/mod.rs`](file:///d:/os/src/ipc/mod.rs) | [docs/src/ipc/mod.md](file:///d:/os/docs/src/ipc/mod.md) | Inter-process communication module export. |
| **Build Automation**| [`build.rs`](file:///d:/os/build.rs) | [docs/build.md](file:///d:/os/docs/build.md) | NASM assembly build script and link configuration. |
| **Legacy Boot** | [`src/arch/x86_64/boot/`](file:///d:/os/src/arch/x86_64/boot) | [docs/src/arch/x86_64/boot.md](file:///d:/os/docs/src/arch/x86_64/boot.md) | Multiboot2 header and 32-bit to 64-bit assembly trampoline. |

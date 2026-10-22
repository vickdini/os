# Capability Subsystem Overview

- **Source File**: [`src/capability/mod.rs`](file:///d:/os/src/capability/mod.rs)
- **Subsystem**: Capability-Based Security

---

## 1. Conceptual Overview

Traditional operating systems (like standard Linux or Windows) often rely on **Access Control Lists (ACLs)** or ambient authority (e.g., "this process runs as root/administrator, so it can touch anything"). If an administrative process has a bug or is hijacked, the entire system is compromised.

**Hydra uses Capability-Based Security** (similar to seL4 and Fuchsia Zircon).

### The Real-World Analogy
Think of a **Capability** as a non-forgeable, digital smart keycard:
- You cannot just claim "I am the administrator".
- You can only perform an operation if you hold a specific keycard that explicitly grants the right to perform that action on that exact resource.
- You can hand a copy of a keycard with restricted permissions to another process (e.g., giving someone a key that allows reading a file, but not modifying or deleting it).

---

## 2. The `Rights` Bitflags

`Rights` defines what actions a process holding a capability is permitted to execute:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rights(pub u32);
```

### Supported Rights Flags

| Flag | Value | Description |
| :--- | :--- | :--- |
| `Rights::READ` | `1 << 0` (1) | Permission to inspect data in the target object. |
| `Rights::WRITE` | `1 << 1` (2) | Permission to modify or overwrite the target object. |
| `Rights::EXECUTE` | `1 << 2` (4) | Permission to execute machine instructions from the object (memory frame). |
| `Rights::GRANT` | `1 << 3` (8) | Permission to delegate, duplicate, or transfer this capability to another process. |
| `Rights::CALL` | `1 << 4` (16)| Permission to invoke or call an IPC Endpoint. |
| `Rights::ALL` | `0x1F` (31) | Full combined permissions (`READ \| WRITE \| EXECUTE \| GRANT \| CALL`). |

### Checking Permissions
- `rights.contains(required)`: Returns `true` if all bits in `required` are present.

---

## 3. The `CapType` Enum

`CapType` defines what kind of physical or virtual kernel resource the capability represents:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapType {
    UntypedMemory { size_bytes: usize },
    MemoryFrame { physical_addr: u64 },
    AddressSpace { root_cr3: u64 },
    Endpoint { endpoint_id: u32 },
    Thread { thread_id: u32 },
    HardwareDevice { device_id: u32 },
}
```

- **`UntypedMemory`**: Raw, unallocated physical memory blocks that can be carved up into specific objects.
- **`MemoryFrame`**: A specific 4096-byte page of physical RAM.
- **`AddressSpace`**: A virtual memory context (represented by the x86_64 CR3 root page table).
- **`Endpoint`**: A communication mailbox for IPC.
- **`Thread`**: A schedulable execution thread.
- **`HardwareDevice`**: An I/O hardware device (such as a GPU, network card, or timer).

---

## 4. The `Capability` Struct

```rust
#[derive(Debug, Clone, Copy)]
pub struct Capability {
    pub cap_type: CapType,
    pub rights: Rights,
}
```

Combines the resource descriptor (`cap_type`) with the allowable permissions (`rights`).

### Constructor
```rust
let memory_cap = Capability::new(
    CapType::MemoryFrame { physical_addr: 0x1000 },
    Rights(Rights::READ.0 | Rights::WRITE.0)
);
```

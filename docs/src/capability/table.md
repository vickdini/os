# Capability Table Module Documentation

- **Source File**: [`src/capability/table.rs`](file:///d:/os/src/capability/table.rs)
- **Subsystem**: Capability-Based Security

---

## 1. Conceptual Overview

In Hydra, userland processes cannot store raw pointers to kernel objects or hardware devices. Instead, the kernel maintains a **Capability Table** (also known as a **CSpace** or **CNode**) on behalf of each process.

### The Real-World Analogy
Think of a **Capability Table** like a keyring or safe-deposit locker:
- The process holds numbers (slots: `0`, `1`, `2`, ...).
- When a process asks the kernel, *"Read from memory at Slot 3"*, the kernel checks Slot 3 in that process's Capability Table.
- If Slot 3 contains a valid capability and includes `Rights::READ`, the kernel performs the operation.
- If the slot is empty or lacks the required right, the kernel immediately denies the request.
- The process never touches the raw physical address or hardware pointer directly, preventing spoofing and unauthorized access.

---

## 2. Architecture & Constants

```
+--------------------------------------------------------+
|                   CapabilityTable                      |
+--------------------------------------------------------+
| Slot 0:  [ Some(Cap: AddressSpace, Rights::ALL)      ] |
| Slot 1:  [ Some(Cap: Endpoint #1,  READ|WRITE|CALL)  ] |
| Slot 2:  [ None                                      ] |
| ...                                                    |
| Slot 255:[ None                                      ] |
+--------------------------------------------------------+
```

- `MAX_CAPABILITIES: usize = 256`: Each table can hold up to 256 active capabilities per task in fixed array storage, avoiding dynamic heap allocation.

---

## 3. The `CapabilityTable` Struct

```rust
pub struct CapabilityTable {
    slots: [Option<Capability>; MAX_CAPABILITIES],
    next_free_slot: usize,
}
```

- `slots`: Fixed array of 256 optional capability slots.
- `next_free_slot`: Circular search cursor to quickly find available slots upon insertion.

---

## 4. API Reference & Methods

### `CapabilityTable::new() -> Self`
Creates a new, empty capability table where all 256 slots are `None`.

### `CapabilityTable::insert(&mut self, cap: Capability) -> Result<usize, &'static str>`
Allocates the first available empty slot, stores the capability, and returns the slot index (`0..255`).
- **Returns `Ok(slot_index)`** on success.
- **Returns `Err("Capability table full")`** if all 256 slots are currently occupied.

### `CapabilityTable::get(&self, slot: usize) -> Option<&Capability>`
Retrieves a read-only reference to the capability stored at the specified slot.
- Returns `None` if the slot is out of bounds or empty.

### `CapabilityTable::revoke(&mut self, slot: usize) -> Option<Capability>`
Removes and clears the capability at `slot`, effectively revoking the process's access to that resource.

### `CapabilityTable::check_rights(&self, slot: usize, required_rights: Rights) -> bool`
Helper method that verifies:
1. Whether `slot` contains an active capability.
2. Whether the capability possesses all bits defined in `required_rights`.

---

## 5. Practical Example

```rust
use crate::capability::table::CapabilityTable;
use crate::capability::{Capability, CapType, Rights};

let mut cspace = CapabilityTable::new();

// 1. Grant access to a hardware memory frame
let frame_cap = Capability::new(
    CapType::MemoryFrame { physical_addr: 0x2000_0000 },
    Rights(Rights::READ.0 | Rights::WRITE.0)
);

// 2. Insert into the process table
let slot = cspace.insert(frame_cap).expect("Table has free slots");

// 3. Verify permissions before performing an operation
if cspace.check_rights(slot, Rights::WRITE) {
    // Permission granted: perform write
} else {
    // Permission denied: trigger error
}
```

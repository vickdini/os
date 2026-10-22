# IPC Message Module Documentation

- **Source File**: [`src/ipc/message.rs`](file:///d:/os/src/ipc/message.rs)
- **Subsystem**: Microkernel IPC (Inter-Process Communication)

---

## 1. Conceptual Overview

In a microkernel operating system like **Hydra**, core operating system services (such as file systems, network drivers, and device managers) run as separate, isolated programs rather than inside a single monolithic kernel. Because these isolated programs cannot directly access each other's memory, they must communicate by sending **Messages** across a secured boundary.

### The Real-World Analogy
Think of an **IPC Message** like a registered letter or courier package:
1. **The Label (`label`)**: The subject line or action request (e.g., "Read File", "Get Time", "Allocate Memory").
2. **The Fast Payload (`args`)**: A quick handwritten note with up to 4 numbers or pointers that fit directly into the CPU's registers for high-speed delivery without touching RAM.
3. **The Enclosed Key/Passport (`capability`)**: An optional security token granted to the receiver, giving them special permission to access a specific memory buffer, device, or channel.

---

## 2. Architecture & Data Structures

```
+-------------------------------------------------------------+
|                        Message                              |
+-------------------------------------------------------------+
| label: u32                | Protocol / Syscall ID           |
| args: [u64; 4]            | Up to 4 fast 64-bit parameters  |
| capability: Option<Cap>   | Optional transferred security   |
|                           | token / permission              |
+-------------------------------------------------------------+
```

### Constants

- `MAX_INLINE_WORDS: usize = 4`: The number of 64-bit arguments (`u64`) carried directly within a message. Using 4 words (32 bytes) allows the microkernel to transfer arguments directly using CPU registers (such as `RDI`, `RSI`, `RDX`, `RCX` on x86_64), making IPC fast.

---

## 3. The `Message` Struct

```rust
#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub label: u32,
    pub args: [u64; MAX_INLINE_WORDS],
    pub capability: Option<Capability>,
}
```

### Fields Explained

| Field | Type | Description |
| :--- | :--- | :--- |
| `label` | `u32` | An operation code indicating what action is requested (e.g. `0x1001` for ping, `0x2001` for read). |
| `args` | `[u64; 4]` | Array of 4 inline 64-bit numbers. Can be integers, memory offsets, or byte lengths. |
| `capability`| `Option<Capability>` | An optional security token transferred along with the message to delegate authority. |

---

## 4. API Reference & Methods

### `Message::new(label: u32) -> Self`
Creates a new message with the given operation label. All 4 arguments are initialized to `0`, and no capability is attached.

```rust
let msg = Message::new(0x01); // Simple ping message
```

### `Message::with_args(label: u32, args: [u64; MAX_INLINE_WORDS]) -> Self`
Creates a new message with an operation label and up to 4 inline arguments.

```rust
// Example: requesting to read 512 bytes starting at offset 1024
let msg = Message::with_args(0x1002, [1024, 512, 0, 0]);
```

### `Message::with_cap(mut self, cap: Capability) -> Self`
Builder pattern method to attach a capability to an existing message. This transfers access rights to the receiving process.

```rust
let msg = Message::new(0x2000).with_cap(shared_memory_cap);
```

---

## 5. Practical Example

```rust
use crate::capability::{Capability, CapType, Rights};
use crate::ipc::Message;

// 1. Create a capability granting READ access to an endpoint
let ep_cap = Capability::new(CapType::Endpoint { endpoint_id: 10 }, Rights::READ);

// 2. Prepare an IPC message carrying data and delegating the capability
let message = Message::with_args(0x1001, [42, 1337, 0, 0])
    .with_cap(ep_cap);

// 3. The message is now ready to be sent over an Endpoint!
```

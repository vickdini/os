# IPC Endpoint Module Documentation

- **Source File**: [`src/ipc/endpoint.rs`](file:///d:/os/src/ipc/endpoint.rs)
- **Subsystem**: Microkernel IPC (Inter-Process Communication)

---

## 1. Conceptual Overview

In a microkernel architecture, programs cannot talk directly to each other because their address spaces are completely isolated by the CPU's memory management unit (MMU). An **Endpoint** acts as a meeting place (rendezvous point) or postal mailbox managed by the kernel.

### The Real-World Analogy
Imagine an apartment building where residents cannot enter each other's apartments:
- An **Endpoint** is a secure PO Box in the lobby.
- Process A drops a [`Message`](file:///d:/os/docs/src/ipc/message.md) into the box (`send`).
- Process B checks the box and retrieves the message (`receive`).
- The kernel ensures only residents holding the correct security key ([`Capability`](file:///d:/os/docs/src/capability/mod.md)) can post or retrieve items from that specific box.

---

## 2. Architecture & Data Structures

```
                 +-----------------------------------------+
                 |         SharedEndpoint (Mutex)          |
                 +-----------------------------------------+
                                      |
                                      v
  [Process A] ---> send() --->  [ Ring Buffer ]  ---> receive() ---> [Process B]
  (Sender)                       Slots: 0..16                         (Receiver)
```

### Queue Parameters
- `QUEUE_CAPACITY: usize = 16`: The maximum number of unread messages the endpoint buffer can hold before rejecting new incoming messages with an error.

---

## 3. Struct Details

### `Endpoint`
The core internal endpoint managing a circular ring buffer of messages.

```rust
pub struct Endpoint {
    id: u32,
    buffer: [Option<Message>; QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
}
```

- `id`: A unique numerical identifier for this endpoint.
- `buffer`: Fixed-size array holding pending messages without needing dynamic heap allocations.
- `head`: Read pointer pointing to the next message to be consumed (FIFO order).
- `tail`: Write pointer pointing to the next free slot for an incoming message.
- `count`: Current number of queued, unread messages.

### `SharedEndpoint`
A thread-safe wrapper around `Endpoint` using a `spin::Mutex`. This allows concurrent threads and interrupts to safely send and receive messages without race conditions.

```rust
pub struct SharedEndpoint(Mutex<Endpoint>);
```

---

## 4. API Reference & Methods

### `Endpoint::new(id: u32) -> Self`
Initializes a new empty message queue with the specified ID.

### `Endpoint::send(&mut self, msg: Message) -> Result<(), &'static str>`
Pushes a message into the queue:
- **Returns `Ok(())`** if the message was placed into the buffer.
- **Returns `Err("Endpoint message queue is full")`** if all 16 slots are occupied.

### `Endpoint::receive(&mut self) -> Option<Message>`
Pulls the oldest message out of the queue (FIFO):
- **Returns `Some(Message)`** if a message is available.
- **Returns `None`** if the queue is currently empty.

### `Endpoint::pending_count(&self) -> usize`
Returns how many messages are waiting to be read.

---

## 5. Practical Example

```rust
use crate::ipc::{Endpoint, Message};

// 1. Create a communication endpoint with ID 1
let mut mailbox = Endpoint::new(1);

// 2. Sender prepares and sends a message
let request = Message::with_args(0x100, [1, 2, 3, 4]);
mailbox.send(request).expect("Queue has space");

// 3. Check queue status
assert_eq!(mailbox.pending_count(), 1);

// 4. Receiver reads the message
if let Some(msg) = mailbox.receive() {
    // Process msg.label and msg.args...
}
```

# 8042 PS/2 Keyboard Driver & Input Subsystem

## 1. Architectural Role
The **8042 Keyboard Controller** provides Hydra with independent, firmware-free human interface input. Operating on standard I/O ports `0x60` (Data) and `0x64` (Status/Command), it decodes raw hardware scancodes into high-level keyboard events.

Following Phase 1, the driver operates in dual mode:
- **Interrupt-Driven Mode (Hardware Mode)**: IRQ 1 (IDT Vector 33) captures scancodes asynchronously into a 128-key circular ring buffer (`KeyRingBuffer`), enabling low-power `hlt` execution while waiting for input.
- **Polling Fallback**: Reads status bit 0 from port `0x64` when hardware interrupts are not yet active (e.g. early kernel bootstrap).

---

## 2. Port Architecture

| Port | Direction | Function |
|:---:|:---:|:---|
| **`0x60`** | Read | Data Port (receives raw scancodes from keyboard) |
| **`0x64`** | Read | Status Register (Bit 0 = Output Buffer Full / Byte Ready) |

---

## 3. Scancode Set 1 & Modifier Tracking

The driver tracks modifier state machines for:
- **Shift (`0x2A` / `0x36`)**: Make/Break tracking alters character case and symbols.
- **Ctrl (`0x1D`)**: Translates ASCII letters into control characters `^A`..`^Z` (ASCII `1..26`) for interrupt signaling (`Ctrl+C`, `Ctrl+S`).
- **Extended Prefix (`0xE0`)**: Signals two-byte extended navigation keys (Arrow keys, Home, End, Delete, PageUp, PageDown).

---

## 4. Circular Ring Buffer & Low-Power Halts

```text
  [Hardware IRQ 1]
         │
         ▼
  asm_irq_33 -> hydra_exception_handler(33)
         │
         ▼
  keyboard::on_keyboard_irq() ──► Reads Port 0x60
                                      │
                                      ▼
                             process_scancode()
                                      │
                                      ▼
                         KEY_BUFFER.push(Key)
                                      │
                                      ▼
                             pic::send_eoi(1)
```

When the interactive shell waits for user input, `get_key()` invokes `x86_64::instructions::hlt()`. The CPU enters a sleep state until the next timer tick (IRQ 0) or keystroke (IRQ 1) wakes the core, reducing idle power consumption to near zero.

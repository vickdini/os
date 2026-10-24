# 8259 Programmable Interrupt Controller (PIC)

## 1. Architectural Role
The **8259 Programmable Interrupt Controller (PIC)** is the legacy interrupt routing hardware in standard PC/AT architectures. It aggregates up to 15 hardware IRQs (Interrupt Request lines) across two cascading controllers and forwards them to the CPU via the `INTR` line.

In standard x86 BIOS systems, hardware IRQs initially collide with CPU exception vectors (`0x00–0x1F`). Hydra remaps the dual-PIC controllers to vectors `0x20–0x2F` upon hardware takeover (`exitboot`), ensuring clear separation between CPU faults and hardware peripherals.

---

## 2. Port & Vector Mapping

| Controller | Command Port | Data Port | Vector Range | IRQ Lines |
|:---|:---:|:---:|:---:|:---|
| **Master PIC** | `0x20` | `0x21` | `0x20..0x27` (32..39) | IRQ 0..7 (Timer, PS/2 Keyboard, Cascade, etc.) |
| **Slave PIC**  | `0xA0` | `0xA1` | `0x28..0x2F` (40..47) | IRQ 8..15 (RTC, ACPI, Primary/Secondary ATA) |

---

## 3. Initialization & Remapping Sequence

Initialization uses a sequence of 4 Initialization Command Words (**ICWs**):

```text
ICW1 (0x11): Start initialization in cascade mode, expect ICW4
   │
   ├─► ICW2: Set base vector offset
   │     Master = 0x20 (32)
   │     Slave  = 0x28 (40)
   │
   ├─► ICW3: Cascade wiring setup
   │     Master: bit 2 = 1 (Slave attached at IRQ 2)
   │     Slave : cascade identity 0x02
   │
   └─► ICW4 (0x01): 8086/88 operation mode
```

---

## 4. Interrupt Masking & End of Interrupt (EOI)

- **Default Masks**:
  - `Master Mask`: `0xF8` (Unmasks IRQ 0 Timer, IRQ 1 Keyboard, IRQ 2 Cascade).
  - `Slave Mask`: `0xFF` (All slave IRQs masked).
- **End of Interrupt (EOI)**:
  - When an interrupt is serviced, an EOI byte (`0x20`) must be written to the PIC command port.
  - If the IRQ originated from the Slave PIC (`irq >= 8`), an EOI must be sent to **both** Slave (`0xA0`) and Master (`0x20`).

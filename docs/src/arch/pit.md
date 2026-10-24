# 8254 Programmable Interval Timer (PIT)

## 1. Architectural Role
The **Intel 8254 Programmable Interval Timer (PIT)** provides Hydra with a hardware clock source for measuring wall-clock time, coordinating preemptive multitasking ticks, and executing low-power sleeps.

The oscillator operates at an input frequency of **1,193,182 Hz** (~1.193182 MHz). Channel 0 is wired directly to IRQ 0 on the Master PIC.

---

## 2. Configuration & Ports

- **Port `0x40`**: Channel 0 Data Port (read/write counter divisor)
- **Port `0x43`**: Mode/Command Register

Hydra configures Channel 0 in **Mode 3 (Square Wave Generator)**:
- Command Byte: `0x36` (`00` Channel 0 | `11` Access lo/hi byte | `011` Mode 3 | `0` 16-bit binary)
- Target Frequency: **100 Hz** (10 ms per tick)
- Divisor: `1193182 / 100 = 11932` (`0x2E9C`)

---

## 3. Tick Counter & Low-Power Halts

On every IRQ 0 (Vector 32) interrupt:
1. `pit::on_timer_tick()` increments a 64-bit atomic counter `TICKS`.
2. The PIC is notified via `pic::send_eoi(0)`.
3. Routines like `sleep_ms()` check `ticks() < target` and execute `hlt` instructions, keeping CPU usage near 0% while waiting.

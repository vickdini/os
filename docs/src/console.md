# Console & Diagnostic Output Documentation

- **Source File**: [`src/console.rs`](file:///d:/os/src/console.rs)
- **Subsystem**: Core I/O & Formatting

---

## 1. Conceptual Overview

In a bare-metal `#![no_std]` environment, Rust's standard library (`std::io`, `std::println!`) does not exist because there is no underlying operating system yet.

The `console` module bridges Rust's `core::fmt` formatting system directly to UEFI's standard text output protocol (`SimpleTextOutput`), providing thread-safe `print!` and `println!` macros that can be used everywhere across the kernel.

---

## 2. Architecture & Concurrency

```
  Any Kernel Thread / Subsystem
               |
               v
  println!("Text: {}", value)
               |
               v
       CONSOLE (Mutex)  <--- Thread-safe synchronization via spin::Mutex
               |
               v
     uefi::system::with_stdout
               |
               v
     Physical Video Display / UEFI Console Screen
```

- **`spin::Mutex`**: Used instead of standard OS mutexes because interrupts or threads can acquire the lock without requiring OS scheduling services.

---

## 3. API Reference & Methods

### `print!` and `println!` Macros
Drop-in replacements for standard Rust printing macros.

```rust
print!("Initializing subsystem...");
println!("Done: {} frames free.", free_count);
```

### `console::set_color(foreground: Color, background: Color)`
Changes the active text color and background color on the UEFI screen.

```rust
use uefi::proto::console::text::Color;

console::set_color(Color::LightGreen, Color::Black);
println!("[+] Operation Succeeded");
```

### `console::clear_screen()`
Clears the active screen buffer and resets the cursor to the top-left coordinate `(0, 0)`.

### `console::CONSOLE` and `_print`
```rust
pub static CONSOLE: Mutex<Console> = Mutex::new(Console);

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut console = CONSOLE.lock();
    let _ = console.write_fmt(args);
}
```
`CONSOLE` is a globally accessible, mutex-locked unit struct implementing `core::fmt::Write`. The `print!` and `println!` macros format arguments and invoke `_print`, acquiring the lock to safely serialize output to the UEFI `SimpleTextOutput` protocol.

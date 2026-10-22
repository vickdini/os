# Kernel Entry Point & Bootstrapping Documentation

- **Source File**: [`src/main.rs`](file:///d:/os/src/main.rs)
- **Subsystem**: Microkernel Bootstrap & Initialization Orchestrator

---

## 1. Conceptual Overview

[`src/main.rs`](file:///d:/os/src/main.rs) is the primary entry point executed when the UEFI firmware finishes hardware Power-On Self-Test (POST) and transfers control to `BOOTX64.EFI`.

Because Hydra targets native 64-bit UEFI directly:
- The CPU is already in **64-bit Long Mode**.
- Paging is enabled by the UEFI firmware.
- The kernel receives a valid `uefi::prelude::SystemTable` reference, enabling it to access firmware services before taking full ownership of the machine.

---

## 2. Boot Execution Pipeline

```mermaid
graph TD
    A[UEFI Firmware POST] --> B[main entry point]
    B --> C[Initialize Console & Clear Screen]
    C --> D[arch::gdt::init: Load GDT, TSS, Ring 0/3 Segments]
    D --> E[arch::cpu::read_state: Inspect CR0, CR3 PML4 Root, CR4]
    E --> F[memory::frame_allocator: Parse UEFI Memory Map & Test Frame]
    F --> G[memory::paging: Test PML4 Virtual-to-Physical Translation]
    G --> H[capability: Setup Task CSpace & Capability Table]
    H --> I[ipc: Test Endpoint Message Rendezvous]
    I --> J[Halt Loop with Interrupts Disabled]
```

---

## 3. Step-by-Step Code Walkthrough

### The Panic Handler
```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    console::set_color(Color::LightRed, Color::Black);
    println!("\n[!] KERNEL PANIC: {:?}", _info);
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}
```
If an assertion fails or an unexpected unrecoverable condition occurs, the panic handler prints diagnostic error messages in red to the UEFI console, disables interrupts to prevent re-entrancy, and halts the CPU in a low-power infinite loop (`hlt`).

---

### Step 1: Segmentation & Hardware Privilege Setup
```rust
console::set_color(Color::Yellow, Color::Black);
println!("[*] Step 1: Initializing GDT, TSS & Ring 3 Segments...");
arch::gdt::init();
console::set_color(Color::LightGreen, Color::Black);
println!("[+] GDT loaded successfully.");
```
Disables interrupts and installs Hydra's custom Global Descriptor Table and Task State Segment, configuring the CPU for both supervisor (Ring 0) and isolated userland (Ring 3) execution.

---

### Step 2: CPU Diagnostics
```rust
let cpu = arch::cpu::read_state();
println!("    - CR0: {:?}", cpu.cr0);
println!("    - CR3 (Active PML4 Root): {:#x}", cpu.cr3_frame.as_u64());
println!("    - CR4: {:?}", cpu.cr4);
```
Reads control registers to log the active hardware configuration, including the physical address of the initial root page table (`CR3`).

---

### Step 3: Physical Memory Discovery & Frame Allocation
```rust
console::set_color(Color::Yellow, Color::Black);
println!("\n[*] Step 2: Discovering Memory & Initializing Frame Allocator...");
memory::frame_allocator::init_memory_from_uefi();

let (usable_mb, allocated_kb) = {
    let alloc = memory::frame_allocator::FRAME_ALLOCATOR.lock();
    (
        alloc.total_usable_memory() / (1024 * 1024),
        alloc.allocated_memory() / 1024,
    )
};
```
Queries UEFI firmware for the memory map, identifies all conventional RAM regions (skipping the first 1 MiB to protect BIOS/IVT data and null pointers), and initializes the kernel's physical frame allocator. It then tests allocating a sample 4 KiB physical frame.

---

### Step 4: Virtual Memory Verification
```rust
console::set_color(Color::Yellow, Color::Black);
println!("\n[*] Step 3: Verifying 4-Level Paging (PML4) Address Translation...");
let address_space = memory::paging::AddressSpace::active();
let sample_virt = VirtAddr::new(main as *const () as usize as u64);
let sample_phys = unsafe { address_space.translate_addr(sample_virt) };
```
Walks the active 4-level PML4 page table hierarchy to verify that virtual addresses successfully resolve to physical memory frames, accommodating 4 KiB, 2 MiB, and 1 GiB page mappings.

---

### Step 5: Capability Table Initialization
```rust
console::set_color(Color::Yellow, Color::Black);
println!("\n[*] Step 4: Initializing Task Capability Space...");
let mut cap_table = CapabilityTable::new();
let root_cap = Capability::new(
    CapType::AddressSpace {
        root_cr3: cpu.cr3_frame.as_u64(),
    },
    Rights::ALL,
);
let slot0 = cap_table.insert(root_cap).unwrap();
let ep_cap = Capability::new(
    CapType::Endpoint { endpoint_id: 1 },
    Rights(Rights::READ.0 | Rights::WRITE.0 | Rights::CALL.0),
);
let slot1 = cap_table.insert(ep_cap).unwrap();
```
Establishes the process capability table (CSpace), assigning granular access tokens to kernel resources (Root CR3 address space and IPC Endpoint #1).

---

### Step 6: Microkernel IPC Rendezvous
```rust
console::set_color(Color::Yellow, Color::Black);
println!("\n[*] Step 5: Testing Microkernel IPC Rendezvous...");
let mut endpoint = Endpoint::new(1);
let msg = Message::with_args(0x1001, [42, 1337, 0, 0]).with_cap(ep_cap);
endpoint.send(msg).unwrap();
if let Some(received) = endpoint.receive() { ... }
```
Demonstrates inter-process communication: sending a message with inline arguments and an attached delegated capability, followed by receiving and consuming the message from the endpoint queue.

---

### Step 7: Subsystems Operational & Terminal Halt Loop
```rust
console::set_color(Color::LightCyan, Color::Black);
println!("\n============================================================");
println!(" [+] HYDRA MICROKERNEL PHASE 1 SUBSYSTEMS OPERATIONAL!      ");
println!("============================================================");

x86_64::instructions::interrupts::disable();
loop {
    x86_64::instructions::hlt();
}
```
Upon successful boot and subsystem diagnostics, the kernel enters a halted state awaiting hardware interrupts or process scheduling.

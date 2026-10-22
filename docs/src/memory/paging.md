# Virtual Memory & Paging Documentation

- **Source File**: [`src/memory/paging.rs`](file:///d:/os/src/memory/paging.rs)
- **Subsystem**: Memory Management (Virtual Layer & Hardware MMU)

---

## 1. Conceptual Overview

Modern 64-bit CPUs do not let user programs or even high-level kernel functions read and write directly to physical RAM chips. Instead, they use **Virtual Memory**.

Every address a program interacts with is a **Virtual Address**. The hardware Memory Management Unit (MMU) consults a multi-level tree of translation tables (called **Page Tables**) to convert that virtual address into a **Physical Address** in RAM.

### Benefits of Paging
1. **Isolation**: Process A and Process B can both have code at virtual address `0x400000`, but they map to completely different physical RAM chips.
2. **Protection**: Pages can be marked read-only, non-executable (NX/XD bit to prevent malware buffer-overflow attacks), or kernel-only.
3. **Continuity**: Large arrays can appear contiguous in virtual memory even if their physical frames are scattered across RAM.

---

## 2. 4-Level Paging Architecture (x86_64)

In 64-bit x86 architecture, translating a 48-bit canonical virtual address uses a 4-level tree:

```
Virtual Address:
[ PML4 Index (9 bits) | PDPT Index (9 bits) | PD Index (9 bits) | PT Index (9 bits) | Offset (12 bits) ]
        |                      |                     |                  |                     |
        v                      v                     v                  v                     v
   [ CR3 Register ]
        |
        +---> PML4 Table (Level 4)
                  |
                  +---> PDPT Table (Level 3 - Page Directory Pointer Table)
                            |
                            +---> PD Table (Level 2 - Page Directory)
                                      |
                                      +---> PT Table (Level 1 - Page Table)
                                                |
                                                +---> Physical Frame (4 KiB) + Offset
```

- **CR3**: The CPU control register holding the physical address of the active Level 4 (PML4) table.

---

## 3. The `AddressSpace` Struct

```rust
pub struct AddressSpace {
    pml4_phys: PhysAddr,
}
```

Represents a virtual memory context anchored by its PML4 root table.

---

## 4. API Reference & Methods

### `AddressSpace::active() -> Self`
Reads the CPU's `CR3` register to attach to the currently running address space.

```rust
let active_space = AddressSpace::active();
```

### `AddressSpace::pml4_physical_address(&self) -> PhysAddr`
Returns the physical address of the Level 4 Page Table.

### `AddressSpace::table_at<'a>(&self, phys: PhysAddr) -> &'a PageTable`
*(Unsafe)* Dereferences a physical address as an active `PageTable` reference. In native 64-bit UEFI environments, memory is identity-mapped, allowing direct address dereferencing during bootstrap.

### `AddressSpace::translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr>`
Walks the 4-level page table hierarchy for a given virtual address:
1. Checks PML4 entry for `PRESENT`.
2. Checks PDPT entry for `PRESENT` and handles 1 GiB huge pages.
3. Checks PD entry for `PRESENT` and handles 2 MiB huge pages.
4. Checks PT entry for `PRESENT`.
5. Extracts the physical frame base and adds the 12-bit offset (`0..4095`).
- Returns `Some(PhysAddr)` if the address is currently mapped.
- Returns `None` if any table level in the walk is unmapped.

---

## 5. Practical Example

```rust
use crate::memory::paging::AddressSpace;
use x86_64::VirtAddr;

let space = AddressSpace::active();
let virt_ptr = VirtAddr::new(0x1000_5000);

// Translate to physical address:
match unsafe { space.translate_addr(virt_ptr) } {
    Some(phys) => println!("Virtual {:?} maps to Physical {:?}", virt_ptr, phys),
    None => println!("Virtual {:?} is not mapped in this address space", virt_ptr),
}
```

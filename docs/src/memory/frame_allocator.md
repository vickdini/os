# Physical Memory & Frame Allocator Documentation

- **Source File**: [`src/memory/frame_allocator.rs`](file:///d:/os/src/memory/frame_allocator.rs)
- **Subsystem**: Memory Management (Physical Layer)

---

## 1. Conceptual Overview

When a computer boots up, its physical Random Access Memory (RAM) is not one continuous empty space. Certain chunks are occupied by motherboard ROM, video graphics RAM, ACPI tables, and UEFI firmware code.

The **Physical Memory Frame Allocator** is responsible for:
1. Asking UEFI firmware which memory regions are safe and usable (`CONVENTIONAL` memory).
2. Dividing that usable RAM into standard **4096-byte (4 KiB) chunks**, called **Physical Frames**.
3. Handing out frames one-by-one to the kernel when it needs memory for page tables, stacks, or user processes.

### The Real-World Analogy
Think of RAM as a vast parking lot:
- Some spots are reserved for emergency vehicles or building infrastructure (firmware/hardware).
- The **Physical Memory Manager** is the parking attendant with a map of all standard open spots.
- Whenever a driver (a process or page table) arrives, the attendant assigns them the next available numbered spot (`0x1000`, `0x2000`, etc.).

---

## 2. Architecture & Data Structures

```
  +-------------------------------------------------------------------------+
  |                        PhysicalMemoryManager                            |
  +-------------------------------------------------------------------------+
  |  regions: [ Option<MemoryRegion>; 64 ]                                  |
  |  - Region 0: Start: 0x100000 (1 MiB), Count: 32768 pages (128 MiB)      |
  |  - Region 1: Start: 0x80000000, Count: 65536 pages (256 MiB)           |
  |  ...                                                                    |
  |  current_region_idx: 0                                                  |
  |  next_page_in_region: 15 (allocating frame by frame)                   |
  +-------------------------------------------------------------------------+
```

### Key Constants
- `PAGE_SIZE: u64 = 4096`: Standard x86_64 base page size (4 KiB).
- `MAX_REGIONS: usize = 64`: Maximum number of non-contiguous usable RAM blocks tracked.

---

## 3. Struct Details

### `MemoryRegion`
Represents a contiguous block of usable physical memory reported by firmware:
```rust
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub page_count: u64,
}
```

### `PhysicalMemoryManager`
The state tracker for physical memory:
```rust
pub struct PhysicalMemoryManager {
    regions: [Option<MemoryRegion>; MAX_REGIONS],
    region_count: usize,
    current_region_idx: usize,
    next_page_in_region: u64,
    total_usable_bytes: u64,
    allocated_frames_count: u64,
}
```

---

## 4. API Reference & Methods

### `PhysicalMemoryManager::add_region(&mut self, start: PhysAddr, page_count: u64)`
Registers a contiguous block of usable physical memory into the tracker.

### `PhysicalMemoryManager::allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>>`
Finds the next free 4 KiB frame from the current memory region and advances the bump pointer. If a region is exhausted, it moves to the next region.
- Returns `Some(PhysFrame)` containing the physical start address.
- Returns `None` if physical memory is exhausted (Out of Memory).

### `PhysicalMemoryManager::total_usable_memory(&self) -> u64`
Returns total detected usable physical RAM in bytes.

### `PhysicalMemoryManager::allocated_memory(&self) -> u64`
Returns total allocated physical RAM in bytes.

### `PhysicalMemoryManager::free_memory(&self) -> u64`
Returns currently unallocated physical RAM in bytes.

### `x86_64::structures::paging::FrameAllocator<Size4KiB>` Trait Implementation
```rust
unsafe impl FrameAllocator<Size4KiB> for PhysicalMemoryManager {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.allocate_frame()
    }
}
```
Enables `PhysicalMemoryManager` to serve directly as the frame supplier for `x86_64` page table manipulators and memory allocators.

---

## 5. UEFI Firmware Integration & Low Memory Protection

```rust
pub fn init_memory_from_uefi()
```
Queries `uefi::boot::memory_map(MemoryType::LOADER_DATA)`:
1. Iterates through all memory descriptors provided by the UEFI firmware.
2. Filters specifically for `uefi::boot::MemoryType::CONVENTIONAL`.
3. **Reserves the First 1 MiB (`0x100000`)**: Skips addresses below 1 MiB to protect legacy Real-Mode Interrupt Vector Tables (IVT), BIOS Data Areas (BDA), and to prevent `PhysAddr(0)` from being issued as a valid frame (protecting against null-pointer aliasing).
4. Registers each usable conventional memory block into the global `FRAME_ALLOCATOR` mutex.

---

## 6. Practical Example

```rust
use crate::memory::frame_allocator::{FRAME_ALLOCATOR, init_memory_from_uefi};

// 1. Read UEFI firmware memory map and discover usable RAM
init_memory_from_uefi();

// 2. Allocate a 4 KiB physical frame
let frame = {
    let mut alloc = FRAME_ALLOCATOR.lock();
    alloc.allocate_frame().expect("System has available RAM")
};

// 3. Inspect frame address
let physical_address = frame.start_address().as_u64();
```

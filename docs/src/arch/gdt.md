# Global Descriptor Table (GDT) & TSS Documentation

- **Source File**: [`src/arch/gdt.rs`](file:///d:/os/src/arch/gdt.rs)
- **Subsystem**: Architecture / CPU Privilege & Segmentation

---

## 1. Conceptual Overview

On x86_64 processors, the **Global Descriptor Table (GDT)** is a legacy hardware structure that remains vital for two critical operating system responsibilities:
1. **Privilege Levels (Rings)**: Defining segments for **Ring 0 (Kernel Mode)** and **Ring 3 (Userland Mode)**.
2. **Task State Segment (TSS)**: Defining emergency hardware stack pointers for CPU privilege transitions and catastrophic fault recovery (Double Faults).

### Privilege Rings
- **Ring 0**: Full CPU privilege. The kernel executes here and can run privileged instructions (`cli`, `sti`, `invlpg`, write to control registers `CR0..CR4`).
- **Ring 3**: Sandboxed user mode. Applications run here and are restricted by the MMU and capability system.

---

## 2. Architecture & Components

```
+-------------------------------------------------------------+
|               Global Descriptor Table (GDT)                 |
+-------------------------------------------------------------+
| 1. Kernel Code Segment (64-bit, Privilege Level 0)          |
| 2. Kernel Data Segment (Privilege Level 0)                  |
| 3. User Data Segment   (Privilege Level 3)                  |
| 4. User Code Segment   (64-bit, Privilege Level 3)          |
| 5. TSS Segment         (Task State Segment Descriptor)      |
+-------------------------------------------------------------+
                               |
                               v
               +-------------------------------+
               |   Task State Segment (TSS)    |
               +-------------------------------+
               | IST[0]: Double Fault Stack    |
               | RSP[0]: Privilege Stack       |
               +-------------------------------+
```

---

## 3. Data Structures & Stacks

### Stack Allocation
```rust
const STACK_SIZE: usize = 4096 * 5; // 20 KiB
static mut DOUBLE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
static mut PRIVILEGE_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
```
- **`DOUBLE_FAULT_STACK`**: When a double fault occurs (e.g. a page fault inside a kernel stack overflow), the CPU cannot use the current corrupted stack. Using the **Interrupt Stack Table (IST index 0)**, the CPU automatically switches to this clean stack.
- **`PRIVILEGE_STACK`**: When a Ring 3 user program triggers a system call or interrupt to enter Ring 0, the CPU loads this stack into `RSP` (`tss.privilege_stack_table[0]`).

### `Selectors`
Stores the segment selectors returned when the segments are appended to the GDT:
```rust
pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}
```

---

## 4. API Reference & Methods

### `arch::gdt::init()`
Initializes and prepares Hydra's segmentation and hardware privilege structures in memory:
1. **Prepares Task State Segment (TSS)**: Configures `interrupt_stack_table[0]` for the double-fault emergency stack and `privilege_stack_table[0]` for Ring 3 &rarr; Ring 0 privilege transitions.
2. **Builds the GDT**: Allocates and configures Kernel Code, Kernel Data, User Data, User Code, and TSS descriptors.
3. **UEFI Firmware Coexistence**: While native UEFI Boot Services are active, the firmware's IDT relies on firmware code selectors (e.g., `0x38` on VirtualBox EDK2). Hydra ensures GDT and TSS structures are initialized and accessible via `get_selectors()` without disrupting the firmware's active interrupt vectors.

```rust
arch::gdt::init();
```

### `arch::gdt::get_selectors() -> &'static Selectors`
Provides access to the active segment selectors (`kernel_code_selector`, `kernel_data_selector`, `user_data_selector`, `user_code_selector`, `tss_selector`) for subsequent userland process spawning, context switches, and interrupt returns.

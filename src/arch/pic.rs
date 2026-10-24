use x86_64::instructions::port::Port;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;
const PIC_EOI: u8 = 0x20;

pub const PIC1_OFFSET: u8 = 0x20; // Vectors 32..39
pub const PIC2_OFFSET: u8 = 0x28; // Vectors 40..47

#[inline(always)]
fn io_wait() {
    unsafe {
        Port::<u8>::new(0x80).write(0);
    }
}

/// Initializes and remaps the legacy 8259 dual-PIC architecture.
///
/// Remaps:
/// - Master PIC (IRQ 0..7)  -> IDT Vectors 32..39 (0x20..0x27)
/// - Slave PIC  (IRQ 8..15) -> IDT Vectors 40..47 (0x28..0x2F)
///
/// Configures initial interrupt masks:
/// - Unmasks IRQ 0 (PIT Timer), IRQ 1 (PS/2 Keyboard), and IRQ 2 (Cascade line to Slave).
/// - Masks all other peripheral IRQs to prevent unexpected spurious interrupts.
pub fn init() {
    unsafe {
        let mut p1_cmd: Port<u8> = Port::new(PIC1_COMMAND);
        let mut p1_data: Port<u8> = Port::new(PIC1_DATA);
        let mut p2_cmd: Port<u8> = Port::new(PIC2_COMMAND);
        let mut p2_data: Port<u8> = Port::new(PIC2_DATA);

        // ICW1: Start initialization sequence in cascade mode
        p1_cmd.write(ICW1_INIT);
        io_wait();
        p2_cmd.write(ICW1_INIT);
        io_wait();

        // ICW2: Vector offset programming
        p1_data.write(PIC1_OFFSET);
        io_wait();
        p2_data.write(PIC2_OFFSET);
        io_wait();

        // ICW3: Cascade configuration
        // Tell Master PIC that there is a slave PIC at IRQ 2 (bit 2 = 1 -> 0b0000_0100)
        p1_data.write(0x04);
        io_wait();
        // Tell Slave PIC its cascade identity (IRQ 2 -> 0b0000_0010)
        p2_data.write(0x02);
        io_wait();

        // ICW4: Additional environment configuration (8086/88 mode)
        p1_data.write(ICW4_8086);
        io_wait();
        p2_data.write(ICW4_8086);
        io_wait();

        // Unmask IRQ 0 (Timer), IRQ 1 (Keyboard), and IRQ 2 (Cascade to Slave PIC)
        // Master Mask: 0b1111_1000 = 0xF8 (bits 0, 1, 2 unmasked)
        // Slave Mask:  0b1111_1111 = 0xFF (all slave IRQs masked initially)
        p1_data.write(0xF8);
        io_wait();
        p2_data.write(0xFF);
        io_wait();
    }
}

/// Sends an End of Interrupt (EOI) signal to the PIC.
/// For IRQ 0..7, sends to Master PIC.
/// For IRQ 8..15, sends to both Slave and Master PIC.
pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            Port::<u8>::new(PIC2_COMMAND).write(PIC_EOI);
        }
        Port::<u8>::new(PIC1_COMMAND).write(PIC_EOI);
    }
}

/// Unmasks a specific IRQ line on the PIC.
pub fn unmask_irq(irq: u8) {
    unsafe {
        if irq < 8 {
            let mut port: Port<u8> = Port::new(PIC1_DATA);
            let val = port.read();
            port.write(val & !(1 << irq));
        } else {
            let mut port: Port<u8> = Port::new(PIC2_DATA);
            let val = port.read();
            port.write(val & !(1 << (irq - 8)));
        }
    }
}

/// Masks a specific IRQ line on the PIC.
pub fn mask_irq(irq: u8) {
    unsafe {
        if irq < 8 {
            let mut port: Port<u8> = Port::new(PIC1_DATA);
            let val = port.read();
            port.write(val | (1 << irq));
        } else {
            let mut port: Port<u8> = Port::new(PIC2_DATA);
            let val = port.read();
            port.write(val | (1 << (irq - 8)));
        }
    }
}

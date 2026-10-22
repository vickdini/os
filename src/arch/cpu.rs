use x86_64::registers::control::{Cr0, Cr0Flags, Cr2, Cr3, Cr4, Cr4Flags};
use x86_64::PhysAddr;

#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    pub cr0: Cr0Flags,
    pub cr2: u64,
    pub cr3_frame: PhysAddr,
    pub cr4: Cr4Flags,
}

pub fn read_state() -> CpuState {
    let (cr3_frame, _) = Cr3::read();
    CpuState {
        cr0: Cr0::read(),
        cr2: Cr2::read_raw(),
        cr3_frame: cr3_frame.start_address(),
        cr4: Cr4::read(),
    }
}

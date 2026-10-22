pub mod table;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rights(pub u32);

impl Rights {
    pub const READ: Rights = Rights(1 << 0);
    pub const WRITE: Rights = Rights(1 << 1);
    pub const EXECUTE: Rights = Rights(1 << 2);
    pub const GRANT: Rights = Rights(1 << 3);
    pub const CALL: Rights = Rights(1 << 4);
    pub const ALL: Rights = Rights(0x1F);

    pub fn contains(&self, other: Rights) -> bool {
        (self.0 & other.0) == other.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapType {
    UntypedMemory { size_bytes: usize },
    MemoryFrame { physical_addr: u64 },
    AddressSpace { root_cr3: u64 },
    Endpoint { endpoint_id: u32 },
    Thread { thread_id: u32 },
    HardwareDevice { device_id: u32 },
}

#[derive(Debug, Clone, Copy)]
pub struct Capability {
    pub cap_type: CapType,
    pub rights: Rights,
}

impl Capability {
    pub fn new(cap_type: CapType, rights: Rights) -> Self {
        Self { cap_type, rights }
    }
}

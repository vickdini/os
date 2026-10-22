use crate::capability::Capability;

pub const MAX_INLINE_WORDS: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct Message {
    /// Operation code / protocol label
    pub label: u32,
    /// Fast inline register arguments (e.g. syscall / RPC parameters)
    pub args: [u64; MAX_INLINE_WORDS],
    /// Optional capability delegated or transferred with this message
    pub capability: Option<Capability>,
}

impl Message {
    pub const fn new(label: u32) -> Self {
        Self {
            label,
            args: [0; MAX_INLINE_WORDS],
            capability: None,
        }
    }

    pub fn with_args(label: u32, args: [u64; MAX_INLINE_WORDS]) -> Self {
        Self {
            label,
            args,
            capability: None,
        }
    }

    pub fn with_cap(mut self, cap: Capability) -> Self {
        self.capability = Some(cap);
        self
    }
}

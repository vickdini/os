use super::{Capability, Rights};

pub const MAX_CAPABILITIES: usize = 256;

pub struct CapabilityTable {
    slots: [Option<Capability>; MAX_CAPABILITIES],
    next_free_slot: usize,
}

impl CapabilityTable {
    pub const fn new() -> Self {
        Self {
            slots: [None; MAX_CAPABILITIES],
            next_free_slot: 0,
        }
    }

    pub fn insert(&mut self, cap: Capability) -> Result<usize, &'static str> {
        for idx in 0..MAX_CAPABILITIES {
            let slot = (self.next_free_slot + idx) % MAX_CAPABILITIES;
            if self.slots[slot].is_none() {
                self.slots[slot] = Some(cap);
                self.next_free_slot = (slot + 1) % MAX_CAPABILITIES;
                return Ok(slot);
            }
        }
        Err("Capability table full")
    }

    pub fn get(&self, slot: usize) -> Option<&Capability> {
        if slot < MAX_CAPABILITIES {
            self.slots[slot].as_ref()
        } else {
            None
        }
    }

    pub fn revoke(&mut self, slot: usize) -> Option<Capability> {
        if slot < MAX_CAPABILITIES {
            self.slots[slot].take()
        } else {
            None
        }
    }

    pub fn check_rights(&self, slot: usize, required_rights: Rights) -> bool {
        if let Some(cap) = self.get(slot) {
            cap.rights.contains(required_rights)
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        for slot in self.slots.iter() {
            if slot.is_some() {
                count += 1;
            }
        }
        count
    }
}

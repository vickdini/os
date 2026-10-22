use uefi::proto::console::text::Color;

use crate::arch;
use crate::capability::table::CapabilityTable;
use crate::capability::{CapType, Capability, Rights};
use crate::ipc::Endpoint;

pub struct ShellState {
    pub cap_table: CapabilityTable,
    pub endpoint: Endpoint,
    pub prompt_color: Color,
}

impl ShellState {
    pub fn new() -> Self {
        let mut cap_table = CapabilityTable::new();

        // Initialize root capabilities
        let cpu = arch::cpu::read_state();
        let _ = cap_table.insert(Capability::new(
            CapType::AddressSpace {
                root_cr3: cpu.cr3_frame.as_u64(),
            },
            Rights::ALL,
        ));
        let _ = cap_table.insert(Capability::new(
            CapType::Endpoint { endpoint_id: 1 },
            Rights(Rights::READ.0 | Rights::WRITE.0 | Rights::CALL.0),
        ));

        Self {
            cap_table,
            endpoint: Endpoint::new(1),
            prompt_color: Color::LightCyan,
        }
    }
}

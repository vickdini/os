use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};

pub struct AddressSpace {
    pml4_phys: PhysAddr,
}

impl AddressSpace {
    /// Obtains the currently active address space from the CPU CR3 register.
    pub fn active() -> Self {
        let (cr3_frame, _) = Cr3::read();
        Self {
            pml4_phys: cr3_frame.start_address(),
        }
    }

    pub fn pml4_physical_address(&self) -> PhysAddr {
        self.pml4_phys
    }

    /// In UEFI x86_64 identity-mapped environments or direct-mapped kernels,
    /// read a page table at a given physical address.
    pub unsafe fn table_at<'a>(&self, phys: PhysAddr) -> &'a PageTable {
        // In native 64-bit UEFI, the page table memory is identity mapped.
        let ptr = phys.as_u64() as *const PageTable;
        &*ptr
    }

    /// Walk the 4-level page table to translate a virtual address to physical address if mapped.
    pub unsafe fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        let p4 = self.table_at(self.pml4_phys);
        let p4_entry = &p4[addr.p4_index()];
        if !p4_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let p3 = self.table_at(p4_entry.addr());
        let p3_entry = &p3[addr.p3_index()];
        if !p3_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        if p3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            // 1 GiB huge page
            let offset = addr.as_u64() & 0x3fff_ffff;
            return Some(p3_entry.addr() + offset);
        }

        let p2 = self.table_at(p3_entry.addr());
        let p2_entry = &p2[addr.p2_index()];
        if !p2_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        if p2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            // 2 MiB huge page
            let offset = addr.as_u64() & 0x1f_ffff;
            return Some(p2_entry.addr() + offset);
        }

        let p1 = self.table_at(p2_entry.addr());
        let p1_entry = &p1[addr.p1_index()];
        if !p1_entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let offset = addr.as_u64() & 0xfff;
        Some(p1_entry.addr() + offset)
    }
}

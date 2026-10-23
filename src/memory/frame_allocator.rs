use spin::Mutex;
use uefi::mem::memory_map::MemoryMap;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub const PAGE_SIZE: u64 = 4096;
const MAX_REGIONS: usize = 64;

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub page_count: u64,
}

pub struct PhysicalMemoryManager {
    regions: [Option<MemoryRegion>; MAX_REGIONS],
    region_count: usize,
    current_region_idx: usize,
    next_page_in_region: u64,
    total_usable_bytes: u64,
    allocated_frames_count: u64,
}

impl PhysicalMemoryManager {
    pub const fn new() -> Self {
        Self {
            regions: [None; MAX_REGIONS],
            region_count: 0,
            current_region_idx: 0,
            next_page_in_region: 0,
            total_usable_bytes: 0,
            allocated_frames_count: 0,
        }
    }

    pub fn add_region(&mut self, start: PhysAddr, page_count: u64) {
        if self.region_count < MAX_REGIONS && page_count > 0 {
            self.regions[self.region_count] = Some(MemoryRegion { start, page_count });
            self.region_count += 1;
            self.total_usable_bytes += page_count * PAGE_SIZE;
        }
    }

    pub fn total_usable_memory(&self) -> u64 {
        self.total_usable_bytes
    }

    pub fn allocated_memory(&self) -> u64 {
        self.allocated_frames_count * PAGE_SIZE
    }

    pub fn free_memory(&self) -> u64 {
        self.total_usable_bytes.saturating_sub(self.allocated_memory())
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        while self.current_region_idx < self.region_count {
            if let Some(region) = self.regions[self.current_region_idx] {
                if self.next_page_in_region < region.page_count {
                    let frame_addr = region.start + (self.next_page_in_region * PAGE_SIZE);
                    self.next_page_in_region += 1;
                    self.allocated_frames_count += 1;
                    return Some(PhysFrame::containing_address(frame_addr));
                }
            }
            self.current_region_idx += 1;
            self.next_page_in_region = 0;
        }
        None
    }

    /// Allocates a contiguous block of physical frames (e.g. for the kernel heap or DMA buffers).
    pub fn allocate_contiguous_frames(&mut self, count: usize) -> Option<PhysFrame<Size4KiB>> {
        let count_u64 = count as u64;
        while self.current_region_idx < self.region_count {
            if let Some(region) = self.regions[self.current_region_idx] {
                let remaining = region.page_count.saturating_sub(self.next_page_in_region);
                if remaining >= count_u64 {
                    let frame_addr = region.start + (self.next_page_in_region * PAGE_SIZE);
                    self.next_page_in_region += count_u64;
                    self.allocated_frames_count += count_u64;
                    return Some(PhysFrame::containing_address(frame_addr));
                }
            }
            self.current_region_idx += 1;
            self.next_page_in_region = 0;
        }
        None
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalMemoryManager {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.allocate_frame()
    }
}

pub static FRAME_ALLOCATOR: Mutex<PhysicalMemoryManager> = Mutex::new(PhysicalMemoryManager::new());

pub fn init_memory_from_uefi() {
    let mut allocator = FRAME_ALLOCATOR.lock();

    if let Ok(memory_map) = uefi::boot::memory_map(uefi::boot::MemoryType::LOADER_DATA) {
        for desc in memory_map.entries() {
            // Conventional memory is free memory available for operating system allocation
            if desc.ty == uefi::boot::MemoryType::CONVENTIONAL {
                // Reserve the first 1 MiB (0x100000) to protect IVT/BDA and avoid null pointer frame (0x0)
                if desc.phys_start < 0x100000 {
                    let end = desc.phys_start + desc.page_count * PAGE_SIZE;
                    if end > 0x100000 {
                        let skipped_pages = (0x100000 - desc.phys_start) / PAGE_SIZE;
                        let remaining_pages = desc.page_count.saturating_sub(skipped_pages);
                        if remaining_pages > 0 {
                            allocator.add_region(PhysAddr::new(0x100000), remaining_pages);
                        }
                    }
                } else {
                    allocator.add_region(PhysAddr::new(desc.phys_start), desc.page_count);
                }
            }
        }
    }
}

use uefi::boot::{allocate_pages, AllocateType};
use uefi::mem::memory_map::MemoryType;

use super::types::{BlockPointer, Fletcher4, Uberblock, ZFS_BLOCK_SIZE};

/// Total storage capacity of the default Hydra ZFS storage pool (8 MiB).
pub const POOL_CAPACITY_BYTES: usize = 8 * 1024 * 1024;
pub const POOL_TOTAL_BLOCKS: usize = POOL_CAPACITY_BYTES / ZFS_BLOCK_SIZE; // 2048 blocks

/// Virtual Device (vdev) storage backing the ZFS pool.
/// Dynamically allocated from UEFI loader data pages at boot time.
pub struct VdevStorage {
    base_ptr: *mut u8,
    _capacity_bytes: usize,
}

unsafe impl Send for VdevStorage {}
unsafe impl Sync for VdevStorage {}

impl VdevStorage {
    pub fn new() -> Self {
        let pages = (POOL_CAPACITY_BYTES + 4095) / 4096;
        let ptr = match allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages) {
            Ok(p) => p.as_ptr(),
            Err(_) => core::ptr::null_mut(),
        };

        if !ptr.is_null() {
            unsafe {
                core::ptr::write_bytes(ptr, 0, POOL_CAPACITY_BYTES);
            }
        }

        Self {
            base_ptr: ptr,
            _capacity_bytes: POOL_CAPACITY_BYTES,
        }
    }

    pub fn read_block(&self, block_index: usize, out_buf: &mut [u8]) -> Result<(), &'static str> {
        if self.base_ptr.is_null() {
            return Err("Vdev storage uninitialized");
        }
        if block_index >= POOL_TOTAL_BLOCKS {
            return Err("Block index out of bounds");
        }
        let offset = block_index * ZFS_BLOCK_SIZE;
        let copy_len = if out_buf.len() < ZFS_BLOCK_SIZE { out_buf.len() } else { ZFS_BLOCK_SIZE };
        unsafe {
            let src = self.base_ptr.add(offset);
            core::ptr::copy_nonoverlapping(src, out_buf.as_mut_ptr(), copy_len);
        }
        Ok(())
    }

    pub fn write_block(&mut self, block_index: usize, in_buf: &[u8]) -> Result<(), &'static str> {
        if self.base_ptr.is_null() {
            return Err("Vdev storage uninitialized");
        }
        if block_index >= POOL_TOTAL_BLOCKS {
            return Err("Block index out of bounds");
        }
        let offset = block_index * ZFS_BLOCK_SIZE;
        let copy_len = if in_buf.len() < ZFS_BLOCK_SIZE { in_buf.len() } else { ZFS_BLOCK_SIZE };
        unsafe {
            let dst = self.base_ptr.add(offset);
            core::ptr::copy_nonoverlapping(in_buf.as_ptr(), dst, copy_len);
            if copy_len < ZFS_BLOCK_SIZE {
                core::ptr::write_bytes(dst.add(copy_len), 0, ZFS_BLOCK_SIZE - copy_len);
            }
        }
        Ok(())
    }
}

/// Storage Pool Allocator (SPA)
/// Tracks block allocations, transaction groups (TXG), and active Uberblock.
pub struct Spa {
    pub vdev: VdevStorage,
    pub block_bitmap: [u64; POOL_TOTAL_BLOCKS / 64],
    pub active_uberblock: Uberblock,
    pub pool_name: [u8; 16],
    pub allocated_blocks: usize,
}

impl Spa {
    pub fn new(name: &str) -> Self {
        let mut pool_name = [0u8; 16];
        let bytes = name.as_bytes();
        let len = if bytes.len() > 15 { 15 } else { bytes.len() };
        for i in 0..len {
            pool_name[i] = bytes[i];
        }

        let initial_ub = Uberblock::new(1, BlockPointer::empty(), pool_name);

        let mut spa = Self {
            vdev: VdevStorage::new(),
            block_bitmap: [0u64; POOL_TOTAL_BLOCKS / 64],
            active_uberblock: initial_ub,
            pool_name,
            allocated_blocks: 0,
        };

        // Reserve block 0 for Vdev Label and Uberblock array
        spa.mark_block(0, true);
        spa.allocated_blocks = 1;

        spa
    }

    fn mark_block(&mut self, block: usize, used: bool) {
        let word_idx = block / 64;
        let bit_idx = block % 64;
        if used {
            self.block_bitmap[word_idx] |= 1u64 << bit_idx;
        } else {
            self.block_bitmap[word_idx] &= !(1u64 << bit_idx);
        }
    }

    fn is_block_used(&self, block: usize) -> bool {
        let word_idx = block / 64;
        let bit_idx = block % 64;
        (self.block_bitmap[word_idx] & (1u64 << bit_idx)) != 0
    }

    /// Allocates a new block using Copy-On-Write (CoW) allocation,
    /// writes the data into the allocated block, and computes the Fletcher-4 checksum.
    pub fn cow_allocate_block(&mut self, data: &[u8]) -> Result<BlockPointer, &'static str> {
        let mut target_block = None;

        // Linear scan for first free block
        for b in 1..POOL_TOTAL_BLOCKS {
            if !self.is_block_used(b) {
                target_block = Some(b);
                break;
            }
        }

        let block_idx = match target_block {
            Some(idx) => idx,
            None => return Err("Storage pool is out of space (metaslabs full)"),
        };

        // Mark block as allocated
        self.mark_block(block_idx, true);
        self.allocated_blocks += 1;

        // Write block payload
        self.vdev.write_block(block_idx, data)?;

        // Compute Fletcher-4 checksum for end-to-end data integrity
        let checksum = Fletcher4::compute(data);

        let bp = BlockPointer {
            vdev_id: 0,
            offset: (block_idx * ZFS_BLOCK_SIZE) as u64,
            logical_size: data.len() as u32,
            physical_size: ZFS_BLOCK_SIZE as u32,
            birth_txg: self.active_uberblock.txg,
            checksum,
            is_allocated: true,
        };

        Ok(bp)
    }

    /// Reads data from a BlockPointer and verifies its Fletcher-4 checksum.
    pub fn read_and_verify_block(&self, bp: &BlockPointer, out: &mut [u8]) -> Result<usize, &'static str> {
        if !bp.is_allocated {
            return Err("Attempted to read unallocated block pointer");
        }

        let block_idx = (bp.offset as usize) / ZFS_BLOCK_SIZE;
        self.vdev.read_block(block_idx, out)?;

        // Verify checksum on valid slice
        let valid_len = if (bp.logical_size as usize) < out.len() {
            bp.logical_size as usize
        } else {
            out.len()
        };

        let calculated_checksum = Fletcher4::compute(&out[..valid_len]);
        if calculated_checksum != bp.checksum {
            return Err("ZFS Checksum verification failed (silent data corruption detected)");
        }

        Ok(valid_len)
    }

    /// Advances transaction group (TXG) and commits the new root Uberblock.
    pub fn commit_txg(&mut self, root_bp: BlockPointer) -> u64 {
        let new_txg = self.active_uberblock.txg + 1;
        self.active_uberblock = Uberblock::new(new_txg, root_bp, self.pool_name);
        new_txg
    }

    pub fn pool_name_str(&self) -> &str {
        let mut len = 0;
        while len < self.pool_name.len() && self.pool_name[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&self.pool_name[..len]).unwrap_or("hydrapool")
    }

    pub fn total_capacity_bytes(&self) -> usize {
        POOL_CAPACITY_BYTES
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated_blocks * ZFS_BLOCK_SIZE
    }

    pub fn free_bytes(&self) -> usize {
        POOL_CAPACITY_BYTES.saturating_sub(self.allocated_bytes())
    }
}

use super::spa::Spa;
use super::types::{DnodePhys, DnodeType};
use super::zap::ZapDirectory;

pub const MAX_OBJECTS: usize = 64;
pub const MAX_DATASETS: usize = 8;
pub const MAX_SNAPSHOTS: usize = 8;

/// A ZFS Dataset representing an isolated filesystem or volume (e.g. `hydrapool/root`).
#[derive(Debug, Clone, Copy)]
pub struct Dataset {
    pub name: [u8; 32],
    pub mountpoint: [u8; 32],
    pub root_dir_obj: u64,
    pub used_bytes: u64,
    pub is_valid: bool,
}

impl Dataset {
    pub const fn empty() -> Self {
        Self {
            name: [0u8; 32],
            mountpoint: [0u8; 32],
            root_dir_obj: 0,
            used_bytes: 0,
            is_valid: false,
        }
    }

    pub fn set(&mut self, name: &str, mountpoint: &str, root_dir_obj: u64) {
        let name_bytes = name.as_bytes();
        let name_len = if name_bytes.len() > 31 { 31 } else { name_bytes.len() };
        for i in 0..name_len {
            self.name[i] = name_bytes[i];
        }
        self.name[name_len] = 0;

        let mp_bytes = mountpoint.as_bytes();
        let mp_len = if mp_bytes.len() > 31 { 31 } else { mp_bytes.len() };
        for i in 0..mp_len {
            self.mountpoint[i] = mp_bytes[i];
        }
        self.mountpoint[mp_len] = 0;

        self.root_dir_obj = root_dir_obj;
        self.is_valid = true;
    }

    pub fn name_str(&self) -> &str {
        let mut len = 0;
        while len < self.name.len() && self.name[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&self.name[..len]).unwrap_or("")
    }

    pub fn mountpoint_str(&self) -> &str {
        let mut len = 0;
        while len < self.mountpoint.len() && self.mountpoint[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&self.mountpoint[..len]).unwrap_or("")
    }
}

/// A point-in-time Copy-on-Write snapshot of a dataset.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    pub dataset_name: [u8; 32],
    pub snapshot_name: [u8; 32],
    pub txg: u64,
    pub root_obj: u64,
    pub is_valid: bool,
}

impl Snapshot {
    pub const fn empty() -> Self {
        Self {
            dataset_name: [0u8; 32],
            snapshot_name: [0u8; 32],
            txg: 0,
            root_obj: 0,
            is_valid: false,
        }
    }

    pub fn full_name<'a>(&self, buf: &'a mut [u8; 64]) -> &'a str {
        let mut idx = 0;
        for &b in self.dataset_name.iter() {
            if b == 0 { break; }
            buf[idx] = b;
            idx += 1;
        }
        buf[idx] = b'@';
        idx += 1;
        for &b in self.snapshot_name.iter() {
            if b == 0 { break; }
            buf[idx] = b;
            idx += 1;
        }
        core::str::from_utf8(&buf[..idx]).unwrap_or("")
    }
}

/// Data Management Unit (DMU)
/// Manages the Object Set (MOS and datasets), dnodes, and ZAP directories.
pub struct Dmu {
    pub dnodes: [DnodePhys; MAX_OBJECTS],
    pub zap_dirs: [ZapDirectory; MAX_OBJECTS],
    pub datasets: [Dataset; MAX_DATASETS],
    pub snapshots: [Snapshot; MAX_SNAPSHOTS],
    pub next_obj_id: u64,
}

impl Dmu {
    pub fn new() -> Self {
        let mut dmu = Self {
            dnodes: [DnodePhys::empty(); MAX_OBJECTS],
            zap_dirs: [ZapDirectory::new(); MAX_OBJECTS],
            datasets: [Dataset::empty(); MAX_DATASETS],
            snapshots: [Snapshot::empty(); MAX_SNAPSHOTS],
            next_obj_id: 1, // 0 is reserved for MOS root
        };

        // Initialize Meta-Object Set (MOS) root at index 0
        dmu.dnodes[0].obj_type = DnodeType::MasterNode;
        dmu.dnodes[0].set_name("$MOS");

        dmu
    }

    /// Allocates a new dnode object ID
    pub fn allocate_object(&mut self, obj_type: DnodeType, name: &str) -> Result<u64, &'static str> {
        if self.next_obj_id as usize >= MAX_OBJECTS {
            return Err("Object set full (dnodes exhausted)");
        }
        let id = self.next_obj_id;
        self.next_obj_id += 1;

        let dnode = &mut self.dnodes[id as usize];
        dnode.obj_type = obj_type;
        dnode.set_name(name);

        Ok(id)
    }

    /// Creates a new dataset (e.g. `hydrapool/root`)
    pub fn create_dataset(&mut self, name: &str, mountpoint: &str) -> Result<usize, &'static str> {
        let mut free_slot = None;
        for (i, ds) in self.datasets.iter().enumerate() {
            if !ds.is_valid {
                free_slot = Some(i);
                break;
            }
        }

        let slot = free_slot.ok_or("Maximum datasets limit reached")?;
        let root_dir = self.allocate_object(DnodeType::Directory, name)?;
        self.datasets[slot].set(name, mountpoint, root_dir);
        Ok(slot)
    }

    /// Writes data to a file object using Copy-on-Write (CoW).
    pub fn cow_write_file(
        &mut self,
        obj_id: u64,
        data: &[u8],
        spa: &mut Spa,
    ) -> Result<u64, &'static str> {
        let idx = obj_id as usize;
        if idx >= MAX_OBJECTS || self.dnodes[idx].obj_type == DnodeType::None {
            return Err("Invalid object ID");
        }

        // Allocate block via SPA with Fletcher-4 checksum
        let bp = spa.cow_allocate_block(data)?;

        let dnode = &mut self.dnodes[idx];
        dnode.direct_blocks[0] = bp;
        dnode.file_size = data.len() as u64;
        dnode.birth_txg = bp.birth_txg;

        // Commit transaction group with updated root block pointer
        let new_txg = spa.commit_txg(bp);
        Ok(new_txg)
    }

    /// Reads data from a file object and verifies its integrity.
    pub fn read_file(
        &self,
        obj_id: u64,
        out_buf: &mut [u8],
        spa: &Spa,
    ) -> Result<usize, &'static str> {
        let idx = obj_id as usize;
        if idx >= MAX_OBJECTS || self.dnodes[idx].obj_type != DnodeType::PlainFile {
            return Err("Target object is not a readable file");
        }

        let dnode = &self.dnodes[idx];
        let bp = &dnode.direct_blocks[0];

        if !bp.is_allocated {
            return Ok(0); // Empty file
        }

        spa.read_and_verify_block(bp, out_buf)
    }

    /// Adds a file entry into a parent directory's ZAP structure.
    pub fn add_directory_entry(
        &mut self,
        parent_dir_obj: u64,
        name: &str,
        child_obj: u64,
        obj_type: DnodeType,
    ) -> Result<(), &'static str> {
        let idx = parent_dir_obj as usize;
        if idx >= MAX_OBJECTS {
            return Err("Parent directory ID out of bounds");
        }
        self.zap_dirs[idx].insert(name, child_obj, obj_type)
    }

    /// Creates a point-in-time CoW snapshot
    pub fn create_snapshot(&mut self, dataset_name: &str, snapshot_name: &str, current_txg: u64) -> Result<(), &'static str> {
        for snap in self.snapshots.iter_mut() {
            if !snap.is_valid {
                let d_bytes = dataset_name.as_bytes();
                let d_len = if d_bytes.len() > 31 { 31 } else { d_bytes.len() };
                for i in 0..d_len { snap.dataset_name[i] = d_bytes[i]; }
                snap.dataset_name[d_len] = 0;

                let s_bytes = snapshot_name.as_bytes();
                let s_len = if s_bytes.len() > 31 { 31 } else { s_bytes.len() };
                for i in 0..s_len { snap.snapshot_name[i] = s_bytes[i]; }
                snap.snapshot_name[s_len] = 0;

                snap.txg = current_txg;
                snap.is_valid = true;
                return Ok(());
            }
        }
        Err("Maximum snapshots limit reached")
    }
}

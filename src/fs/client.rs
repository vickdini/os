use super::zfs::server::{
    simple_str_hash, ZFS_IPC_LOOKUP, ZFS_IPC_PING, ZFS_IPC_POOL_STATUS,
    ZFS_IPC_READ, ZFS_IPC_SNAPSHOT, ZFS_IPC_WRITE, ZFS_SERVER, ZFS_STATUS_OK,
};
use super::zfs::types::DnodeType;
use crate::capability::Capability;
use crate::ipc::{Endpoint, Message};

/// Client interface for interacting with the ZFS File Server via Microkernel IPC
pub struct ZfsClient;

impl ZfsClient {
    /// Pings the ZFS File Server over IPC and returns current active transaction group (TXG).
    pub fn ping() -> Result<u64, &'static str> {
        let req = Message::new(ZFS_IPC_PING);
        let mut reply_ep = Endpoint::new(999);

        let mut server_lock = ZFS_SERVER.lock();
        if let Some(server) = server_lock.as_mut() {
            server.dispatch_request(req, &mut reply_ep);
        } else {
            return Err("ZFS File Server is not initialized");
        }

        if let Some(resp) = reply_ep.receive() {
            if resp.args[0] == ZFS_STATUS_OK {
                return Ok(resp.args[1]);
            }
        }
        Err("Failed to ping ZFS server")
    }

    /// Queries the ZFS pool status (total MiB, allocated KiB, free KiB, TXG)
    pub fn pool_status() -> Result<(u64, u64, u64, u64), &'static str> {
        let req = Message::new(ZFS_IPC_POOL_STATUS);
        let mut reply_ep = Endpoint::new(999);

        let mut server_lock = ZFS_SERVER.lock();
        if let Some(server) = server_lock.as_mut() {
            server.dispatch_request(req, &mut reply_ep);
        } else {
            return Err("ZFS File Server is not initialized");
        }

        if let Some(resp) = reply_ep.receive() {
            if resp.args[0] == ZFS_STATUS_OK {
                let total_mb = resp.args[1];
                let alloc_kb = resp.args[2];
                let free_kb = resp.args[3] >> 32;
                let txg = resp.args[3] & 0xFFFF_FFFF;
                return Ok((total_mb, alloc_kb, free_kb, txg));
            }
        }
        Err("Failed to query pool status from ZFS server")
    }

    /// Looks up an object ID by path (e.g. "/bin/hello.efi" or "hello.efi") in the ZFS pool.
    pub fn lookup(path: &str) -> Result<(u64, DnodeType), &'static str> {
        let clean = path.trim_start_matches('/').trim_start_matches('\\');
        let filename = clean.strip_prefix("bin/").unwrap_or_else(|| {
            clean.strip_prefix("bin\\").unwrap_or(clean)
        });

        let mut server_lock = ZFS_SERVER.lock();
        let server = match server_lock.as_mut() {
            Some(s) => s,
            None => return Err("ZFS File Server not running"),
        };

        let dir_obj = server.bin_dir_obj;
        let hash = simple_str_hash(filename);

        let req = Message::with_args(ZFS_IPC_LOOKUP, [dir_obj, hash, 0, 0]);
        let mut reply_ep = Endpoint::new(999);
        server.dispatch_request(req, &mut reply_ep);

        if let Some(resp) = reply_ep.receive() {
            if resp.args[0] == ZFS_STATUS_OK {
                let obj_id = resp.args[1];
                let obj_type = match resp.args[2] {
                    1 => DnodeType::PlainFile,
                    2 => DnodeType::Directory,
                    3 => DnodeType::Dataset,
                    _ => DnodeType::None,
                };
                return Ok((obj_id, obj_type));
            }
        }

        Err("File not found in ZFS File Server")
    }

    /// Reads data from a file object via Capability-mediated ZFS IPC.
    pub fn read_file(obj_id: u64, out_buf: &mut [u8], cap: Option<Capability>) -> Result<usize, &'static str> {
        let req = Message::with_args(
            ZFS_IPC_READ,
            [obj_id, out_buf.as_mut_ptr() as u64, out_buf.len() as u64, 0],
        );
        let req = if let Some(c) = cap { req.with_cap(c) } else { req };

        let mut reply_ep = Endpoint::new(999);
        let mut server_lock = ZFS_SERVER.lock();
        let server = match server_lock.as_mut() {
            Some(s) => s,
            None => return Err("ZFS File Server not running"),
        };

        server.dispatch_request(req, &mut reply_ep);

        if let Some(resp) = reply_ep.receive() {
            if resp.args[0] == ZFS_STATUS_OK {
                return Ok(resp.args[1] as usize);
            } else if resp.args[0] == 2 {
                return Err("Capability rights violation (READ right required)");
            } else if resp.args[0] == 4 {
                return Err("ZFS Fletcher-4 checksum failure: file corrupted");
            }
        }

        Err("Read error from ZFS server")
    }

    /// Writes data to a file object via Capability-mediated ZFS CoW IPC.
    pub fn write_file(obj_id: u64, data: &[u8], cap: Option<Capability>) -> Result<u64, &'static str> {
        let req = Message::with_args(
            ZFS_IPC_WRITE,
            [obj_id, data.as_ptr() as u64, data.len() as u64, 0],
        );
        let req = if let Some(c) = cap { req.with_cap(c) } else { req };

        let mut reply_ep = Endpoint::new(999);
        let mut server_lock = ZFS_SERVER.lock();
        let server = match server_lock.as_mut() {
            Some(s) => s,
            None => return Err("ZFS File Server not running"),
        };

        server.dispatch_request(req, &mut reply_ep);

        if let Some(resp) = reply_ep.receive() {
            if resp.args[0] == ZFS_STATUS_OK {
                return Ok(resp.args[1]); // new TXG
            } else if resp.args[0] == 2 {
                return Err("Capability rights violation (WRITE right required)");
            }
        }

        Err("Write error from ZFS server")
    }

    /// Creates a snapshot of a dataset via ZFS IPC
    pub fn snapshot(dataset_id: usize) -> Result<u64, &'static str> {
        let req = Message::with_args(ZFS_IPC_SNAPSHOT, [dataset_id as u64, 0, 0, 0]);
        let mut reply_ep = Endpoint::new(999);

        let mut server_lock = ZFS_SERVER.lock();
        let server = match server_lock.as_mut() {
            Some(s) => s,
            None => return Err("ZFS File Server not running"),
        };

        server.dispatch_request(req, &mut reply_ep);

        if let Some(resp) = reply_ep.receive() {
            if resp.args[0] == ZFS_STATUS_OK {
                return Ok(resp.args[1]);
            }
        }
        Err("Failed to create snapshot")
    }

    /// Queries directory contents from the ZFS File Server.
    pub fn list_dir(
        path: &str,
    ) -> Result<([crate::fs::zfs::zap::ZapEntry; crate::fs::zfs::zap::MAX_ZAP_ENTRIES], usize, &'static str), &'static str> {
        let clean = path.trim_start_matches('/').trim_start_matches('\\');
        let server_lock = ZFS_SERVER.lock();
        let server = match server_lock.as_ref() {
            Some(s) => s,
            None => return Err("ZFS File Server not running"),
        };

        let (target_obj, dataset_name) = if clean.is_empty() {
            (server.bin_dir_obj, "hydrapool/bin")
        } else if clean == "root" || clean == "/" {
            (server.root_dir_obj, "hydrapool/root")
        } else if clean == "bin" {
            (server.bin_dir_obj, "hydrapool/bin")
        } else {
            (server.bin_dir_obj, "hydrapool/bin")
        };

        let idx = target_obj as usize;
        if idx < crate::fs::zfs::dmu::MAX_OBJECTS {
            let zap = &server.dmu.zap_dirs[idx];
            return Ok((zap.entries, zap.count, dataset_name));
        }

        Err("Directory not found in ZFS")
    }
}

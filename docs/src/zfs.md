# ZFS Microkernel File Server Documentation

- **Source Directory**: [`src/fs/zfs/`](file:///d:/os/src/fs/zfs/) & [`src/fs/client.rs`](file:///d:/os/src/fs/client.rs)
- **Subsystem**: Default Filesystem Architecture & Standalone File Server
- **Endpoint**: `0x20` (`ENDPOINT_ZFS_SERVER`)

---

## 1. Architectural Motivation

In traditional monolithic operating systems, the filesystem runs in kernel space (Ring 0) with direct block device access. This violates principle-of-least-privilege and microkernel design tenants. A single bug or corruption in the filesystem driver can compromise the entire operating system kernel.

Hydra implements **ZFS (Zettabyte File System)** as the default filesystem, structured strictly as an **autonomous Microkernel File Server (`ZfsServer`)**.

```
+--------------------------------------------------------------+
|                     Client Applications                       |
|           (Hydra Interactive Shell, Process Loader)          |
+--------------------------------------------------------------+
                               |
                   Capability Token Validation
                   Microkernel IPC Rendezvous (Endpoint 0x20)
                               v
+--------------------------------------------------------------+
|               ZFS MICROKERNEL FILE SERVER                    |
|                                                              |
|   +------------------------------------------------------+   |
|   | ZPL (ZFS POSIX Layer) / IPC Server Dispatcher        |   |
|   +------------------------------------------------------+   |
|                              |                               |
|   +------------------------------------------------------+   |
|   | ZAP (ZFS Attribute Processor): Micro-ZAP Directories |   |
|   +------------------------------------------------------+   |
|                              |                               |
|   +------------------------------------------------------+   |
|   | DMU (Data Management Unit): Datasets & Dnodes        |   |
|   +------------------------------------------------------+   |
|                              |                               |
|   +------------------------------------------------------+   |
|   | SPA (Storage Pool Allocator): Metaslabs & Vdevs      |   |
|   | - Copy-on-Write (CoW) Merkle Tree Engine             |   |
|   | - Fletcher-4 End-to-End Checksum Validation          |   |
|   | - Uberblock Transaction Groups (TXG)                 |   |
|   +------------------------------------------------------+   |
+--------------------------------------------------------------+
```

---

## 2. ZFS Core Architecture

Hydra's ZFS implementation faithfully models the fundamental layers of OpenZFS:

### 1. Storage Pool Allocator (SPA)
- **Storage Pool**: Named `hydrapool` by default.
- **VDEV (Virtual Device)**: Manages physical block storage or memory-backed frames.
- **Copy-on-Write (CoW)**: Data blocks are never updated in-place. Writing allocates new blocks, preserves previous data for snapshots, and links to a new Merkle root.
- **Active Uberblock**: The root of the storage pool (`magic: 0x00bab10c`, `version: 5000`). Every write transaction commits an incremented Transaction Group (`TXG`).

### 2. Fletcher-4 End-to-End Checksums
Every block pointer (`blkptr_t`) carries a 256-bit Fletcher-4 checksum (4 parallel 64-bit accumulators):
$$\text{Fletcher-4: } a = \sum w_i, \quad b = \sum a_i, \quad c = \sum b_i, \quad d = \sum c_i$$
Upon read operations, the ZFS File Server recalculates the checksum and detects any silent data corruption before returning data to the caller.

### 3. Data Management Unit (DMU) & Dnodes
- Objects in ZFS are represented by `DnodePhys` structures (`obj_type`, block size, file size, direct block pointers).
- Supported Object Types:
  - `PlainFile`: Data payload blocks.
  - `Directory`: Associated with a ZAP object.
  - `Dataset`: Isolated filesystem namespace (`hydrapool/root`, `hydrapool/bin`).
  - `MasterNode`: Meta-Object Set (MOS) root.

### 4. ZFS Attribute Processor (ZAP)
- Micro-ZAP structure indexing directory entries (mapping string filenames to object IDs).
- Enables $O(1)$ file lookups inside datasets.

---

## 3. Microkernel IPC Protocol (`Endpoint #0x20`)

Clients interact with the File Server by sending `Message` envelopes through `ENDPOINT_ZFS_SERVER`.

| Protocol Label | Opcode | Arguments (`args`) | Return Values | Description |
| :--- | :--- | :--- | :--- | :--- |
| `ZFS_IPC_PING` | `0x3001` | - | `[status, txg, 0, 0]` | Server health check and active TXG. |
| `ZFS_IPC_LOOKUP` | `0x3002` | `[dir_obj, name_hash, 0, 0]` | `[status, obj_id, obj_type, 0]` | Resolves filename to object ID. |
| `ZFS_IPC_READ` | `0x3003` | `[obj_id, dest_buf_ptr, max_len, 0]` | `[status, bytes_read, txg, 0]` | Reads and checksum-verifies file data. |
| `ZFS_IPC_WRITE` | `0x3004` | `[obj_id, src_buf_ptr, len, 0]` | `[status, new_txg, 0, 0]` | CoW writes data and commits new TXG. |
| `ZFS_IPC_POOL_STATUS` | `0x3006` | - | `[status, total_mb, alloc_kb, free_txg]` | Reports pool metrics and health. |
| `ZFS_IPC_SNAPSHOT` | `0x3007` | `[dataset_id, 0, 0, 0]` | `[status, txg, 0, 0]` | Creates point-in-time snapshot. |

---

## 4. Capability-Based Security Enforcement

The File Server verifies client capabilities on every request:
- **Read Requests**: The client must present a capability containing `Rights::READ`. If missing, the server responds with `ZFS_STATUS_PERMISSION_DENIED`.
- **Write / CoW Transactions**: Requires `Rights::WRITE`.
- **Snapshots**: Requires `Rights::WRITE`.

---

## 5. Interactive Shell Commands

Hydra provides native user commands to interact with the ZFS File Server:

```
hydra> zpool status
  pool: hydrapool
 state: ONLINE
status: The pool is operating normally via the Microkernel ZFS File Server.
action: No known data errors detected; all blocks checksum-verified.
config:
        NAME                 STATE     READ WRITE CKSUM
        hydrapool            ONLINE       0     0     0
          vdev-ramdisk-0     ONLINE       0     0     0

Pool Properties & Metrics:
  - Capacity:           8 MiB
  - Allocated:          128 KiB
  - Free Space:         8064 KiB
  - Active Uberblock:   TXG #2
  - Checksum Algorithm: Fletcher-4 (4x 64-bit words)
  - Allocation Policy:  Copy-on-Write (CoW) Merkle Tree

hydra> zfs list
NAME                     USED    AVAIL MOUNTPOINT
------------------------------------------------------------------
hydrapool/root           128K    8064K /
hydrapool/bin            128K    8064K /bin

hydra> zfs cat readme.txt
Hydra OS Default Filesystem: OpenZFS Architecture
Integrated as an independent Microkernel File Server.
All operations are verified via Fletcher-4 checksums!

hydra> zfs write demo.txt "Hello from ZFS on Hydra Microkernel!"
[+] CoW write committed to ZFS File Server!
    - File: 'demo.txt' (36 bytes)
    - Merkle Root Updated to TXG #3
    - Fletcher-4 Checksum computed & stored
```

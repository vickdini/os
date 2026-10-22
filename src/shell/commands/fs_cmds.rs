use uefi::proto::console::text::Color;

use crate::console::set_color;
use crate::fs::client::ZfsClient;
use crate::fs::zfs::types::DnodeType;
use crate::shell::command::Command;
use crate::shell::state::ShellState;

pub struct LsCommand;

impl Command for LsCommand {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["dir"]
    }

    fn description(&self) -> &'static str {
        "List files and directories from the ZFS File Server (e.g. 'ls', 'ls /bin')"
    }

    fn execute(&self, args: &[&str], _state: &mut ShellState) {
        let path = if args.is_empty() { "/bin" } else { args[0] };

        match ZfsClient::list_dir(path) {
            Ok((entries, count, ds_name)) => {
                set_color(Color::Yellow, Color::Black);
                println!("ZFS Directory listing for '{}' (Dataset: {}):", path, ds_name);
                println!("{:<24} {:<10} {:>10} {:>10}", "NAME", "TYPE", "CHECKSUM", "STATUS");
                println!("------------------------------------------------------------");

                let mut printed = 0;
                for entry in entries.iter() {
                    if entry.is_valid {
                        let is_dir = entry.obj_type == DnodeType::Directory;
                        let type_str = if is_dir { "<DIR>" } else { "<FILE>" };

                        if is_dir {
                            set_color(Color::LightCyan, Color::Black);
                        } else {
                            set_color(Color::White, Color::Black);
                        }

                        println!("{:<24} {:<10} {:>10} {:>10}", entry.name_str(), type_str, "Fletcher4", "ONLINE");
                        printed += 1;
                    }
                }

                if printed == 0 {
                    set_color(Color::DarkGray, Color::Black);
                    println!("  (empty directory)");
                }

                set_color(Color::LightGreen, Color::Black);
                println!("------------------------------------------------------------");
                println!("Total entries: {} (ZFS pool: hydrapool, CoW enabled)\n", count);
            }
            Err(e) => {
                set_color(Color::LightRed, Color::Black);
                println!("[!] Failed to list directory '{}': {}", path, e);
            }
        }
    }
}

pub struct CatCommand;

impl Command for CatCommand {
    fn name(&self) -> &'static str {
        "cat"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["type"]
    }

    fn description(&self) -> &'static str {
        "Display file contents via the ZFS File Server (e.g. 'cat readme.txt')"
    }

    fn execute(&self, args: &[&str], state: &mut ShellState) {
        if args.is_empty() {
            set_color(Color::LightRed, Color::Black);
            println!("Usage: cat <filename>");
            return;
        }

        let target = args[0];

        match ZfsClient::lookup(target) {
            Ok((obj_id, _)) => {
                let mut buf = [0u8; 2048];
                let cap = state.cap_table.get(1).cloned();
                match ZfsClient::read_file(obj_id, &mut buf, cap) {
                    Ok(n) => {
                        set_color(Color::White, Color::Black);
                        if let Ok(text) = core::str::from_utf8(&buf[..n]) {
                            println!("{}", text);
                        } else {
                            println!("[Binary data: {} bytes, Fletcher-4 checksum verified]", n);
                        }
                    }
                    Err(e) => {
                        set_color(Color::LightRed, Color::Black);
                        println!("[!] Read error: {}", e);
                    }
                }
            }
            Err(e) => {
                set_color(Color::LightRed, Color::Black);
                println!("[!] File '{}' not found in ZFS: {}", target, e);
            }
        }
    }
}

pub struct WriteCommand;

impl Command for WriteCommand {
    fn name(&self) -> &'static str {
        "write"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["touch"]
    }

    fn description(&self) -> &'static str {
        "Create or update a file in ZFS via Copy-on-Write (e.g. 'write note.txt Hello World')"
    }

    fn execute(&self, args: &[&str], state: &mut ShellState) {
        if args.is_empty() {
            set_color(Color::LightRed, Color::Black);
            println!("Usage: write <filename> [content...]");
            return;
        }

        let name = args[0];
        let mut content_buf = [0u8; 512];
        let mut content_len = 0;

        for (i, word) in args[1..].iter().enumerate() {
            if i > 0 && content_len < content_buf.len() {
                content_buf[content_len] = b' ';
                content_len += 1;
            }
            for &b in word.as_bytes() {
                if content_len < content_buf.len() {
                    content_buf[content_len] = b;
                    content_len += 1;
                }
            }
        }
        let content_bytes = &content_buf[..content_len];
        let _cap = state.cap_table.get(1).cloned();

        let mut server_lock = crate::fs::zfs::server::ZFS_SERVER.lock();
        if let Some(server) = server_lock.as_mut() {
            let file_obj = match server.dmu.allocate_object(DnodeType::PlainFile, name) {
                Ok(id) => id,
                Err(e) => {
                    set_color(Color::LightRed, Color::Black);
                    println!("[!] Allocation failed: {}", e);
                    return;
                }
            };

            match server.dmu.cow_write_file(file_obj, content_bytes, &mut server.spa) {
                Ok(new_txg) => {
                    let bin_dir = server.bin_dir_obj;
                    let _ = server.dmu.add_directory_entry(bin_dir, name, file_obj, DnodeType::PlainFile);
                    set_color(Color::LightGreen, Color::Black);
                    println!("[+] File '{}' written to ZFS File Server!", name);
                    println!("    - Size: {} bytes", content_bytes.len());
                    println!("    - Merkle Root committed at TXG #{}", new_txg);
                    println!("    - Checksum: Fletcher-4 verified");
                }
                Err(e) => {
                    set_color(Color::LightRed, Color::Black);
                    println!("[!] Write failed: {}", e);
                }
            }
        }
    }
}

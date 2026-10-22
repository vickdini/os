use crate::shell::state::ShellState;

/// Common trait that all built-in commands and integrated programs implement.
pub trait Command: Sync {
    /// The primary invocation keyword (e.g. "mem", "help", "alloc")
    fn name(&self) -> &'static str;

    /// Alternative invocation names (e.g. ["?"] for help, ["memory"] for mem)
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    /// Brief description shown in the manual listing
    fn description(&self) -> &'static str;

    /// Command syntax and argument help
    fn usage(&self) -> &'static str {
        ""
    }

    /// Executes the command with the parsed argument slice
    fn execute(&self, args: &[&str], state: &mut ShellState);
}

// Utility parsing helpers shared across commands
pub fn parse_hex(s: &str) -> Option<u64> {
    let s = s.trim();
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    if s.is_empty() {
        return None;
    }
    let mut val: u64 = 0;
    for &b in s.as_bytes() {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as u64,
            b'a'..=b'f' => (b - b'a' + 10) as u64,
            b'A'..=b'F' => (b - b'A' + 10) as u64,
            _ => return None,
        };
        val = val.checked_mul(16)?.checked_add(digit)?;
    }
    Some(val)
}

pub fn parse_u64(s: &str) -> Option<u64> {
    if s.starts_with("0x") || s.starts_with("0X") {
        parse_hex(s)
    } else {
        let mut val: u64 = 0;
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        for &b in s.as_bytes() {
            if !(b'0'..=b'9').contains(&b) {
                return None;
            }
            val = val.checked_mul(10)?.checked_add((b - b'0') as u64)?;
        }
        Some(val)
    }
}

pub fn parse_u32(s: &str) -> Option<u32> {
    parse_u64(s).and_then(|v| u32::try_from(v).ok())
}

pub fn parse_usize(s: &str) -> Option<usize> {
    parse_u64(s).and_then(|v| usize::try_from(v).ok())
}

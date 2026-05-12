use std::fs;
use std::io::Write;

use crate::dict::DictCandidate;
use crate::trie::Trie;

const MAGIC: &[u8; 4] = b"ZIRC";
const VERSION: u8 = 1;

/// Compile a list of (key, entry) pairs into a binary cache file.
pub fn compile_to_cache(path: &str, entries: &[(String, DictCandidate)], checksum: u64) {
    let mut buf: Vec<u8> = Vec::new();

    // Header
    buf.extend_from_slice(MAGIC);
    buf.push(VERSION);
    buf.extend_from_slice(&checksum.to_le_bytes());
    buf.extend_from_slice(&(entries.len() as u32).to_le_bytes());

    // Entries
    for (key, entry) in entries {
        let key_bytes = key.as_bytes();
        let text_bytes = entry.text.as_bytes();
        buf.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(key_bytes);
        buf.extend_from_slice(&(text_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(text_bytes);
        buf.extend_from_slice(&entry.weight.to_le_bytes());
    }

    if let Ok(mut file) = fs::File::create(path) {
        let _ = file.write_all(&buf);
    }
}

/// Load a Trie from a binary cache file.
/// Returns None if the file doesn't exist, has invalid format, or checksum mismatch.
pub fn load_from_cache(path: &str, expected_checksum: u64) -> Option<Trie<DictCandidate>> {
    let data = fs::read(path).ok()?;

    if data.len() < 17 {
        return None;
    }

    // Verify magic and version
    if &data[0..4] != MAGIC || data[4] != VERSION {
        return None;
    }

    // Verify checksum
    let stored_checksum = u64::from_le_bytes([
        data[5], data[6], data[7], data[8], data[9], data[10], data[11], data[12],
    ]);
    if stored_checksum != expected_checksum {
        return None;
    }

    let entry_count = u32::from_le_bytes([data[13], data[14], data[15], data[16]]) as usize;
    let mut trie = Trie::new();
    let mut pos = 17;

    for _ in 0..entry_count {
        if pos + 2 > data.len() {
            return None;
        }
        let key_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        if pos + key_len > data.len() {
            return None;
        }
        let key = std::str::from_utf8(&data[pos..pos + key_len]).ok()?;
        pos += key_len;

        if pos + 2 > data.len() {
            return None;
        }
        let text_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        if pos + text_len > data.len() {
            return None;
        }
        let text = std::str::from_utf8(&data[pos..pos + text_len])
            .ok()?
            .to_string();
        pos += text_len;

        if pos + 4 > data.len() {
            return None;
        }
        let weight = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;

        trie.insert(key, DictCandidate { text, weight });
    }

    Some(trie)
}

/// Generate a simple checksum from source file paths, modification times, and sizes.
pub fn compute_source_checksum(paths: &[String]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for path in paths {
        for byte in path.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        if let Ok(meta) = fs::metadata(path) {
            // Hash file size
            let size = meta.len();
            for byte in size.to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            // Hash modification time
            if let Ok(modified) = meta.modified() {
                if let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH) {
                    for byte in dur.as_secs().to_le_bytes() {
                        hash ^= byte as u64;
                        hash = hash.wrapping_mul(0x100000001b3);
                    }
                }
            }
        }
    }
    hash
}

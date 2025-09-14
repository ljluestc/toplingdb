//! Utility functions and data structures

pub mod coding;
pub mod crc;
pub mod hash;
pub mod bloom;

use std::cmp::Ordering;

/// Compare two byte slices
pub fn compare_keys(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)
}

/// Calculate the next key (for range operations)
pub fn next_key(key: &[u8]) -> Vec<u8> {
    let mut result = key.to_vec();

    // Find the last byte that can be incremented
    for i in (0..result.len()).rev() {
        if result[i] != 0xFF {
            result[i] += 1;
            return result;
        }
        result[i] = 0;
    }

    // All bytes were 0xFF, append a 0x00
    result.push(0x00);
    result
}

/// Calculate previous key
pub fn prev_key(key: &[u8]) -> Option<Vec<u8>> {
    if key.is_empty() {
        return None;
    }

    let mut result = key.to_vec();

    // Find the last byte that can be decremented
    for i in (0..result.len()).rev() {
        if result[i] != 0x00 {
            result[i] -= 1;
            // Fill remaining bytes with 0xFF
            for j in (i + 1)..result.len() {
                result[j] = 0xFF;
            }
            return Some(result);
        }
    }

    // All bytes were 0x00
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_key() {
        assert_eq!(next_key(b"abc"), b"abd");
        assert_eq!(next_key(b"ab\xff"), b"ac\x00");
        assert_eq!(next_key(b"\xff\xff"), b"\x00\x00\x00");
        assert_eq!(next_key(b""), b"\x00");
    }

    #[test]
    fn test_prev_key() {
        assert_eq!(prev_key(b"abc"), Some(b"abb".to_vec()));
        assert_eq!(prev_key(b"ab\x00"), Some(b"aa\xff".to_vec()));
        assert_eq!(prev_key(b"\x00\x00"), None);
        assert_eq!(prev_key(b""), None);
    }
}
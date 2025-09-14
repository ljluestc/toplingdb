//! Internal key format and utilities

use crate::slice::Slice;
use std::cmp::Ordering;

/// Sequence number type
pub type SequenceNumber = u64;

/// Value type for internal keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueType {
    /// Deletion marker
    Delete = 0,
    /// Regular value
    Value = 1,
    /// Merge value
    Merge = 2,
}

impl ValueType {
    /// Create from byte
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(ValueType::Delete),
            1 => Some(ValueType::Value),
            2 => Some(ValueType::Merge),
            _ => None,
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// Internal key format: user_key + sequence_number + value_type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalKey {
    /// The encoded key data
    data: Vec<u8>,
}

impl InternalKey {
    /// Create a new internal key
    pub fn new(user_key: &[u8], sequence_number: SequenceNumber, value_type: ValueType) -> Self {
        let mut data = Vec::with_capacity(user_key.len() + 8);
        data.extend_from_slice(user_key);

        // Pack sequence number (56 bits) and value type (8 bits) into 8 bytes
        let packed = (sequence_number << 8) | (value_type.to_byte() as u64);
        data.extend_from_slice(&packed.to_le_bytes());

        Self { data }
    }

    /// Get the user key part
    pub fn user_key(&self) -> &[u8] {
        if self.data.len() >= 8 {
            &self.data[..self.data.len() - 8]
        } else {
            &[]
        }
    }

    /// Get the sequence number
    pub fn sequence_number(&self) -> SequenceNumber {
        if self.data.len() >= 8 {
            let start = self.data.len() - 8;
            let packed = u64::from_le_bytes([
                self.data[start], self.data[start + 1], self.data[start + 2], self.data[start + 3],
                self.data[start + 4], self.data[start + 5], self.data[start + 6], self.data[start + 7],
            ]);
            packed >> 8
        } else {
            0
        }
    }

    /// Get the value type
    pub fn value_type(&self) -> ValueType {
        if self.data.len() >= 8 {
            let start = self.data.len() - 8;
            let packed = u64::from_le_bytes([
                self.data[start], self.data[start + 1], self.data[start + 2], self.data[start + 3],
                self.data[start + 4], self.data[start + 5], self.data[start + 6], self.data[start + 7],
            ]);
            ValueType::from_byte((packed & 0xFF) as u8).unwrap_or(ValueType::Value)
        } else {
            ValueType::Value
        }
    }

    /// Get the encoded internal key data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the size of the internal key
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Create from encoded data
    pub fn from_data(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Decode from slice
    pub fn decode_from(data: &[u8]) -> Option<Self> {
        if data.len() >= 8 {
            Some(Self { data: data.to_vec() })
        } else {
            None
        }
    }
}

impl PartialOrd for InternalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InternalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare user keys
        match self.user_key().cmp(other.user_key()) {
            Ordering::Equal => {
                // If user keys are equal, compare by sequence number (descending)
                // Higher sequence numbers come first
                other.sequence_number().cmp(&self.sequence_number())
            }
            other => other,
        }
    }
}

/// Parsed internal key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInternalKey {
    /// User key
    pub user_key: Vec<u8>,
    /// Sequence number
    pub sequence_number: SequenceNumber,
    /// Value type
    pub value_type: ValueType,
}

impl ParsedInternalKey {
    /// Create a new parsed internal key
    pub fn new(user_key: Vec<u8>, sequence_number: SequenceNumber, value_type: ValueType) -> Self {
        Self {
            user_key,
            sequence_number,
            value_type,
        }
    }

    /// Encode to internal key
    pub fn encode(&self) -> InternalKey {
        InternalKey::new(&self.user_key, self.sequence_number, self.value_type)
    }
}

/// Parse an internal key
pub fn parse_internal_key(internal_key: &[u8]) -> Option<ParsedInternalKey> {
    if internal_key.len() < 8 {
        return None;
    }

    let user_key_len = internal_key.len() - 8;
    let user_key = internal_key[..user_key_len].to_vec();

    let start = user_key_len;
    let packed = u64::from_le_bytes([
        internal_key[start], internal_key[start + 1], internal_key[start + 2], internal_key[start + 3],
        internal_key[start + 4], internal_key[start + 5], internal_key[start + 6], internal_key[start + 7],
    ]);

    let sequence_number = packed >> 8;
    let value_type = ValueType::from_byte((packed & 0xFF) as u8)?;

    Some(ParsedInternalKey {
        user_key,
        sequence_number,
        value_type,
    })
}

/// Extract user key from internal key
pub fn extract_user_key(internal_key: &[u8]) -> &[u8] {
    if internal_key.len() >= 8 {
        &internal_key[..internal_key.len() - 8]
    } else {
        &[]
    }
}

/// User key comparator
pub struct UserKeyComparator;

impl UserKeyComparator {
    pub fn new() -> Self {
        Self
    }

    /// Compare two user keys
    pub fn compare(&self, a: &[u8], b: &[u8]) -> Ordering {
        a.cmp(b)
    }

    /// Get the name of this comparator
    pub fn name(&self) -> &str {
        "leveldb.BytewiseComparator"
    }

    /// Find the shortest separator between two keys
    pub fn find_short_separator(&self, start: &mut Vec<u8>, limit: &[u8]) {
        // Find the first byte where start and limit differ
        let min_len = start.len().min(limit.len());
        let mut diff_index = 0;

        while diff_index < min_len && start[diff_index] == limit[diff_index] {
            diff_index += 1;
        }

        if diff_index < min_len && start[diff_index] < limit[diff_index] {
            // Try to increment the differing byte
            if start[diff_index] < 255 {
                start[diff_index] += 1;
                start.truncate(diff_index + 1);
            }
        }
    }

    /// Find a short key that is >= key
    pub fn find_short_successor(&self, key: &mut Vec<u8>) {
        // Find the first byte that can be incremented
        for i in 0..key.len() {
            if key[i] != 255 {
                key[i] += 1;
                key.truncate(i + 1);
                return;
            }
        }
        // All bytes are 255, can't make it shorter
    }
}

impl Default for UserKeyComparator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type() {
        assert_eq!(ValueType::Delete.to_byte(), 0);
        assert_eq!(ValueType::Value.to_byte(), 1);
        assert_eq!(ValueType::Merge.to_byte(), 2);

        assert_eq!(ValueType::from_byte(0), Some(ValueType::Delete));
        assert_eq!(ValueType::from_byte(1), Some(ValueType::Value));
        assert_eq!(ValueType::from_byte(2), Some(ValueType::Merge));
        assert_eq!(ValueType::from_byte(3), None);
    }

    #[test]
    fn test_internal_key() {
        let user_key = b"hello";
        let seq_num = 12345;
        let value_type = ValueType::Value;

        let internal_key = InternalKey::new(user_key, seq_num, value_type);

        assert_eq!(internal_key.user_key(), user_key);
        assert_eq!(internal_key.sequence_number(), seq_num);
        assert_eq!(internal_key.value_type(), value_type);
    }

    #[test]
    fn test_parse_internal_key() {
        let user_key = b"test_key";
        let seq_num = 98765;
        let value_type = ValueType::Delete;

        let internal_key = InternalKey::new(user_key, seq_num, value_type);
        let parsed = parse_internal_key(internal_key.data()).unwrap();

        assert_eq!(parsed.user_key, user_key);
        assert_eq!(parsed.sequence_number, seq_num);
        assert_eq!(parsed.value_type, value_type);
    }

    #[test]
    fn test_internal_key_ordering() {
        let key1 = InternalKey::new(b"a", 100, ValueType::Value);
        let key2 = InternalKey::new(b"b", 50, ValueType::Value);
        let key3 = InternalKey::new(b"a", 200, ValueType::Value);

        // Different user keys
        assert!(key1 < key2);

        // Same user key, different sequence numbers (higher seq num comes first)
        assert!(key3 < key1);
    }

    #[test]
    fn test_extract_user_key() {
        let user_key = b"extracted_key";
        let internal_key = InternalKey::new(user_key, 555, ValueType::Merge);
        let extracted = extract_user_key(internal_key.data());

        assert_eq!(extracted, user_key);
    }

    #[test]
    fn test_user_key_comparator() {
        let comp = UserKeyComparator::new();

        assert_eq!(comp.compare(b"a", b"b"), Ordering::Less);
        assert_eq!(comp.compare(b"b", b"a"), Ordering::Greater);
        assert_eq!(comp.compare(b"a", b"a"), Ordering::Equal);
    }

    #[test]
    fn test_find_short_separator() {
        let comp = UserKeyComparator::new();

        let mut start = b"abc".to_vec();
        let limit = b"abd";
        comp.find_short_separator(&mut start, limit);
        // Should become "abd" or shorter
        assert!(start <= b"abd".to_vec());
        assert!(start >= b"abc".to_vec());
    }

    #[test]
    fn test_find_short_successor() {
        let comp = UserKeyComparator::new();

        let mut key = b"abc".to_vec();
        comp.find_short_successor(&mut key);
        // Should be "abd" or "ac" etc.
        assert!(key > b"abc".to_vec());
    }

    #[test]
    fn test_invalid_internal_key() {
        let short_data = b"short"; // Less than 8 bytes
        let parsed = parse_internal_key(short_data);
        assert!(parsed.is_none());

        let extracted = extract_user_key(short_data);
        assert!(extracted.is_empty());
    }
}
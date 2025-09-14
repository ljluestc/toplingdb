//! In-memory table implementation

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use crate::options::ColumnFamilyOptions;
use crate::slice::{Slice, OwnedSlice};
use crate::status::{Status, StatusCode};
use crate::write_batch::{WriteBatch, WriteOperation, WriteType};
use crate::internal::keys::{InternalKey, ValueType};


impl From<WriteType> for ValueType {
    fn from(write_type: WriteType) -> Self {
        match write_type {
            WriteType::Put => Self::Value,
            WriteType::Delete => Self::Delete,
            WriteType::SingleDelete => Self::Delete,
            WriteType::Merge => Self::Merge,
            WriteType::DeleteRange => Self::Delete, // Treat as regular delete for now
        }
    }
}

/// An entry in the memtable
#[derive(Debug, Clone)]
pub struct MemTableEntry {
    /// The key
    pub key: Vec<u8>,
    /// The value
    pub value: Vec<u8>,
    /// The value type
    pub value_type: ValueType,
    /// Sequence number
    pub sequence_number: u64,
}

impl MemTableEntry {
    /// Create a new memtable entry
    pub fn new(key: Vec<u8>, value: Vec<u8>, value_type: ValueType, sequence_number: u64) -> Self {
        Self {
            key,
            value,
            value_type,
            sequence_number,
        }
    }

    /// Create a put entry
    pub fn put(key: Vec<u8>, value: Vec<u8>, sequence_number: u64) -> Self {
        Self::new(key, value, ValueType::Value, sequence_number)
    }

    /// Create a delete entry
    pub fn delete(key: Vec<u8>, sequence_number: u64) -> Self {
        Self::new(key, Vec::new(), ValueType::Delete, sequence_number)
    }

    /// Create a merge entry
    pub fn merge(key: Vec<u8>, value: Vec<u8>, sequence_number: u64) -> Self {
        Self::new(key, value, ValueType::Merge, sequence_number)
    }

    /// Check if this is a deletion
    pub fn is_deletion(&self) -> bool {
        matches!(self.value_type, ValueType::Delete)
    }

    /// Get the size of this entry
    pub fn size(&self) -> usize {
        self.key.len() + self.value.len() + std::mem::size_of::<ValueType>() + std::mem::size_of::<u64>()
    }

    /// Create internal key for sorting
    pub fn internal_key(&self) -> InternalKey {
        InternalKey::new(&self.key, self.sequence_number, self.value_type)
    }
}


/// Memtable implementation using BTreeMap
pub struct MemTable {
    /// The actual data storage
    data: RwLock<BTreeMap<InternalKey, MemTableEntry>>,
    /// Column family options
    options: ColumnFamilyOptions,
    /// Memory usage approximation
    memory_usage: std::sync::atomic::AtomicUsize,
    /// Number of entries
    num_entries: std::sync::atomic::AtomicUsize,
}

impl MemTable {
    /// Create a new memtable
    pub fn new(options: ColumnFamilyOptions) -> Self {
        Self {
            data: RwLock::new(BTreeMap::new()),
            options,
            memory_usage: std::sync::atomic::AtomicUsize::new(0),
            num_entries: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Insert a single key-value pair
    pub fn insert(&self, key: &[u8], value: &[u8], value_type: ValueType, sequence_number: u64) -> Result<(), Status> {
        let entry = MemTableEntry::new(key.to_vec(), value.to_vec(), value_type, sequence_number);
        let internal_key = entry.internal_key();
        let entry_size = entry.size();

        let mut data = self.data.write().unwrap();
        data.insert(internal_key, entry);

        self.memory_usage.fetch_add(entry_size, std::sync::atomic::Ordering::Relaxed);
        self.num_entries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Insert a write batch
    pub fn insert_batch(&self, batch: &WriteBatch, sequence_start: u64) -> Result<(), Status> {
        let mut sequence_number = sequence_start;

        for operation in batch.operations() {
            match operation.op_type {
                WriteType::Put => {
                    self.insert(&operation.key, &operation.value, ValueType::Value, sequence_number)?;
                }
                WriteType::Delete => {
                    self.insert(&operation.key, &[], ValueType::Delete, sequence_number)?;
                }
                WriteType::SingleDelete => {
                    self.insert(&operation.key, &[], ValueType::Delete, sequence_number)?;
                }
                WriteType::Merge => {
                    self.insert(&operation.key, &operation.value, ValueType::Merge, sequence_number)?;
                }
                WriteType::DeleteRange => {
                    // TODO: Implement proper range deletion
                    self.insert(&operation.key, &[], ValueType::Delete, sequence_number)?;
                }
            }
            sequence_number += 1;
        }

        Ok(())
    }

    /// Get a value for a key
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Status> {
        let data = self.data.read().unwrap();

        // Find the latest version of this key
        for (internal_key, entry) in data.range(
            InternalKey::new(key, u64::MAX, ValueType::Value)..
        ) {
            if internal_key.user_key() != key {
                break;
            }

            match entry.value_type {
                ValueType::Value => return Ok(Some(entry.value.clone())),
                ValueType::Delete => return Ok(None),
                ValueType::Merge => {
                    // TODO: Implement proper merge handling
                    return Ok(Some(entry.value.clone()));
                }
            }
        }

        Ok(None)
    }

    /// Check if the key exists
    pub fn contains(&self, key: &[u8]) -> bool {
        self.get(key).unwrap_or(None).is_some()
    }

    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.memory_usage.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get number of entries
    pub fn num_entries(&self) -> usize {
        self.num_entries.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Check if memtable is empty
    pub fn is_empty(&self) -> bool {
        self.num_entries() == 0
    }

    /// Get approximate size of a range
    pub fn approximate_size(&self, start: &[u8], end: &[u8]) -> u64 {
        let data = self.data.read().unwrap();
        let mut size = 0u64;

        for (internal_key, entry) in data.range(
            InternalKey::new(start, u64::MAX, ValueType::Value)..
            InternalKey::new(end, 0, ValueType::Value)
        ) {
            if internal_key.user_key() >= end {
                break;
            }
            size += entry.size() as u64;
        }

        size
    }

    /// Create an iterator over the memtable
    pub fn iter(&self) -> MemTableIterator {
        MemTableIterator::new(self)
    }

    /// Get all entries (for testing)
    #[cfg(test)]
    pub fn entries(&self) -> Vec<MemTableEntry> {
        let data = self.data.read().unwrap();
        data.values().cloned().collect()
    }
}

/// Iterator over memtable entries
pub struct MemTableIterator {
    /// Snapshot of the data
    data: Vec<(InternalKey, MemTableEntry)>,
    /// Current position
    position: Option<usize>,
}

impl MemTableIterator {
    /// Create a new iterator
    fn new(memtable: &MemTable) -> Self {
        let data = memtable.data.read().unwrap();
        let snapshot: Vec<_> = data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        Self {
            data: snapshot,
            position: None,
        }
    }

    /// Check if iterator is valid
    pub fn valid(&self) -> bool {
        self.position.map_or(false, |pos| pos < self.data.len())
    }

    /// Seek to first
    pub fn seek_to_first(&mut self) {
        if !self.data.is_empty() {
            self.position = Some(0);
        } else {
            self.position = None;
        }
    }

    /// Seek to last
    pub fn seek_to_last(&mut self) {
        if !self.data.is_empty() {
            self.position = Some(self.data.len() - 1);
        } else {
            self.position = None;
        }
    }

    /// Seek to key
    pub fn seek(&mut self, target: &[u8]) {
        let target_key = InternalKey::new(target, u64::MAX, ValueType::Value);

        match self.data.binary_search_by_key(&target_key, |(k, _)| k.clone()) {
            Ok(pos) => self.position = Some(pos),
            Err(pos) => {
                if pos < self.data.len() {
                    self.position = Some(pos);
                } else {
                    self.position = None;
                }
            }
        }
    }

    /// Move to next
    pub fn next(&mut self) {
        if let Some(pos) = self.position {
            if pos + 1 < self.data.len() {
                self.position = Some(pos + 1);
            } else {
                self.position = None;
            }
        }
    }

    /// Move to previous
    pub fn prev(&mut self) {
        if let Some(pos) = self.position {
            if pos > 0 {
                self.position = Some(pos - 1);
            } else {
                self.position = None;
            }
        }
    }

    /// Get current key
    pub fn key(&self) -> Option<&[u8]> {
        self.position.and_then(|pos| self.data.get(pos).map(|(k, _)| k.user_key()))
    }

    /// Get current value
    pub fn value(&self) -> Option<&[u8]> {
        self.position.and_then(|pos| self.data.get(pos).map(|(_, v)| v.value.as_slice()))
    }

    /// Get current entry
    pub fn entry(&self) -> Option<&MemTableEntry> {
        self.position.and_then(|pos| self.data.get(pos).map(|(_, v)| v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memtable() -> MemTable {
        MemTable::new(ColumnFamilyOptions::default())
    }

    #[test]
    fn test_internal_key_ordering() {
        let key1 = InternalKey::new(b"key1", 100, ValueType::Value);
        let key2 = InternalKey::new(b"key1", 90, ValueType::Value);
        let key3 = InternalKey::new(b"key2", 100, ValueType::Value);

        // Same user key, newer sequence number comes first
        assert!(key1 < key2);
        // Different user keys
        assert!(key1 < key3);
    }


    #[test]
    fn test_memtable_put_get() {
        let memtable = create_test_memtable();

        // Insert some data
        memtable.insert(b"key1", b"value1", ValueType::Value, 1).unwrap();
        memtable.insert(b"key2", b"value2", ValueType::Value, 2).unwrap();

        // Test get
        assert_eq!(memtable.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(memtable.get(b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(memtable.get(b"nonexistent").unwrap(), None);

        // Test stats
        assert_eq!(memtable.num_entries(), 2);
        assert!(memtable.memory_usage() > 0);
        assert!(!memtable.is_empty());
    }

    #[test]
    fn test_memtable_delete() {
        let memtable = create_test_memtable();

        // Insert and then delete
        memtable.insert(b"key1", b"value1", ValueType::Value, 1).unwrap();
        assert_eq!(memtable.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        memtable.insert(b"key1", b"", ValueType::Delete, 2).unwrap();
        assert_eq!(memtable.get(b"key1").unwrap(), None);
    }

    #[test]
    fn test_memtable_versioning() {
        let memtable = create_test_memtable();

        // Insert multiple versions of the same key
        memtable.insert(b"key1", b"value1", ValueType::Value, 1).unwrap();
        memtable.insert(b"key1", b"value2", ValueType::Value, 2).unwrap();
        memtable.insert(b"key1", b"value3", ValueType::Value, 3).unwrap();

        // Should get the latest version
        assert_eq!(memtable.get(b"key1").unwrap(), Some(b"value3".to_vec()));
    }

    #[test]
    fn test_write_batch_insert() {
        let memtable = create_test_memtable();
        let mut batch = WriteBatch::new();

        batch.put(b"key1", b"value1");
        batch.put(b"key2", b"value2");
        batch.delete(b"key3");

        memtable.insert_batch(&batch, 100).unwrap();

        assert_eq!(memtable.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(memtable.get(b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(memtable.get(b"key3").unwrap(), None);
        assert_eq!(memtable.num_entries(), 3);
    }

    #[test]
    fn test_memtable_iterator() {
        let memtable = create_test_memtable();

        // Insert test data
        memtable.insert(b"key3", b"value3", ValueType::Value, 3).unwrap();
        memtable.insert(b"key1", b"value1", ValueType::Value, 1).unwrap();
        memtable.insert(b"key2", b"value2", ValueType::Value, 2).unwrap();

        let mut iter = memtable.iter();

        // Test seek to first
        iter.seek_to_first();
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"key1".as_slice()));
        assert_eq!(iter.value(), Some(b"value1".as_slice()));

        // Test next
        iter.next();
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"key2".as_slice()));

        iter.next();
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"key3".as_slice()));

        iter.next();
        assert!(!iter.valid());

        // Test seek to specific key
        iter.seek(b"key2");
        assert!(iter.valid());
        assert_eq!(iter.key(), Some(b"key2".as_slice()));
    }

    #[test]
    fn test_approximate_size() {
        let memtable = create_test_memtable();

        memtable.insert(b"a", b"value_a", ValueType::Value, 1).unwrap();
        memtable.insert(b"b", b"value_b", ValueType::Value, 2).unwrap();
        memtable.insert(b"c", b"value_c", ValueType::Value, 3).unwrap();
        memtable.insert(b"d", b"value_d", ValueType::Value, 4).unwrap();

        let size = memtable.approximate_size(b"b", b"d");
        assert!(size > 0);

        let full_size = memtable.approximate_size(b"a", b"z");
        assert!(full_size > size);
    }
}
//! Write batch for atomic operations

use std::collections::VecDeque;

use crate::column_family::{ColumnFamilyHandle, DEFAULT_CF_NAME};
use crate::slice::Slice;
use crate::status::Status;

/// Type of operation in write batch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteType {
    /// Put operation
    Put,
    /// Delete operation
    Delete,
    /// Single delete operation
    SingleDelete,
    /// Delete range operation
    DeleteRange,
    /// Merge operation
    Merge,
}

/// A single write operation
#[derive(Debug, Clone)]
pub struct WriteOperation {
    /// Type of operation
    pub op_type: WriteType,
    /// Column family ID
    pub cf_id: u32,
    /// Key
    pub key: Vec<u8>,
    /// Value (empty for delete operations)
    pub value: Vec<u8>,
}

impl WriteOperation {
    /// Create a new put operation
    pub fn put(cf_id: u32, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            op_type: WriteType::Put,
            cf_id,
            key,
            value,
        }
    }

    /// Create a new delete operation
    pub fn delete(cf_id: u32, key: Vec<u8>) -> Self {
        Self {
            op_type: WriteType::Delete,
            cf_id,
            key,
            value: Vec::new(),
        }
    }

    /// Create a new single delete operation
    pub fn single_delete(cf_id: u32, key: Vec<u8>) -> Self {
        Self {
            op_type: WriteType::SingleDelete,
            cf_id,
            key,
            value: Vec::new(),
        }
    }

    /// Create a new delete range operation
    pub fn delete_range(cf_id: u32, start_key: Vec<u8>, end_key: Vec<u8>) -> Self {
        Self {
            op_type: WriteType::DeleteRange,
            cf_id,
            key: start_key,
            value: end_key,
        }
    }

    /// Create a new merge operation
    pub fn merge(cf_id: u32, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            op_type: WriteType::Merge,
            cf_id,
            key,
            value,
        }
    }

    /// Get the estimated size of this operation
    pub fn size(&self) -> usize {
        std::mem::size_of::<WriteType>()
            + std::mem::size_of::<u32>()
            + self.key.len()
            + self.value.len()
    }
}

/// Write batch for atomic writes
pub struct WriteBatch {
    /// Operations in this batch
    operations: VecDeque<WriteOperation>,
    /// Total size of the batch
    size: usize,
    /// Number of operations
    count: usize,
}

impl WriteBatch {
    /// Create a new empty write batch
    pub fn new() -> Self {
        Self {
            operations: VecDeque::new(),
            size: 0,
            count: 0,
        }
    }

    /// Create a write batch with reserved capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            operations: VecDeque::with_capacity(capacity),
            size: 0,
            count: 0,
        }
    }

    /// Put a key-value pair (default column family)
    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.put_cf(&ColumnFamilyHandle::default_cf(), key, value);
    }

    /// Put a key-value pair in a specific column family
    pub fn put_cf(&mut self, cf: &ColumnFamilyHandle, key: &[u8], value: &[u8]) {
        let op = WriteOperation::put(cf.id(), key.to_vec(), value.to_vec());
        self.size += op.size();
        self.count += 1;
        self.operations.push_back(op);
    }

    /// Delete a key (default column family)
    pub fn delete(&mut self, key: &[u8]) {
        self.delete_cf(&ColumnFamilyHandle::default_cf(), key);
    }

    /// Delete a key from a specific column family
    pub fn delete_cf(&mut self, cf: &ColumnFamilyHandle, key: &[u8]) {
        let op = WriteOperation::delete(cf.id(), key.to_vec());
        self.size += op.size();
        self.count += 1;
        self.operations.push_back(op);
    }

    /// Single delete a key (default column family)
    /// This is an optimization for cases where you know the key has only one version
    pub fn single_delete(&mut self, key: &[u8]) {
        self.single_delete_cf(&ColumnFamilyHandle::default_cf(), key);
    }

    /// Single delete a key from a specific column family
    pub fn single_delete_cf(&mut self, cf: &ColumnFamilyHandle, key: &[u8]) {
        let op = WriteOperation::single_delete(cf.id(), key.to_vec());
        self.size += op.size();
        self.count += 1;
        self.operations.push_back(op);
    }

    /// Delete a range of keys (default column family)
    pub fn delete_range(&mut self, start_key: &[u8], end_key: &[u8]) {
        self.delete_range_cf(&ColumnFamilyHandle::default_cf(), start_key, end_key);
    }

    /// Delete a range of keys from a specific column family
    pub fn delete_range_cf(&mut self, cf: &ColumnFamilyHandle, start_key: &[u8], end_key: &[u8]) {
        let op = WriteOperation::delete_range(cf.id(), start_key.to_vec(), end_key.to_vec());
        self.size += op.size();
        self.count += 1;
        self.operations.push_back(op);
    }

    /// Merge a value (default column family)
    pub fn merge(&mut self, key: &[u8], value: &[u8]) {
        self.merge_cf(&ColumnFamilyHandle::default_cf(), key, value);
    }

    /// Merge a value in a specific column family
    pub fn merge_cf(&mut self, cf: &ColumnFamilyHandle, key: &[u8], value: &[u8]) {
        let op = WriteOperation::merge(cf.id(), key.to_vec(), value.to_vec());
        self.size += op.size();
        self.count += 1;
        self.operations.push_back(op);
    }

    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
        self.size = 0;
        self.count = 0;
    }

    /// Get the number of operations in this batch
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get the total size of this batch
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Get an iterator over the operations
    pub fn iter(&self) -> impl Iterator<Item = &WriteOperation> {
        self.operations.iter()
    }

    /// Get the operations as a slice
    pub fn operations(&self) -> impl Iterator<Item = &WriteOperation> {
        self.operations.iter()
    }

    /// Pop the first operation
    pub fn pop_front(&mut self) -> Option<WriteOperation> {
        if let Some(op) = self.operations.pop_front() {
            self.size -= op.size();
            self.count -= 1;
            Some(op)
        } else {
            None
        }
    }

    /// Get the first operation without removing it
    pub fn front(&self) -> Option<&WriteOperation> {
        self.operations.front()
    }

    /// Check if batch contains any updates (puts or merges)
    pub fn has_put(&self) -> bool {
        self.operations.iter().any(|op| matches!(op.op_type, WriteType::Put | WriteType::Merge))
    }

    /// Check if batch contains any deletes
    pub fn has_delete(&self) -> bool {
        self.operations.iter().any(|op| matches!(
            op.op_type,
            WriteType::Delete | WriteType::SingleDelete | WriteType::DeleteRange
        ))
    }

    /// Check if batch contains any merges
    pub fn has_merge(&self) -> bool {
        self.operations.iter().any(|op| matches!(op.op_type, WriteType::Merge))
    }

    /// Append another write batch to this one
    pub fn append(&mut self, other: &WriteBatch) {
        for op in other.iter() {
            self.operations.push_back(op.clone());
            self.size += op.size();
            self.count += 1;
        }
    }

    /// Get the data representation for WAL writing
    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size);

        // Write count
        data.extend_from_slice(&(self.count as u32).to_le_bytes());

        // Write operations
        for op in &self.operations {
            // Write operation type
            data.push(op.op_type as u8);

            // Write column family ID
            data.extend_from_slice(&op.cf_id.to_le_bytes());

            // Write key length and key
            data.extend_from_slice(&(op.key.len() as u32).to_le_bytes());
            data.extend_from_slice(&op.key);

            // Write value length and value (for applicable operations)
            match op.op_type {
                WriteType::Put | WriteType::Merge | WriteType::DeleteRange => {
                    data.extend_from_slice(&(op.value.len() as u32).to_le_bytes());
                    data.extend_from_slice(&op.value);
                }
                WriteType::Delete | WriteType::SingleDelete => {
                    // No value for delete operations
                }
            }
        }

        data
    }

    /// Create a write batch from data representation
    pub fn from_data(data: &[u8]) -> Result<Self, Status> {
        if data.len() < 4 {
            return Err(Status::corruption(Some("Invalid write batch data".to_string())));
        }

        let mut batch = WriteBatch::new();
        let mut pos = 0;

        // Read count
        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        pos += 4;

        // Read operations
        for _ in 0..count {
            if pos >= data.len() {
                return Err(Status::corruption(Some("Truncated write batch".to_string())));
            }

            // Read operation type
            let op_type = match data[pos] {
                0 => WriteType::Put,
                1 => WriteType::Delete,
                2 => WriteType::SingleDelete,
                3 => WriteType::DeleteRange,
                4 => WriteType::Merge,
                _ => return Err(Status::corruption(Some("Invalid operation type".to_string()))),
            };
            pos += 1;

            // Read column family ID
            if pos + 4 > data.len() {
                return Err(Status::corruption(Some("Truncated write batch".to_string())));
            }
            let cf_id = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
            pos += 4;

            // Read key
            if pos + 4 > data.len() {
                return Err(Status::corruption(Some("Truncated write batch".to_string())));
            }
            let key_len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
            pos += 4;

            if pos + key_len > data.len() {
                return Err(Status::corruption(Some("Truncated write batch".to_string())));
            }
            let key = data[pos..pos + key_len].to_vec();
            pos += key_len;

            // Read value (for applicable operations)
            let value = match op_type {
                WriteType::Put | WriteType::Merge | WriteType::DeleteRange => {
                    if pos + 4 > data.len() {
                        return Err(Status::corruption(Some("Truncated write batch".to_string())));
                    }
                    let value_len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                    pos += 4;

                    if pos + value_len > data.len() {
                        return Err(Status::corruption(Some("Truncated write batch".to_string())));
                    }
                    let value = data[pos..pos + value_len].to_vec();
                    pos += value_len;
                    value
                }
                WriteType::Delete | WriteType::SingleDelete => Vec::new(),
            };

            let op = WriteOperation {
                op_type,
                cf_id,
                key,
                value,
            };

            batch.size += op.size();
            batch.count += 1;
            batch.operations.push_back(op);
        }

        Ok(batch)
    }
}

impl Default for WriteBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WriteBatch {
    fn clone(&self) -> Self {
        Self {
            operations: self.operations.clone(),
            size: self.size,
            count: self.count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_batch_basic() {
        let mut batch = WriteBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.count(), 0);
        assert_eq!(batch.size(), 0);

        // Add operations
        batch.put(b"key1", b"value1");
        batch.delete(b"key2");
        batch.merge(b"key3", b"value3");

        assert!(!batch.is_empty());
        assert_eq!(batch.count(), 3);
        assert!(batch.has_put());
        assert!(batch.has_delete());
        assert!(batch.has_merge());
    }

    #[test]
    fn test_write_batch_operations() {
        let mut batch = WriteBatch::new();
        let cf = ColumnFamilyHandle::default_cf();

        batch.put_cf(&cf, b"key1", b"value1");
        batch.delete_cf(&cf, b"key2");
        batch.single_delete_cf(&cf, b"key3");
        batch.delete_range_cf(&cf, b"start", b"end");
        batch.merge_cf(&cf, b"key4", b"merge_value");

        assert_eq!(batch.count(), 5);

        let ops: Vec<_> = batch.iter().collect();
        assert_eq!(ops[0].op_type, WriteType::Put);
        assert_eq!(ops[1].op_type, WriteType::Delete);
        assert_eq!(ops[2].op_type, WriteType::SingleDelete);
        assert_eq!(ops[3].op_type, WriteType::DeleteRange);
        assert_eq!(ops[4].op_type, WriteType::Merge);
    }

    #[test]
    fn test_write_batch_clear() {
        let mut batch = WriteBatch::new();
        batch.put(b"key1", b"value1");
        batch.put(b"key2", b"value2");

        assert!(!batch.is_empty());
        assert_eq!(batch.count(), 2);

        batch.clear();
        assert!(batch.is_empty());
        assert_eq!(batch.count(), 0);
        assert_eq!(batch.size(), 0);
    }

    #[test]
    fn test_write_batch_append() {
        let mut batch1 = WriteBatch::new();
        batch1.put(b"key1", b"value1");

        let mut batch2 = WriteBatch::new();
        batch2.put(b"key2", b"value2");
        batch2.delete(b"key3");

        batch1.append(&batch2);

        assert_eq!(batch1.count(), 3);
        let ops: Vec<_> = batch1.iter().collect();
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn test_write_batch_pop() {
        let mut batch = WriteBatch::new();
        batch.put(b"key1", b"value1");
        batch.delete(b"key2");

        assert_eq!(batch.count(), 2);

        let op = batch.pop_front().unwrap();
        assert_eq!(op.op_type, WriteType::Put);
        assert_eq!(op.key, b"key1");
        assert_eq!(op.value, b"value1");

        assert_eq!(batch.count(), 1);

        let op = batch.pop_front().unwrap();
        assert_eq!(op.op_type, WriteType::Delete);
        assert_eq!(op.key, b"key2");

        assert_eq!(batch.count(), 0);
        assert!(batch.pop_front().is_none());
    }

    #[test]
    fn test_write_batch_serialization() {
        let mut batch = WriteBatch::new();
        batch.put(b"key1", b"value1");
        batch.delete(b"key2");
        batch.merge(b"key3", b"merge_value");

        let data = batch.data();
        assert!(!data.is_empty());

        let deserialized_batch = WriteBatch::from_data(&data).unwrap();
        assert_eq!(deserialized_batch.count(), batch.count());

        let original_ops: Vec<_> = batch.iter().collect();
        let deserialized_ops: Vec<_> = deserialized_batch.iter().collect();

        assert_eq!(original_ops.len(), deserialized_ops.len());
        for (orig, deser) in original_ops.iter().zip(deserialized_ops.iter()) {
            assert_eq!(orig.op_type, deser.op_type);
            assert_eq!(orig.cf_id, deser.cf_id);
            assert_eq!(orig.key, deser.key);
            assert_eq!(orig.value, deser.value);
        }
    }

    #[test]
    fn test_write_batch_from_invalid_data() {
        // Test with too small data
        let result = WriteBatch::from_data(&[1, 2]);
        assert!(result.is_err());

        // Test with truncated data
        let result = WriteBatch::from_data(&[1, 0, 0, 0]); // count = 1 but no operations
        assert!(result.is_err());
    }
}
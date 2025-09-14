//! Iterator interface for ToplingDB

use crate::slice::Slice;
use crate::status::Status;

/// Iterator trait for database iterations
pub trait Iterator: Send {
    /// Check if iterator is valid (positioned at a key-value pair)
    fn valid(&self) -> bool;

    /// Move to the first key in the source
    fn seek_to_first(&mut self);

    /// Move to the last key in the source
    fn seek_to_last(&mut self);

    /// Position at the first key that is at or past target
    fn seek(&mut self, target: &[u8]);

    /// Position at the first key that is at or before target
    fn seek_for_prev(&mut self, target: &[u8]);

    /// Move to the next key
    fn next(&mut self);

    /// Move to the previous key
    fn prev(&mut self);

    /// Get the current key
    /// Only valid when `valid()` returns true
    fn key(&self) -> &[u8];

    /// Get the current value
    /// Only valid when `valid()` returns true
    fn value(&self) -> &[u8];

    /// Get the status of the iterator
    fn status(&self) -> Status;

    /// Refresh the iterator
    /// This allows an iterator to pick up recent changes to the database
    fn refresh(&mut self) -> Status {
        Status::ok()
    }
}

/// Database iterator implementation
pub struct DBIterator {
    /// Current key
    current_key: Vec<u8>,
    /// Current value
    current_value: Vec<u8>,
    /// Is the iterator valid?
    is_valid: bool,
    /// Iterator status
    iter_status: Status,
}

impl DBIterator {
    /// Create a new empty iterator
    pub fn new_empty() -> Self {
        Self {
            current_key: Vec::new(),
            current_value: Vec::new(),
            is_valid: false,
            iter_status: Status::ok(),
        }
    }

    /// Create a new iterator with initial position
    pub fn new_with_data(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            current_key: key,
            current_value: value,
            is_valid: true,
            iter_status: Status::ok(),
        }
    }

    /// Set the current position
    pub fn set_position(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.current_key = key;
        self.current_value = value;
        self.is_valid = true;
    }

    /// Invalidate the iterator
    pub fn invalidate(&mut self) {
        self.is_valid = false;
        self.current_key.clear();
        self.current_value.clear();
    }

    /// Set error status
    pub fn set_status(&mut self, status: Status) {
        let is_ok = status.is_ok();
        self.iter_status = status;
        if !is_ok {
            self.is_valid = false;
        }
    }
}

impl Iterator for DBIterator {
    fn valid(&self) -> bool {
        self.is_valid
    }

    fn seek_to_first(&mut self) {
        // TODO: Implement actual seeking to first
        self.invalidate();
    }

    fn seek_to_last(&mut self) {
        // TODO: Implement actual seeking to last
        self.invalidate();
    }

    fn seek(&mut self, _target: &[u8]) {
        // TODO: Implement actual seeking
        self.invalidate();
    }

    fn seek_for_prev(&mut self, _target: &[u8]) {
        // TODO: Implement actual seeking backwards
        self.invalidate();
    }

    fn next(&mut self) {
        // TODO: Implement actual next
        self.invalidate();
    }

    fn prev(&mut self) {
        // TODO: Implement actual previous
        self.invalidate();
    }

    fn key(&self) -> &[u8] {
        &self.current_key
    }

    fn value(&self) -> &[u8] {
        &self.current_value
    }

    fn status(&self) -> Status {
        self.iter_status.clone()
    }
}

/// Merging iterator that merges multiple child iterators
pub struct MergingIterator {
    /// Child iterators
    children: Vec<Box<dyn Iterator>>,
    /// Current direction (true for forward, false for backward)
    direction: bool,
    /// Current iterator index
    current_index: Option<usize>,
    /// Iterator status
    iter_status: Status,
}

impl MergingIterator {
    /// Create a new merging iterator
    pub fn new(children: Vec<Box<dyn Iterator>>) -> Self {
        Self {
            children,
            direction: true,
            current_index: None,
            iter_status: Status::ok(),
        }
    }

    /// Find the smallest key among all valid iterators
    fn find_smallest(&mut self) {
        let mut smallest_index = None;
        let mut smallest_key: Option<&[u8]> = None;

        for (i, iter) in self.children.iter().enumerate() {
            if iter.valid() {
                let key = iter.key();
                if smallest_key.is_none() || key < smallest_key.unwrap() {
                    smallest_key = Some(key);
                    smallest_index = Some(i);
                }
            }
        }

        self.current_index = smallest_index;
    }

    /// Find the largest key among all valid iterators
    fn find_largest(&mut self) {
        let mut largest_index = None;
        let mut largest_key: Option<&[u8]> = None;

        for (i, iter) in self.children.iter().enumerate() {
            if iter.valid() {
                let key = iter.key();
                if largest_key.is_none() || key > largest_key.unwrap() {
                    largest_key = Some(key);
                    largest_index = Some(i);
                }
            }
        }

        self.current_index = largest_index;
    }
}

impl Iterator for MergingIterator {
    fn valid(&self) -> bool {
        self.current_index.is_some()
            && self.children.get(self.current_index.unwrap())
                .map_or(false, |iter| iter.valid())
    }

    fn seek_to_first(&mut self) {
        for iter in &mut self.children {
            iter.seek_to_first();
        }
        self.direction = true;
        self.find_smallest();
    }

    fn seek_to_last(&mut self) {
        for iter in &mut self.children {
            iter.seek_to_last();
        }
        self.direction = false;
        self.find_largest();
    }

    fn seek(&mut self, target: &[u8]) {
        for iter in &mut self.children {
            iter.seek(target);
        }
        self.direction = true;
        self.find_smallest();
    }

    fn seek_for_prev(&mut self, target: &[u8]) {
        for iter in &mut self.children {
            iter.seek_for_prev(target);
        }
        self.direction = false;
        self.find_largest();
    }

    fn next(&mut self) {
        if !self.valid() {
            return;
        }

        if self.direction {
            // Moving forward
            if let Some(current_idx) = self.current_index {
                self.children[current_idx].next();
            }
            self.find_smallest();
        } else {
            // Was moving backward, now moving forward
            self.direction = true;
            if let Some(current_idx) = self.current_index {
                let current_key = self.children[current_idx].key().to_vec();
                self.seek(&current_key);
                if self.valid() && self.key() == current_key {
                    self.next();
                }
            }
        }
    }

    fn prev(&mut self) {
        if !self.valid() {
            return;
        }

        if !self.direction {
            // Moving backward
            if let Some(current_idx) = self.current_index {
                self.children[current_idx].prev();
            }
            self.find_largest();
        } else {
            // Was moving forward, now moving backward
            self.direction = false;
            if let Some(current_idx) = self.current_index {
                let current_key = self.children[current_idx].key().to_vec();
                self.seek_for_prev(&current_key);
                if self.valid() && self.key() == current_key {
                    self.prev();
                }
            }
        }
    }

    fn key(&self) -> &[u8] {
        if let Some(current_idx) = self.current_index {
            self.children[current_idx].key()
        } else {
            &[]
        }
    }

    fn value(&self) -> &[u8] {
        if let Some(current_idx) = self.current_index {
            self.children[current_idx].value()
        } else {
            &[]
        }
    }

    fn status(&self) -> Status {
        // Return the first error status we find
        for iter in &self.children {
            let status = iter.status();
            if !status.is_ok() {
                return status;
            }
        }
        Status::ok()
    }
}

/// Empty iterator that is always invalid
pub struct EmptyIterator {
    status: Status,
}

impl EmptyIterator {
    /// Create a new empty iterator
    pub fn new() -> Self {
        Self {
            status: Status::ok(),
        }
    }

    /// Create a new empty iterator with error status
    pub fn new_with_status(status: Status) -> Self {
        Self { status }
    }
}

impl Default for EmptyIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for EmptyIterator {
    fn valid(&self) -> bool {
        false
    }

    fn seek_to_first(&mut self) {
        // No-op
    }

    fn seek_to_last(&mut self) {
        // No-op
    }

    fn seek(&mut self, _target: &[u8]) {
        // No-op
    }

    fn seek_for_prev(&mut self, _target: &[u8]) {
        // No-op
    }

    fn next(&mut self) {
        // No-op
    }

    fn prev(&mut self) {
        // No-op
    }

    fn key(&self) -> &[u8] {
        &[]
    }

    fn value(&self) -> &[u8] {
        &[]
    }

    fn status(&self) -> Status {
        self.status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_iterator() {
        let mut iter = EmptyIterator::new();
        assert!(!iter.valid());
        assert!(iter.status().is_ok());

        iter.seek_to_first();
        assert!(!iter.valid());

        iter.next();
        assert!(!iter.valid());
    }

    #[test]
    fn test_db_iterator() {
        let mut iter = DBIterator::new_empty();
        assert!(!iter.valid());

        // Set position
        iter.set_position(b"key1".to_vec(), b"value1".to_vec());
        assert!(iter.valid());
        assert_eq!(iter.key(), b"key1");
        assert_eq!(iter.value(), b"value1");

        // Invalidate
        iter.invalidate();
        assert!(!iter.valid());
    }

    #[test]
    fn test_merging_iterator_empty() {
        let mut iter = MergingIterator::new(vec![]);
        assert!(!iter.valid());

        iter.seek_to_first();
        assert!(!iter.valid());
    }

    #[test]
    fn test_iterator_status() {
        let mut iter = DBIterator::new_empty();
        assert!(iter.status().is_ok());

        let error_status = Status::io_error(Some("Test error".to_string()));
        iter.set_status(error_status.clone());

        assert!(!iter.status().is_ok());
        assert!(!iter.valid());
    }
}
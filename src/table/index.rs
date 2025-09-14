//! Index implementation for SST files

use crate::iterator::{Iterator, EmptyIterator};
use crate::status::Status;

/// Index block for SST files
pub struct IndexBlock {
    /// Index entries
    entries: Vec<IndexEntry>,
}

/// Single index entry
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Last key in the block
    pub key: Vec<u8>,
    /// Offset of the block
    pub offset: u64,
    /// Size of the block
    pub size: u64,
}

impl IndexBlock {
    /// Create a new index block
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an index entry
    pub fn add_entry(&mut self, key: Vec<u8>, offset: u64, size: u64) {
        self.entries.push(IndexEntry { key, offset, size });
    }

    /// Find block containing the key
    pub fn find_block(&self, key: &[u8]) -> Option<&IndexEntry> {
        self.entries.iter()
            .find(|entry| entry.key.as_slice() >= key)
    }

    /// Create an iterator for the index
    pub fn new_iterator(&self) -> Box<dyn Iterator> {
        Box::new(EmptyIterator::new())
    }

    /// Get number of entries
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for IndexBlock {
    fn default() -> Self {
        Self::new()
    }
}

/// Index iterator
pub struct IndexIterator {
    /// Reference to entries
    entries: Vec<IndexEntry>,
    /// Current position
    position: Option<usize>,
    /// Current key
    current_key: Vec<u8>,
    /// Current value (offset:size)
    current_value: Vec<u8>,
}

impl IndexIterator {
    /// Create a new index iterator
    pub fn new(entries: Vec<IndexEntry>) -> Self {
        Self {
            entries,
            position: None,
            current_key: Vec::new(),
            current_value: Vec::new(),
        }
    }
}

impl Iterator for IndexIterator {
    fn valid(&self) -> bool {
        self.position.is_some()
    }

    fn seek_to_first(&mut self) {
        if !self.entries.is_empty() {
            self.position = Some(0);
            self.update_current();
        } else {
            self.position = None;
        }
    }

    fn seek_to_last(&mut self) {
        if !self.entries.is_empty() {
            self.position = Some(self.entries.len() - 1);
            self.update_current();
        } else {
            self.position = None;
        }
    }

    fn seek(&mut self, target: &[u8]) {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.key.as_slice() >= target {
                self.position = Some(i);
                self.update_current();
                return;
            }
        }
        self.position = None;
    }

    fn seek_for_prev(&mut self, target: &[u8]) {
        let mut found_pos = None;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.key.as_slice() <= target {
                found_pos = Some(i);
            } else {
                break;
            }
        }
        self.position = found_pos;
        if self.position.is_some() {
            self.update_current();
        }
    }

    fn next(&mut self) {
        if let Some(pos) = self.position {
            if pos + 1 < self.entries.len() {
                self.position = Some(pos + 1);
                self.update_current();
            } else {
                self.position = None;
            }
        }
    }

    fn prev(&mut self) {
        if let Some(pos) = self.position {
            if pos > 0 {
                self.position = Some(pos - 1);
                self.update_current();
            } else {
                self.position = None;
            }
        }
    }

    fn key(&self) -> &[u8] {
        &self.current_key
    }

    fn value(&self) -> &[u8] {
        &self.current_value
    }

    fn status(&self) -> Status {
        Status::ok()
    }
}

impl IndexIterator {
    fn update_current(&mut self) {
        if let Some(pos) = self.position {
            if let Some(entry) = self.entries.get(pos) {
                self.current_key = entry.key.clone();
                self.current_value = format!("{}:{}", entry.offset, entry.size).into_bytes();
            }
        }
    }
}
//! Block implementation for SST files

use crate::iterator::{Iterator, EmptyIterator};
use crate::status::Status;

/// A block of data in an SST file
pub struct Block {
    /// Block data
    data: Vec<u8>,
    /// Number of entries
    num_entries: u32,
    /// Restart points
    restart_points: Vec<u32>,
}

impl Block {
    /// Create a new block from data
    pub fn new(data: Vec<u8>) -> Result<Self, Status> {
        Ok(Self {
            data,
            num_entries: 0,
            restart_points: Vec::new(),
        })
    }

    /// Get value for a key
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Status> {
        Ok(None)
    }

    /// Create an iterator for this block
    pub fn new_iterator(&self) -> Box<dyn Iterator> {
        Box::new(EmptyIterator::new())
    }

    /// Get number of entries
    pub fn num_entries(&self) -> u32 {
        self.num_entries
    }

    /// Get block size
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Block iterator
pub struct BlockIterator {
    /// Reference to block data
    block_data: Vec<u8>,
    /// Current position
    position: usize,
    /// Is valid
    valid: bool,
    /// Current key
    current_key: Vec<u8>,
    /// Current value
    current_value: Vec<u8>,
}

impl BlockIterator {
    /// Create a new block iterator
    pub fn new(block_data: Vec<u8>) -> Self {
        Self {
            block_data,
            position: 0,
            valid: false,
            current_key: Vec::new(),
            current_value: Vec::new(),
        }
    }
}

impl Iterator for BlockIterator {
    fn valid(&self) -> bool {
        self.valid
    }

    fn seek_to_first(&mut self) {
        self.position = 0;
        self.valid = false;
    }

    fn seek_to_last(&mut self) {
        self.valid = false;
    }

    fn seek(&mut self, _target: &[u8]) {
        self.valid = false;
    }

    fn seek_for_prev(&mut self, _target: &[u8]) {
        self.valid = false;
    }

    fn next(&mut self) {
        self.valid = false;
    }

    fn prev(&mut self) {
        self.valid = false;
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
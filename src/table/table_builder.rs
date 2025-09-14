//! SST table builder implementation

use crate::options::Options;
use crate::slice::Slice;
use crate::status::Status;
use std::path::Path;

/// Builder for creating SST files
pub struct TableBuilder {
    /// Output file path
    file_path: String,
    /// Number of entries added
    num_entries: u64,
    /// Current data size
    data_size: u64,
}

impl TableBuilder {
    /// Create a new table builder
    pub fn new(options: &Options, file_path: &Path) -> Result<Self, Status> {
        Ok(Self {
            file_path: file_path.to_string_lossy().to_string(),
            num_entries: 0,
            data_size: 0,
        })
    }

    /// Add a key-value pair
    pub fn add(&mut self, key: &[u8], value: &[u8]) -> Result<(), Status> {
        self.num_entries += 1;
        self.data_size += key.len() as u64 + value.len() as u64;
        Ok(())
    }

    /// Finish building and close the file
    pub fn finish(self) -> Result<(), Status> {
        Ok(())
    }

    /// Get number of entries
    pub fn num_entries(&self) -> u64 {
        self.num_entries
    }

    /// Get file size
    pub fn file_size(&self) -> u64 {
        self.data_size
    }

    /// Check if table is empty
    pub fn empty(&self) -> bool {
        self.num_entries == 0
    }
}
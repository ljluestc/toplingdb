//! SST (Sorted String Table) implementation

pub mod table_builder;
pub mod table_reader;
pub mod block;
pub mod index;

use std::collections::HashMap;

/// Table properties
#[derive(Debug, Clone, Default)]
pub struct TableProperties {
    /// Number of entries
    pub num_entries: u64,
    /// Total key size
    pub key_size: u64,
    /// Total value size
    pub value_size: u64,
    /// Total raw key size
    pub raw_key_size: u64,
    /// Total raw value size
    pub raw_value_size: u64,
    /// File size
    pub file_size: u64,
    /// Filter size
    pub filter_size: u64,
    /// Index size
    pub index_size: u64,
    /// Top-level index size
    pub top_level_index_size: u64,
    /// Compression name
    pub compression_name: String,
    /// Column family name
    pub column_family_name: String,
    /// User defined properties
    pub user_defined_properties: HashMap<String, String>,
}

/// Table interface
pub trait Table: Send + Sync {
    /// Get value for a key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, crate::status::Status>;

    /// Create an iterator
    fn new_iterator(&self) -> Box<dyn crate::iterator::Iterator>;

    /// Get approximate offset of a key
    fn approximate_offset_of(&self, key: &[u8]) -> u64;

    /// Get properties
    fn properties(&self) -> TableProperties;
}

// Placeholder implementations
pub struct SSTable;
impl SSTable {
    pub fn open(_path: &std::path::Path) -> Result<Self, crate::status::Status> {
        Ok(Self)
    }
}

impl Table for SSTable {
    fn get(&self, _key: &[u8]) -> Result<Option<Vec<u8>>, crate::status::Status> {
        Ok(None)
    }

    fn new_iterator(&self) -> Box<dyn crate::iterator::Iterator> {
        Box::new(crate::iterator::EmptyIterator::new())
    }

    fn approximate_offset_of(&self, _key: &[u8]) -> u64 {
        0
    }

    fn properties(&self) -> TableProperties {
        TableProperties::default()
    }
}
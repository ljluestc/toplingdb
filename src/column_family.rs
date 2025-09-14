//! Column family management

use std::sync::Arc;
use crate::options::ColumnFamilyOptions;

/// Default column family name
pub const DEFAULT_CF_NAME: &str = "default";

/// Column family handle
#[derive(Debug, Clone)]
pub struct ColumnFamilyHandle {
    /// Column family name
    name: String,
    /// Column family ID
    id: u32,
    /// Column family options
    options: ColumnFamilyOptions,
}

impl ColumnFamilyHandle {
    /// Create a new column family handle
    pub fn new(name: String, id: u32, options: ColumnFamilyOptions) -> Self {
        Self { name, id, options }
    }

    /// Create the default column family handle
    pub fn default_cf() -> Self {
        Self::new(
            DEFAULT_CF_NAME.to_string(),
            0,
            ColumnFamilyOptions::default(),
        )
    }

    /// Get the column family name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the column family ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the column family options
    pub fn options(&self) -> &ColumnFamilyOptions {
        &self.options
    }

    /// Get mutable column family options
    pub fn options_mut(&mut self) -> &mut ColumnFamilyOptions {
        &mut self.options
    }

    /// Check if this is the default column family
    pub fn is_default(&self) -> bool {
        self.name == DEFAULT_CF_NAME
    }
}

impl PartialEq for ColumnFamilyHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ColumnFamilyHandle {}

impl std::hash::Hash for ColumnFamilyHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Column family descriptor used when opening database
#[derive(Debug, Clone)]
pub struct ColumnFamilyDescriptor {
    /// Column family name
    pub name: String,
    /// Column family options
    pub options: ColumnFamilyOptions,
}

impl ColumnFamilyDescriptor {
    /// Create a new column family descriptor
    pub fn new<S: Into<String>>(name: S, options: ColumnFamilyOptions) -> Self {
        Self {
            name: name.into(),
            options,
        }
    }

    /// Create a descriptor for the default column family
    pub fn default_cf() -> Self {
        Self::new(DEFAULT_CF_NAME, ColumnFamilyOptions::default())
    }
}

/// Column family metadata
#[derive(Debug, Clone)]
pub struct ColumnFamilyMetaData {
    /// Column family name
    pub name: String,
    /// Column family ID
    pub id: u32,
    /// Number of files
    pub file_count: u64,
    /// Total size in bytes
    pub size: u64,
    /// Number of entries
    pub num_entries: u64,
    /// Number of deletions
    pub num_deletions: u64,
}

impl ColumnFamilyMetaData {
    /// Create new metadata
    pub fn new(name: String, id: u32) -> Self {
        Self {
            name,
            id,
            file_count: 0,
            size: 0,
            num_entries: 0,
            num_deletions: 0,
        }
    }

    /// Update entry count
    pub fn add_entries(&mut self, count: u64) {
        self.num_entries += count;
    }

    /// Update deletion count
    pub fn add_deletions(&mut self, count: u64) {
        self.num_deletions += count;
    }

    /// Update size
    pub fn add_size(&mut self, size: u64) {
        self.size += size;
    }

    /// Add file
    pub fn add_file(&mut self, size: u64) {
        self.file_count += 1;
        self.size += size;
    }

    /// Remove file
    pub fn remove_file(&mut self, size: u64) {
        if self.file_count > 0 {
            self.file_count -= 1;
        }
        if self.size >= size {
            self.size -= size;
        } else {
            self.size = 0;
        }
    }
}

/// Column family interface
pub struct ColumnFamily {
    /// Handle for this column family
    handle: Arc<ColumnFamilyHandle>,
    /// Metadata for this column family
    metadata: std::sync::RwLock<ColumnFamilyMetaData>,
}

impl ColumnFamily {
    /// Create a new column family
    pub fn new(handle: Arc<ColumnFamilyHandle>) -> Self {
        let metadata = ColumnFamilyMetaData::new(handle.name().to_string(), handle.id());

        Self {
            handle,
            metadata: std::sync::RwLock::new(metadata),
        }
    }

    /// Get the handle for this column family
    pub fn handle(&self) -> &Arc<ColumnFamilyHandle> {
        &self.handle
    }

    /// Get the name
    pub fn name(&self) -> &str {
        self.handle.name()
    }

    /// Get the ID
    pub fn id(&self) -> u32 {
        self.handle.id()
    }

    /// Get the options
    pub fn options(&self) -> &ColumnFamilyOptions {
        self.handle.options()
    }

    /// Get metadata snapshot
    pub fn metadata(&self) -> ColumnFamilyMetaData {
        self.metadata.read().unwrap().clone()
    }

    /// Update metadata
    pub fn update_metadata<F>(&self, f: F)
    where
        F: FnOnce(&mut ColumnFamilyMetaData),
    {
        let mut metadata = self.metadata.write().unwrap();
        f(&mut metadata);
    }

    /// Get file count
    pub fn file_count(&self) -> u64 {
        self.metadata.read().unwrap().file_count
    }

    /// Get total size
    pub fn size(&self) -> u64 {
        self.metadata.read().unwrap().size
    }

    /// Get number of entries
    pub fn num_entries(&self) -> u64 {
        self.metadata.read().unwrap().num_entries
    }

    /// Get number of deletions
    pub fn num_deletions(&self) -> u64 {
        self.metadata.read().unwrap().num_deletions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_family_handle() {
        let options = ColumnFamilyOptions::default();
        let cf = ColumnFamilyHandle::new("test_cf".to_string(), 1, options);

        assert_eq!(cf.name(), "test_cf");
        assert_eq!(cf.id(), 1);
        assert!(!cf.is_default());

        let default_cf = ColumnFamilyHandle::default_cf();
        assert_eq!(default_cf.name(), DEFAULT_CF_NAME);
        assert_eq!(default_cf.id(), 0);
        assert!(default_cf.is_default());
    }

    #[test]
    fn test_column_family_equality() {
        let options1 = ColumnFamilyOptions::default();
        let options2 = ColumnFamilyOptions::default();

        let cf1 = ColumnFamilyHandle::new("test".to_string(), 1, options1);
        let cf2 = ColumnFamilyHandle::new("test".to_string(), 1, options2);
        let cf3 = ColumnFamilyHandle::new("other".to_string(), 2, ColumnFamilyOptions::default());

        assert_eq!(cf1, cf2); // Same ID
        assert_ne!(cf1, cf3); // Different ID
    }

    #[test]
    fn test_column_family_descriptor() {
        let options = ColumnFamilyOptions::default();
        let desc = ColumnFamilyDescriptor::new("test_cf", options);

        assert_eq!(desc.name, "test_cf");

        let default_desc = ColumnFamilyDescriptor::default_cf();
        assert_eq!(default_desc.name, DEFAULT_CF_NAME);
    }

    #[test]
    fn test_column_family_metadata() {
        let mut metadata = ColumnFamilyMetaData::new("test".to_string(), 1);

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.id, 1);
        assert_eq!(metadata.file_count, 0);
        assert_eq!(metadata.size, 0);
        assert_eq!(metadata.num_entries, 0);
        assert_eq!(metadata.num_deletions, 0);

        metadata.add_entries(100);
        metadata.add_deletions(10);
        metadata.add_size(1024);
        metadata.add_file(512);

        assert_eq!(metadata.num_entries, 100);
        assert_eq!(metadata.num_deletions, 10);
        assert_eq!(metadata.size, 1536); // 1024 + 512
        assert_eq!(metadata.file_count, 1);

        metadata.remove_file(512);
        assert_eq!(metadata.file_count, 0);
        assert_eq!(metadata.size, 1024);
    }

    #[test]
    fn test_column_family() {
        let handle = Arc::new(ColumnFamilyHandle::new(
            "test_cf".to_string(),
            1,
            ColumnFamilyOptions::default(),
        ));

        let cf = ColumnFamily::new(handle.clone());

        assert_eq!(cf.name(), "test_cf");
        assert_eq!(cf.id(), 1);
        assert_eq!(cf.file_count(), 0);
        assert_eq!(cf.size(), 0);
        assert_eq!(cf.num_entries(), 0);
        assert_eq!(cf.num_deletions(), 0);

        // Update metadata
        cf.update_metadata(|meta| {
            meta.add_entries(50);
            meta.add_file(1024);
        });

        assert_eq!(cf.num_entries(), 50);
        assert_eq!(cf.file_count(), 1);
        assert_eq!(cf.size(), 1024);

        let metadata = cf.metadata();
        assert_eq!(metadata.name, "test_cf");
        assert_eq!(metadata.id, 1);
        assert_eq!(metadata.num_entries, 50);
        assert_eq!(metadata.file_count, 1);
        assert_eq!(metadata.size, 1024);
    }
}
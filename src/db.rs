//! Core database implementation

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::options::{Options, ReadOptions, WriteOptions, FlushOptions};
use crate::status::{Status, StatusCode};
use crate::slice::{Slice, OwnedSlice};
use crate::iterator::{Iterator, DBIterator};
use crate::write_batch::WriteBatch;
use crate::snapshot::Snapshot;
use crate::column_family::{ColumnFamily, ColumnFamilyHandle, DEFAULT_CF_NAME};
use crate::memtable::MemTable;
use crate::env::Env;

/// Main database interface
pub struct DB {
    /// Database options
    options: Options,
    /// Database path
    path: std::path::PathBuf,
    /// Sequence number generator
    sequence_number: AtomicU64,
    /// Column families
    column_families: RwLock<HashMap<String, Arc<ColumnFamilyHandle>>>,
    /// Default column family
    default_cf: Arc<ColumnFamilyHandle>,
    /// Current memtable
    current_memtable: RwLock<Arc<MemTable>>,
    /// Immutable memtables
    immutable_memtables: RwLock<Vec<Arc<MemTable>>>,
    /// Environment
    env: Arc<dyn Env>,
    /// Write mutex to ensure atomic writes
    write_mutex: Mutex<()>,
    /// Database state
    state: RwLock<DBState>,
}

/// Database state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DBState {
    Open,
    Closing,
    Closed,
}

impl DB {
    /// Open a database with the specified options
    pub fn open<P: AsRef<Path>>(options: &Options, path: P) -> Result<Arc<Self>, Status> {
        let path = path.as_ref().to_path_buf();

        // Create database directory if it doesn't exist
        if options.db_options.create_if_missing {
            std::fs::create_dir_all(&path).map_err(|e| {
                Status::io_error(Some(format!("Failed to create database directory: {}", e)))
            })?;

            // Create a marker file to indicate database exists
            let marker_path = path.join("IDENTITY");
            std::fs::write(&marker_path, "toplingdb\n").map_err(|e| {
                Status::io_error(Some(format!("Failed to create database marker file: {}", e)))
            })?;
        }

        // Check if database exists
        if !path.exists() && !options.db_options.create_if_missing {
            return Err(Status::not_found(Some("Database does not exist".to_string())));
        }

        if path.exists() && options.db_options.error_if_exists {
            return Err(Status::invalid_argument(Some("Database already exists".to_string())));
        }

        // Create environment
        let env = options.db_options.env
            .clone()
            .unwrap_or_else(|| Arc::new(crate::env::DefaultEnv::new()));

        // Create default column family handle
        let default_cf = Arc::new(ColumnFamilyHandle::new(
            DEFAULT_CF_NAME.to_string(),
            0,
            options.cf_options.clone(),
        ));

        // Initialize column families map
        let mut column_families = HashMap::new();
        column_families.insert(DEFAULT_CF_NAME.to_string(), default_cf.clone());

        // Create initial memtable
        let memtable = Arc::new(MemTable::new(options.cf_options.clone()));

        let db = Arc::new(Self {
            options: options.clone(),
            path,
            sequence_number: AtomicU64::new(1),
            column_families: RwLock::new(column_families),
            default_cf,
            current_memtable: RwLock::new(memtable),
            immutable_memtables: RwLock::new(Vec::new()),
            env,
            write_mutex: Mutex::new(()),
            state: RwLock::new(DBState::Open),
        });

        Ok(db)
    }

    /// Put a key-value pair
    pub fn put(&self, options: &WriteOptions, key: &[u8], value: &[u8]) -> Result<(), Status> {
        self.put_cf(options, &self.default_cf, key, value)
    }

    /// Put a key-value pair in a specific column family
    pub fn put_cf(
        &self,
        options: &WriteOptions,
        cf: &ColumnFamilyHandle,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Status> {
        let mut write_batch = WriteBatch::new();
        write_batch.put_cf(cf, key, value);
        self.write(options, &write_batch)
    }

    /// Get a value for a key
    pub fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<Option<Vec<u8>>, Status> {
        self.get_cf(options, &self.default_cf, key)
    }

    /// Get a value for a key from a specific column family
    pub fn get_cf(
        &self,
        options: &ReadOptions,
        cf: &ColumnFamilyHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, Status> {
        self.check_open()?;

        // First check the current memtable
        {
            let current_memtable = self.current_memtable.read().unwrap();
            if let Some(value) = current_memtable.get(key)? {
                return Ok(Some(value));
            }
        }

        // Then check immutable memtables
        {
            let immutable_memtables = self.immutable_memtables.read().unwrap();
            for memtable in immutable_memtables.iter().rev() {
                if let Some(value) = memtable.get(key)? {
                    return Ok(Some(value));
                }
            }
        }

        // TODO: Check SST files
        Ok(None)
    }

    /// Delete a key
    pub fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<(), Status> {
        self.delete_cf(options, &self.default_cf, key)
    }

    /// Delete a key from a specific column family
    pub fn delete_cf(
        &self,
        options: &WriteOptions,
        cf: &ColumnFamilyHandle,
        key: &[u8],
    ) -> Result<(), Status> {
        let mut write_batch = WriteBatch::new();
        write_batch.delete_cf(cf, key);
        self.write(options, &write_batch)
    }

    /// Write a batch of operations
    pub fn write(&self, options: &WriteOptions, batch: &WriteBatch) -> Result<(), Status> {
        self.check_open()?;

        // Acquire write lock to ensure atomic writes
        let _write_guard = self.write_mutex.lock().unwrap();

        // Check if we need to create a new memtable
        self.maybe_switch_memtable()?;

        // Get sequence numbers for this batch
        let sequence_start = self.sequence_number.load(Ordering::SeqCst);
        let sequence_count = batch.count() as u64;
        self.sequence_number.store(sequence_start + sequence_count, Ordering::SeqCst);

        // Apply the batch to the memtable
        {
            let current_memtable = self.current_memtable.read().unwrap();
            current_memtable.insert_batch(batch, sequence_start)?;
        }

        // TODO: Write to WAL if not disabled
        if !options.disablewal {
            // Write to WAL
        }

        // TODO: Sync if requested
        if options.sync {
            // Sync WAL
        }

        Ok(())
    }

    /// Create an iterator
    pub fn new_iterator(&self, options: &ReadOptions) -> DBIterator {
        self.new_iterator_cf(options, &self.default_cf)
    }

    /// Create an iterator for a specific column family
    pub fn new_iterator_cf(&self, options: &ReadOptions, cf: &ColumnFamilyHandle) -> DBIterator {
        // TODO: Implement proper iterator with SST file support
        DBIterator::new_empty()
    }

    /// Create a snapshot
    pub fn get_snapshot(&self) -> Snapshot {
        let sequence_number = self.sequence_number.load(Ordering::SeqCst);
        Snapshot::new(sequence_number)
    }

    /// Release a snapshot
    pub fn release_snapshot(&self, _snapshot: &Snapshot) {
        // TODO: Implement snapshot cleanup
    }

    /// Flush memtable to disk
    pub fn flush(&self, options: &FlushOptions) -> Result<(), Status> {
        self.flush_cf(options, &self.default_cf)
    }

    /// Flush memtable to disk for a specific column family
    pub fn flush_cf(&self, options: &FlushOptions, cf: &ColumnFamilyHandle) -> Result<(), Status> {
        self.check_open()?;

        // Switch to a new memtable
        self.switch_memtable()?;

        // TODO: Flush immutable memtables to SST files
        if options.wait {
            // Wait for flush to complete
        }

        Ok(())
    }

    /// Compact a range of keys
    pub fn compact_range<S: AsRef<[u8]>>(
        &self,
        options: Option<&crate::options::CompactionOptions>,
        start: Option<S>,
        end: Option<S>,
    ) -> Result<(), Status> {
        self.compact_range_cf(options, &self.default_cf, start, end)
    }

    /// Compact a range of keys in a specific column family
    pub fn compact_range_cf<S: AsRef<[u8]>>(
        &self,
        _options: Option<&crate::options::CompactionOptions>,
        _cf: &ColumnFamilyHandle,
        _start: Option<S>,
        _end: Option<S>,
    ) -> Result<(), Status> {
        self.check_open()?;

        // TODO: Implement compaction
        Ok(())
    }

    /// Get database statistics
    pub fn get_property(&self, property: &str) -> Option<String> {
        self.get_property_cf(&self.default_cf, property)
    }

    /// Get database statistics for a specific column family
    pub fn get_property_cf(&self, cf: &ColumnFamilyHandle, property: &str) -> Option<String> {
        match property {
            "rocksdb.num-entries-active-mem-table" => {
                let memtable = self.current_memtable.read().unwrap();
                Some(memtable.num_entries().to_string())
            }
            "rocksdb.size-all-mem-tables" => {
                let current_size = {
                    let memtable = self.current_memtable.read().unwrap();
                    memtable.memory_usage()
                };
                let immutable_size = {
                    let immutable_memtables = self.immutable_memtables.read().unwrap();
                    immutable_memtables.iter().map(|m| m.memory_usage()).sum::<usize>()
                };
                Some((current_size + immutable_size).to_string())
            }
            "rocksdb.cur-size-active-mem-table" => {
                let memtable = self.current_memtable.read().unwrap();
                Some(memtable.memory_usage().to_string())
            }
            "rocksdb.num-immutable-mem-table" => {
                let immutable_memtables = self.immutable_memtables.read().unwrap();
                Some(immutable_memtables.len().to_string())
            }
            _ => None,
        }
    }

    /// Create a column family
    pub fn create_column_family(
        &self,
        cf_options: &crate::options::ColumnFamilyOptions,
        cf_name: &str,
    ) -> Result<Arc<ColumnFamilyHandle>, Status> {
        self.check_open()?;

        let mut column_families = self.column_families.write().unwrap();

        if column_families.contains_key(cf_name) {
            return Err(Status::invalid_argument(Some(format!(
                "Column family '{}' already exists",
                cf_name
            ))));
        }

        let cf_id = column_families.len() as u32;
        let cf_handle = Arc::new(ColumnFamilyHandle::new(
            cf_name.to_string(),
            cf_id,
            cf_options.clone(),
        ));

        column_families.insert(cf_name.to_string(), cf_handle.clone());

        Ok(cf_handle)
    }

    /// Drop a column family
    pub fn drop_column_family(&self, cf: &ColumnFamilyHandle) -> Result<(), Status> {
        self.check_open()?;

        if cf.name() == DEFAULT_CF_NAME {
            return Err(Status::invalid_argument(Some(
                "Cannot drop default column family".to_string()
            )));
        }

        let mut column_families = self.column_families.write().unwrap();
        column_families.remove(cf.name());

        Ok(())
    }

    /// Get all column family names
    pub fn list_column_families(&self) -> Vec<String> {
        let column_families = self.column_families.read().unwrap();
        column_families.keys().cloned().collect()
    }

    /// Close the database
    pub fn close(&self) -> Result<(), Status> {
        let mut state = self.state.write().unwrap();
        *state = DBState::Closing;

        // TODO: Wait for background operations to complete
        // TODO: Flush remaining data
        // TODO: Close files

        *state = DBState::Closed;
        Ok(())
    }

    // Private helper methods

    /// Check if database is open
    fn check_open(&self) -> Result<(), Status> {
        let state = self.state.read().unwrap();
        match *state {
            DBState::Open => Ok(()),
            DBState::Closing => Err(Status::new(
                StatusCode::ShutdownInProgress,
                Some("Database is closing".to_string()),
            )),
            DBState::Closed => Err(Status::new(
                StatusCode::ShutdownInProgress,
                Some("Database is closed".to_string()),
            )),
        }
    }

    /// Check if we need to switch to a new memtable
    fn maybe_switch_memtable(&self) -> Result<(), Status> {
        let current_memtable = self.current_memtable.read().unwrap();
        if current_memtable.memory_usage() >= self.options.cf_options.write_buffer_size {
            drop(current_memtable);
            self.switch_memtable()?;
        }
        Ok(())
    }

    /// Switch to a new memtable
    fn switch_memtable(&self) -> Result<(), Status> {
        let new_memtable = Arc::new(MemTable::new(self.options.cf_options.clone()));

        // Move current memtable to immutable list
        {
            let mut current_memtable = self.current_memtable.write().unwrap();
            let old_memtable = std::mem::replace(&mut *current_memtable, new_memtable);

            let mut immutable_memtables = self.immutable_memtables.write().unwrap();
            immutable_memtables.push(old_memtable);

            // Limit the number of immutable memtables
            let max_immutable = self.options.cf_options.max_write_buffer_number as usize;
            if immutable_memtables.len() > max_immutable {
                // TODO: Trigger flush
            }
        }

        Ok(())
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (Arc<DB>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_db");

        let mut options = Options::new();
        options.create_if_missing(true);

        let db = DB::open(&options, &path).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_db_open() {
        let (db, _temp_dir) = create_test_db();
        assert!(!db.path.as_os_str().is_empty());
    }

    #[test]
    fn test_put_get() {
        let (db, _temp_dir) = create_test_db();

        // Test basic put/get
        let write_opts = WriteOptions::default();
        let read_opts = ReadOptions::default();

        db.put(&write_opts, b"key1", b"value1").unwrap();

        let result = db.get(&read_opts, b"key1").unwrap();
        assert_eq!(result, Some(b"value1".to_vec()));

        // Test non-existent key
        let result = db.get(&read_opts, b"nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete() {
        let (db, _temp_dir) = create_test_db();

        let write_opts = WriteOptions::default();
        let read_opts = ReadOptions::default();

        // Put a key-value pair
        db.put(&write_opts, b"key1", b"value1").unwrap();
        let result = db.get(&read_opts, b"key1").unwrap();
        assert_eq!(result, Some(b"value1".to_vec()));

        // Delete the key
        db.delete(&write_opts, b"key1").unwrap();
        let result = db.get(&read_opts, b"key1").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_write_batch() {
        let (db, _temp_dir) = create_test_db();

        let write_opts = WriteOptions::default();
        let read_opts = ReadOptions::default();

        // Create a write batch
        let mut batch = WriteBatch::new();
        batch.put(b"key1", b"value1");
        batch.put(b"key2", b"value2");
        batch.delete(b"key3");

        // Write the batch
        db.write(&write_opts, &batch).unwrap();

        // Verify the results
        assert_eq!(db.get(&read_opts, b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(&read_opts, b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(db.get(&read_opts, b"key3").unwrap(), None);
    }

    #[test]
    fn test_snapshot() {
        let (db, _temp_dir) = create_test_db();

        let write_opts = WriteOptions::default();

        // Put initial data
        db.put(&write_opts, b"key1", b"value1").unwrap();

        // Create snapshot
        let snapshot = db.get_snapshot();

        // Modify data after snapshot
        db.put(&write_opts, b"key1", b"value2").unwrap();

        // TODO: Verify snapshot isolation when implemented
        db.release_snapshot(&snapshot);
    }

    #[test]
    fn test_column_families() {
        let (db, _temp_dir) = create_test_db();

        let cf_options = crate::options::ColumnFamilyOptions::default();

        // Create a new column family
        let cf = db.create_column_family(&cf_options, "test_cf").unwrap();
        assert_eq!(cf.name(), "test_cf");

        // List column families
        let cf_names = db.list_column_families();
        assert!(cf_names.contains(&"default".to_string()));
        assert!(cf_names.contains(&"test_cf".to_string()));

        // Use the column family
        let write_opts = WriteOptions::default();
        let read_opts = ReadOptions::default();

        db.put_cf(&write_opts, &cf, b"cf_key", b"cf_value").unwrap();
        let result = db.get_cf(&read_opts, &cf, b"cf_key").unwrap();
        assert_eq!(result, Some(b"cf_value".to_vec()));

        // Drop the column family
        db.drop_column_family(&cf).unwrap();
    }

    #[test]
    fn test_properties() {
        let (db, _temp_dir) = create_test_db();

        // Test getting properties
        let prop = db.get_property("rocksdb.num-entries-active-mem-table");
        assert!(prop.is_some());

        let prop = db.get_property("rocksdb.cur-size-active-mem-table");
        assert!(prop.is_some());
    }

    #[test]
    fn test_flush() {
        let (db, _temp_dir) = create_test_db();

        let write_opts = WriteOptions::default();
        let flush_opts = FlushOptions::default();

        // Add some data
        db.put(&write_opts, b"key1", b"value1").unwrap();
        db.put(&write_opts, b"key2", b"value2").unwrap();

        // Flush to disk
        db.flush(&flush_opts).unwrap();

        // Data should still be accessible
        let read_opts = ReadOptions::default();
        assert_eq!(db.get(&read_opts, b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(&read_opts, b"key2").unwrap(), Some(b"value2".to_vec()));
    }
}
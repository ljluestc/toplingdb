//! Integration tests for ToplingDB

use tempfile::TempDir;
use toplingdb::{DB, Options, ReadOptions, WriteOptions, WriteBatch};

#[test]
fn test_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();

    // Test put and get
    db.put(&write_opts, b"key1", b"value1").unwrap();
    assert_eq!(db.get(&read_opts, b"key1").unwrap(), Some(b"value1".to_vec()));

    // Test delete
    db.delete(&write_opts, b"key1").unwrap();
    assert_eq!(db.get(&read_opts, b"key1").unwrap(), None);
}

#[test]
fn test_write_batch() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

    let mut batch = WriteBatch::new();
    batch.put(b"key1", b"value1");
    batch.put(b"key2", b"value2");
    batch.delete(b"key3");

    let write_opts = WriteOptions::default();
    db.write(&write_opts, &batch).unwrap();

    let read_opts = ReadOptions::default();
    assert_eq!(db.get(&read_opts, b"key1").unwrap(), Some(b"value1".to_vec()));
    assert_eq!(db.get(&read_opts, b"key2").unwrap(), Some(b"value2".to_vec()));
    assert_eq!(db.get(&read_opts, b"key3").unwrap(), None);
}

#[test]
fn test_large_values() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();

    // Test with large value
    let large_value = vec![b'x'; 100_000];
    db.put(&write_opts, b"large_key", &large_value).unwrap();

    let retrieved = db.get(&read_opts, b"large_key").unwrap().unwrap();
    assert_eq!(retrieved, large_value);
}

#[test]
fn test_multiple_keys() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();

    // Insert many keys
    for i in 0..1000 {
        let key = format!("key{:04}", i);
        let value = format!("value{:04}", i);
        db.put(&write_opts, key.as_bytes(), value.as_bytes()).unwrap();
    }

    // Verify all keys
    for i in 0..1000 {
        let key = format!("key{:04}", i);
        let expected_value = format!("value{:04}", i);
        let actual_value = db.get(&read_opts, key.as_bytes()).unwrap().unwrap();
        assert_eq!(actual_value, expected_value.as_bytes());
    }
}

#[test]
fn test_snapshots() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

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
fn test_database_properties() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

    // Test getting properties
    let prop = db.get_property("rocksdb.num-entries-active-mem-table");
    assert!(prop.is_some());

    let prop = db.get_property("rocksdb.cur-size-active-mem-table");
    assert!(prop.is_some());
}

#[test]
fn test_column_families() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

    let cf_options = toplingdb::options::ColumnFamilyOptions::default();

    // Create a new column family
    let cf = db.create_column_family(&cf_options, "test_cf").unwrap();

    // Use the column family
    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();

    db.put_cf(&write_opts, &cf, b"cf_key", b"cf_value").unwrap();
    let result = db.get_cf(&read_opts, &cf, b"cf_key").unwrap();
    assert_eq!(result, Some(b"cf_value".to_vec()));

    // List column families
    let cf_names = db.list_column_families();
    assert!(cf_names.contains(&"default".to_string()));
    assert!(cf_names.contains(&"test_cf".to_string()));

    // Drop the column family
    db.drop_column_family(&cf).unwrap();
}

#[test]
fn test_flush() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    let mut options = Options::default();
    options.create_if_missing(true);

    let db = DB::open(&options, &path).unwrap();

    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();
    let flush_opts = toplingdb::options::FlushOptions::default();

    // Add data
    for i in 0..100 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        db.put(&write_opts, key.as_bytes(), value.as_bytes()).unwrap();
    }

    // Flush
    db.flush(&flush_opts).unwrap();

    // Verify data is still accessible
    for i in 0..100 {
        let key = format!("key{}", i);
        let expected_value = format!("value{}", i);
        let actual_value = db.get(&read_opts, key.as_bytes()).unwrap().unwrap();
        assert_eq!(actual_value, expected_value.as_bytes());
    }
}

#[test]
fn test_error_conditions() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_db");

    // Test opening non-existent database without create_if_missing
    let options = Options::default(); // create_if_missing = false
    let result = DB::open(&options, &path);
    assert!(result.is_err());

    // Test opening with error_if_exists when database exists
    let mut options = Options::default();
    options.create_if_missing(true);
    let _db1 = DB::open(&options, &path).unwrap();

    options.error_if_exists(true);
    let result = DB::open(&options, &path);
    assert!(result.is_err());
}
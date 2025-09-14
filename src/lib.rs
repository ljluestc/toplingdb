//! ToplingDB - A high-performance embeddable persistent key-value store
//!
//! ToplingDB is a Rust implementation of RocksDB, providing high-performance
//! storage with advanced features like LSM trees, compression, and transactions.
//!
//! # Examples
//!
//! ```rust
//! use toplingdb::{DB, Options, WriteOptions, ReadOptions};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let path = "/tmp/my_db";
//! let mut opts = Options::default();
//! opts.create_if_missing(true);
//!
//! let db = DB::open(&opts, path)?;
//!
//! // Put a key-value pair
//! db.put(&WriteOptions::default(), b"key1", b"value1")?;
//!
//! // Get a value
//! let value = db.get(&ReadOptions::default(), b"key1")?;
//! assert_eq!(value, Some(b"value1".to_vec()));
//!
//! // Delete a key
//! db.delete(&WriteOptions::default(), b"key1")?;
//! # Ok(())
//! # }
//! ```

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rust_2018_idioms,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

// Core database functionality
pub mod db;
pub mod options;
pub mod status;
pub mod slice;
pub mod iterator;
pub mod write_batch;
pub mod snapshot;
pub mod transaction;
pub mod column_family;

// Storage layer
pub mod table;
pub mod memtable;
pub mod sst;
pub mod compaction;
pub mod env;
pub mod cache;

// Utilities
pub mod util;
pub mod filter;
pub mod merge;
pub mod stats;
pub mod trace;
pub mod compression;

// Internal modules
mod internal;

// FFI exports
#[cfg(feature = "python-bindings")]
pub mod python;

#[cfg(feature = "java-bindings")]
pub mod java;

// Re-exports for convenience
pub use db::DB;
pub use iterator::DBIterator;
pub use options::{
    Options, ReadOptions, WriteOptions, FlushOptions, CompactionOptions,
    ColumnFamilyOptions, DBOptions
};
pub use status::{Status, StatusCode};
pub use slice::Slice;
pub use iterator::Iterator;
pub use write_batch::WriteBatch;
pub use snapshot::Snapshot;
pub use column_family::ColumnFamily;

// Error types
pub use status::Error as ToplingError;
pub type Result<T> = std::result::Result<T, ToplingError>;

/// Version information
pub const VERSION_MAJOR: u32 = 9;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;
pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
    ".",
    env!("CARGO_PKG_VERSION_PATCH")
);

/// Build information
pub const BUILD_DATE: &str = match option_env!("BUILD_DATE") {
    Some(date) => date,
    None => "unknown",
};
pub const GIT_SHA: &str = match option_env!("GIT_SHA") {
    Some(sha) => sha,
    None => "unknown",
};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_basic_db_operations() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_db");

        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, &path).unwrap();

        // Test put and get
        db.put(&WriteOptions::default(), b"key1", b"value1").unwrap();
        let result = db.get(&ReadOptions::default(), b"key1").unwrap();
        assert_eq!(result, Some(b"value1".to_vec()));

        // Test delete
        db.delete(&WriteOptions::default(), b"key1").unwrap();
        let result = db.get(&ReadOptions::default(), b"key1").unwrap();
        assert_eq!(result, None);
    }
}
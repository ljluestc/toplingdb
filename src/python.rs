//! Python bindings for ToplingDB using PyO3

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python-bindings")]
use pyo3::types::PyBytes;
#[cfg(feature = "python-bindings")]
use std::sync::Arc;

#[cfg(feature = "python-bindings")]
use crate::{DB, Options, ReadOptions, WriteOptions, FlushOptions};

/// Python wrapper for ToplingDB
#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct PyToplingDB {
    db: Arc<DB>,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl PyToplingDB {
    /// Open a database
    #[staticmethod]
    fn open(path: &str, create_if_missing: Option<bool>) -> PyResult<Self> {
        let mut options = Options::default();
        options.create_if_missing(create_if_missing.unwrap_or(false));

        let db = DB::open(&options, std::path::Path::new(path))
            .map_err(|e| PyException::new_err(e.to_string()))?;

        Ok(Self { db })
    }

    /// Put a key-value pair
    fn put(&self, key: &[u8], value: &[u8]) -> PyResult<()> {
        let write_options = WriteOptions::default();
        self.db.put(&write_options, key, value)
            .map_err(|e| PyException::new_err(e.to_string()))?;
        Ok(())
    }

    /// Get a value by key
    fn get<'py>(&self, py: Python<'py>, key: &[u8]) -> PyResult<Option<&'py PyBytes>> {
        let read_options = ReadOptions::default();
        match self.db.get(&read_options, key) {
            Ok(Some(value)) => Ok(Some(PyBytes::new(py, &value))),
            Ok(None) => Ok(None),
            Err(e) => Err(PyException::new_err(e.to_string())),
        }
    }

    /// Delete a key
    fn delete(&self, key: &[u8]) -> PyResult<()> {
        let write_options = WriteOptions::default();
        self.db.delete(&write_options, key)
            .map_err(|e| PyException::new_err(e.to_string()))?;
        Ok(())
    }

    /// Flush memtable to disk
    fn flush(&self) -> PyResult<()> {
        let flush_options = FlushOptions::default();
        self.db.flush(&flush_options)
            .map_err(|e| PyException::new_err(e.to_string()))?;
        Ok(())
    }

    /// Get database property
    fn get_property(&self, property: &str) -> PyResult<Option<String>> {
        Ok(self.db.get_property(property))
    }

    /// Close the database
    fn close(&self) -> PyResult<()> {
        self.db.close()
            .map_err(|e| PyException::new_err(e.to_string()))?;
        Ok(())
    }

    /// Context manager entry
    fn __enter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Context manager exit
    fn __exit__(&self, _exc_type: &PyAny, _exc_value: &PyAny, _traceback: &PyAny) -> PyResult<()> {
        self.close()
    }
}

/// Python wrapper for WriteBatch
#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct PyWriteBatch {
    batch: crate::WriteBatch,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl PyWriteBatch {
    /// Create a new write batch
    #[new]
    fn new() -> Self {
        Self {
            batch: crate::WriteBatch::new(),
        }
    }

    /// Add a put operation
    fn put(&mut self, key: &[u8], value: &[u8]) {
        self.batch.put(key, value);
    }

    /// Add a delete operation
    fn delete(&mut self, key: &[u8]) {
        self.batch.delete(key);
    }

    /// Add a merge operation
    fn merge(&mut self, key: &[u8], value: &[u8]) {
        self.batch.merge(key, value);
    }

    /// Clear all operations
    fn clear(&mut self) {
        self.batch.clear();
    }

    /// Get the number of operations
    fn count(&self) -> usize {
        self.batch.count()
    }

    /// Check if batch is empty
    fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }
}

/// Python wrapper for database iterator
#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct PyIterator {
    iter: crate::iterator::DBIterator,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl PyIterator {
    /// Check if iterator is valid
    fn valid(&self) -> bool {
        self.iter.valid()
    }

    /// Seek to first key
    fn seek_to_first(&mut self) {
        self.iter.seek_to_first();
    }

    /// Seek to last key
    fn seek_to_last(&mut self) {
        self.iter.seek_to_last();
    }

    /// Seek to a specific key
    fn seek(&mut self, target: &[u8]) {
        self.iter.seek(target);
    }

    /// Move to next key
    fn next(&mut self) {
        self.iter.next();
    }

    /// Move to previous key
    fn prev(&mut self) {
        self.iter.prev();
    }

    /// Get current key
    fn key<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        Ok(PyBytes::new(py, self.iter.key()))
    }

    /// Get current value
    fn value<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        Ok(PyBytes::new(py, self.iter.value()))
    }

    /// Iterator protocol
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Iterator next
    fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<(&'py PyBytes, &'py PyBytes)>> {
        if !self.iter.valid() {
            return Ok(None);
        }

        let key = PyBytes::new(py, self.iter.key());
        let value = PyBytes::new(py, self.iter.value());
        self.iter.next();

        Ok(Some((key, value)))
    }
}

/// Python module initialization
#[cfg(feature = "python-bindings")]
#[pymodule]
fn toplingdb(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyToplingDB>()?;
    m.add_class::<PyWriteBatch>()?;
    m.add_class::<PyIterator>()?;

    // Add version information
    m.add("VERSION", crate::VERSION)?;
    m.add("VERSION_MAJOR", crate::VERSION_MAJOR)?;
    m.add("VERSION_MINOR", crate::VERSION_MINOR)?;
    m.add("VERSION_PATCH", crate::VERSION_PATCH)?;

    // Add module docstring
    m.add("__doc__", "ToplingDB - A high-performance embeddable persistent key-value store")?;

    Ok(())
}

// Provide stub implementations when python-bindings feature is not enabled
#[cfg(not(feature = "python-bindings"))]
pub fn init_python_module() {
    // Python bindings not available without the feature
}

// Python setup.py template
#[cfg(feature = "python-bindings")]
pub const PYTHON_SETUP_TEMPLATE: &str = r#"
from setuptools import setup
from pyo3_setuptools_rust import Pyo3RustExtension, build_rust

setup(
    name="toplingdb",
    version="9.1.0",
    description="ToplingDB Python bindings",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    author="ToplingDB Contributors",
    url="https://github.com/topling/toplingdb",
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "License :: OSI Approved :: GNU General Public License v2 (GPLv2)",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Rust",
        "Topic :: Database",
        "Topic :: Software Development :: Libraries",
    ],
    rust_extensions=[
        Pyo3RustExtension("toplingdb", "Cargo.toml", binding=Pyo3RustExtension.Pyo3),
    ],
    cmdclass={"build_rust": build_rust},
    zip_safe=False,
    python_requires=">=3.8",
)
"#;

#[cfg(not(feature = "python-bindings"))]
pub const PYTHON_SETUP_TEMPLATE: &str = r#"
# Python bindings not available. Enable the 'python-bindings' feature to use Python bindings.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_setup_template() {
        assert!(!PYTHON_SETUP_TEMPLATE.is_empty());
    }

    #[cfg(feature = "python-bindings")]
    #[test]
    fn test_python_write_batch() {
        let mut batch = PyWriteBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.count(), 0);

        batch.put(b"key1", b"value1");
        batch.delete(b"key2");
        batch.merge(b"key3", b"value3");

        assert!(!batch.is_empty());
        assert_eq!(batch.count(), 3);

        batch.clear();
        assert!(batch.is_empty());
        assert_eq!(batch.count(), 0);
    }
}
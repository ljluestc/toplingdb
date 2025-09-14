//! Environment interface for system interactions

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::status::{Status, StatusCode};

/// File handle for reading
pub trait RandomAccessFile: Send + Sync {
    /// Read data from the file at the specified offset
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Status>;

    /// Get the size of the file
    fn size(&self) -> Result<u64, Status>;
}

/// File handle for writing
pub trait WritableFile: Send + Sync {
    /// Append data to the file
    fn append(&mut self, data: &[u8]) -> Result<(), Status>;

    /// Close the file
    fn close(&mut self) -> Result<(), Status>;

    /// Flush the file to disk
    fn flush(&mut self) -> Result<(), Status>;

    /// Sync the file to disk
    fn sync(&mut self) -> Result<(), Status>;

    /// Truncate the file to the specified size
    fn truncate(&mut self, size: u64) -> Result<(), Status>;

    /// Get the file size
    fn size(&self) -> Result<u64, Status>;
}

/// File handle for sequential reading
pub trait SequentialFile: Send + Sync {
    /// Read data from the file
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Status>;

    /// Skip the specified number of bytes
    fn skip(&mut self, n: u64) -> Result<(), Status>;
}

/// Logger interface
pub trait Logger: Send + Sync + std::fmt::Debug {
    /// Log a message
    fn log(&self, level: LogLevel, message: &str);

    /// Log a formatted message
    fn logf(&self, level: LogLevel, fmt: &str, args: std::fmt::Arguments);

    /// Get the log level
    fn level(&self) -> LogLevel;

    /// Set the log level
    fn set_level(&mut self, level: LogLevel);
}

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Fatal = 4,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Fatal => write!(f, "FATAL"),
        }
    }
}

/// Environment interface
pub trait Env: Send + Sync + std::fmt::Debug {
    /// Create a new sequential file reader
    fn new_sequential_file(&self, path: &Path) -> Result<Box<dyn SequentialFile>, Status>;

    /// Create a new random access file reader
    fn new_random_access_file(&self, path: &Path) -> Result<Box<dyn RandomAccessFile>, Status>;

    /// Create a new writable file
    fn new_writable_file(&self, path: &Path) -> Result<Box<dyn WritableFile>, Status>;

    /// Create a new appendable file
    fn new_appendable_file(&self, path: &Path) -> Result<Box<dyn WritableFile>, Status>;

    /// Check if a file exists
    fn file_exists(&self, path: &Path) -> bool;

    /// Get file size
    fn get_file_size(&self, path: &Path) -> Result<u64, Status>;

    /// Delete a file
    fn delete_file(&self, path: &Path) -> Result<(), Status>;

    /// Create a directory
    fn create_dir(&self, path: &Path) -> Result<(), Status>;

    /// Create directories recursively
    fn create_dir_all(&self, path: &Path) -> Result<(), Status>;

    /// Delete a directory
    fn delete_dir(&self, path: &Path) -> Result<(), Status>;

    /// List directory contents
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Status>;

    /// Rename a file
    fn rename_file(&self, from: &Path, to: &Path) -> Result<(), Status>;

    /// Lock a file
    fn lock_file(&self, path: &Path) -> Result<Box<dyn FileLock>, Status>;

    /// Get current time in microseconds since epoch
    fn now_micros(&self) -> u64;

    /// Get current time in nanoseconds since epoch
    fn now_nanos(&self) -> u64;

    /// Sleep for the specified duration
    fn sleep(&self, duration: std::time::Duration);

    /// Get number of CPU cores
    fn get_cpu_count(&self) -> u32;

    /// Start a thread
    fn start_thread(&self, name: &str, f: Box<dyn FnOnce() + Send + 'static>);

    /// Schedule a function to run after a delay
    fn schedule(&self, delay: std::time::Duration, f: Box<dyn FnOnce() + Send>);

    /// Get environment variable
    fn get_env_var(&self, name: &str) -> Option<String>;

    /// Set environment variable
    fn set_env_var(&self, name: &str, value: &str);

    /// Get logger
    fn get_logger(&self) -> Option<Arc<dyn Logger>>;

    /// Set logger
    fn set_logger(&mut self, logger: Arc<dyn Logger>);
}

/// File lock interface
pub trait FileLock: Send + Sync {
    /// Release the lock
    fn unlock(&self) -> Result<(), Status>;
}

/// Standard file implementation for reading
pub struct StdRandomAccessFile {
    file: Arc<Mutex<File>>,
}

impl StdRandomAccessFile {
    pub fn new(path: &Path) -> Result<Self, Status> {
        let file = File::open(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to open file {}: {}", path.display(), e)))
        })?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }
}

impl RandomAccessFile for StdRandomAccessFile {
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Status> {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start(offset)).map_err(|e| {
            Status::io_error(Some(format!("Failed to seek: {}", e)))
        })?;

        file.read(buf).map_err(|e| {
            Status::io_error(Some(format!("Failed to read: {}", e)))
        })
    }

    fn size(&self) -> Result<u64, Status> {
        let file = self.file.lock().unwrap();
        let metadata = file.metadata().map_err(|e| {
            Status::io_error(Some(format!("Failed to get metadata: {}", e)))
        })?;
        Ok(metadata.len())
    }
}

/// Standard file implementation for writing
pub struct StdWritableFile {
    file: File,
    path: PathBuf,
}

impl StdWritableFile {
    pub fn new(path: &Path) -> Result<Self, Status> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| {
                Status::io_error(Some(format!("Failed to create file {}: {}", path.display(), e)))
            })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    pub fn append(path: &Path) -> Result<Self, Status> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| {
                Status::io_error(Some(format!("Failed to open file for append {}: {}", path.display(), e)))
            })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }
}

impl WritableFile for StdWritableFile {
    fn append(&mut self, data: &[u8]) -> Result<(), Status> {
        self.file.write_all(data).map_err(|e| {
            Status::io_error(Some(format!("Failed to write: {}", e)))
        })
    }

    fn close(&mut self) -> Result<(), Status> {
        self.sync()
    }

    fn flush(&mut self) -> Result<(), Status> {
        self.file.flush().map_err(|e| {
            Status::io_error(Some(format!("Failed to flush: {}", e)))
        })
    }

    fn sync(&mut self) -> Result<(), Status> {
        self.file.sync_all().map_err(|e| {
            Status::io_error(Some(format!("Failed to sync: {}", e)))
        })
    }

    fn truncate(&mut self, size: u64) -> Result<(), Status> {
        self.file.set_len(size).map_err(|e| {
            Status::io_error(Some(format!("Failed to truncate: {}", e)))
        })
    }

    fn size(&self) -> Result<u64, Status> {
        let metadata = self.file.metadata().map_err(|e| {
            Status::io_error(Some(format!("Failed to get file size: {}", e)))
        })?;
        Ok(metadata.len())
    }
}

/// Standard sequential file implementation
pub struct StdSequentialFile {
    file: File,
}

impl StdSequentialFile {
    pub fn new(path: &Path) -> Result<Self, Status> {
        let file = File::open(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to open file {}: {}", path.display(), e)))
        })?;

        Ok(Self { file })
    }
}

impl SequentialFile for StdSequentialFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Status> {
        self.file.read(buf).map_err(|e| {
            Status::io_error(Some(format!("Failed to read: {}", e)))
        })
    }

    fn skip(&mut self, n: u64) -> Result<(), Status> {
        self.file.seek(SeekFrom::Current(n as i64)).map_err(|e| {
            Status::io_error(Some(format!("Failed to skip: {}", e)))
        })?;
        Ok(())
    }
}

/// Standard file lock implementation
pub struct StdFileLock {
    path: PathBuf,
}

impl FileLock for StdFileLock {
    fn unlock(&self) -> Result<(), Status> {
        // TODO: Implement proper file locking
        Ok(())
    }
}

/// Standard logger implementation
#[derive(Debug)]
pub struct StdLogger {
    level: LogLevel,
}

impl StdLogger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

impl Logger for StdLogger {
    fn log(&self, level: LogLevel, message: &str) {
        if level >= self.level {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            println!("[{}] {}: {}", timestamp, level, message);
        }
    }

    fn logf(&self, level: LogLevel, fmt: &str, args: std::fmt::Arguments) {
        if level >= self.level {
            let message = format!("{}", args);
            self.log(level, &message);
        }
    }

    fn level(&self) -> LogLevel {
        self.level
    }

    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }
}

/// Default environment implementation
#[derive(Debug)]
pub struct DefaultEnv {
    logger: Option<Arc<dyn Logger>>,
    env_vars: Arc<Mutex<HashMap<String, String>>>,
}

impl DefaultEnv {
    pub fn new() -> Self {
        Self {
            logger: Some(Arc::new(StdLogger::new(LogLevel::Info))),
            env_vars: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for DefaultEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl Env for DefaultEnv {
    fn new_sequential_file(&self, path: &Path) -> Result<Box<dyn SequentialFile>, Status> {
        Ok(Box::new(StdSequentialFile::new(path)?))
    }

    fn new_random_access_file(&self, path: &Path) -> Result<Box<dyn RandomAccessFile>, Status> {
        Ok(Box::new(StdRandomAccessFile::new(path)?))
    }

    fn new_writable_file(&self, path: &Path) -> Result<Box<dyn WritableFile>, Status> {
        Ok(Box::new(StdWritableFile::new(path)?))
    }

    fn new_appendable_file(&self, path: &Path) -> Result<Box<dyn WritableFile>, Status> {
        Ok(Box::new(StdWritableFile::append(path)?))
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn get_file_size(&self, path: &Path) -> Result<u64, Status> {
        let metadata = std::fs::metadata(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to get file size for {}: {}", path.display(), e)))
        })?;
        Ok(metadata.len())
    }

    fn delete_file(&self, path: &Path) -> Result<(), Status> {
        std::fs::remove_file(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to delete file {}: {}", path.display(), e)))
        })
    }

    fn create_dir(&self, path: &Path) -> Result<(), Status> {
        std::fs::create_dir(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to create directory {}: {}", path.display(), e)))
        })
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), Status> {
        std::fs::create_dir_all(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to create directories {}: {}", path.display(), e)))
        })
    }

    fn delete_dir(&self, path: &Path) -> Result<(), Status> {
        std::fs::remove_dir_all(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to delete directory {}: {}", path.display(), e)))
        })
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Status> {
        let entries = std::fs::read_dir(path).map_err(|e| {
            Status::io_error(Some(format!("Failed to read directory {}: {}", path.display(), e)))
        })?;

        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                Status::io_error(Some(format!("Failed to read directory entry: {}", e)))
            })?;
            paths.push(entry.path());
        }

        Ok(paths)
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<(), Status> {
        std::fs::rename(from, to).map_err(|e| {
            Status::io_error(Some(format!("Failed to rename {} to {}: {}", from.display(), to.display(), e)))
        })
    }

    fn lock_file(&self, path: &Path) -> Result<Box<dyn FileLock>, Status> {
        // TODO: Implement proper file locking
        Ok(Box::new(StdFileLock {
            path: path.to_path_buf(),
        }))
    }

    fn now_micros(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64
    }

    fn now_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    fn sleep(&self, duration: std::time::Duration) {
        std::thread::sleep(duration);
    }

    fn get_cpu_count(&self) -> u32 {
        num_cpus::get() as u32
    }

    fn start_thread(&self, name: &str, f: Box<dyn FnOnce() + Send + 'static>) {
        std::thread::Builder::new()
            .name(name.to_string())
            .spawn(f)
            .expect("Failed to start thread");
    }

    fn schedule(&self, delay: std::time::Duration, f: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            f();
        });
    }

    fn get_env_var(&self, name: &str) -> Option<String> {
        // First check our local env vars, then system env vars
        if let Ok(env_vars) = self.env_vars.lock() {
            if let Some(value) = env_vars.get(name) {
                return Some(value.clone());
            }
        }
        std::env::var(name).ok()
    }

    fn set_env_var(&self, name: &str, value: &str) {
        if let Ok(mut env_vars) = self.env_vars.lock() {
            env_vars.insert(name.to_string(), value.to_string());
        }
    }

    fn get_logger(&self) -> Option<Arc<dyn Logger>> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = Some(logger);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_random_access_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let raf = StdRandomAccessFile::new(temp_file.path()).unwrap();

        let mut buf = vec![0u8; 5];
        let n = raf.read(0, &mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, b"hello");

        let n = raf.read(6, &mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, b"world");

        assert_eq!(raf.size().unwrap(), 11);
    }

    #[test]
    fn test_writable_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut wf = StdWritableFile::new(&file_path).unwrap();
        wf.append(b"hello").unwrap();
        wf.append(b" world").unwrap();
        wf.flush().unwrap();

        assert_eq!(wf.size().unwrap(), 11);

        // Read back the content
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_sequential_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let mut sf = StdSequentialFile::new(temp_file.path()).unwrap();

        let mut buf = vec![0u8; 5];
        let n = sf.read(&mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, b"hello");

        sf.skip(1).unwrap(); // skip space

        let n = sf.read(&mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, b"world");
    }

    #[test]
    fn test_default_env() {
        let env = DefaultEnv::new();

        // Test time functions
        let micros = env.now_micros();
        let nanos = env.now_nanos();
        assert!(micros > 0);
        assert!(nanos > 0);
        assert!(nanos > micros * 1000); // nanos should be larger

        // Test CPU count
        let cpu_count = env.get_cpu_count();
        assert!(cpu_count > 0);

        // Test environment variables
        env.set_env_var("TEST_VAR", "test_value");
        assert_eq!(env.get_env_var("TEST_VAR"), Some("test_value".to_string()));
        assert_eq!(env.get_env_var("NONEXISTENT_VAR"), None);
    }

    #[test]
    fn test_env_file_operations() {
        let env = DefaultEnv::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Test file existence
        assert!(!env.file_exists(&test_file));

        // Create and write to file
        let mut wf = env.new_writable_file(&test_file).unwrap();
        wf.append(b"test data").unwrap();
        wf.close().unwrap();

        assert!(env.file_exists(&test_file));
        assert_eq!(env.get_file_size(&test_file).unwrap(), 9);

        // Test directory operations
        let test_dir = temp_dir.path().join("subdir");
        env.create_dir(&test_dir).unwrap();
        assert!(test_dir.exists());

        let entries = env.list_dir(temp_dir.path()).unwrap();
        assert!(entries.len() >= 2); // At least test.txt and subdir

        // Test file renaming
        let new_name = temp_dir.path().join("renamed.txt");
        env.rename_file(&test_file, &new_name).unwrap();
        assert!(!env.file_exists(&test_file));
        assert!(env.file_exists(&new_name));

        // Test file deletion
        env.delete_file(&new_name).unwrap();
        assert!(!env.file_exists(&new_name));

        // Test directory deletion
        env.delete_dir(&test_dir).unwrap();
        assert!(!test_dir.exists());
    }

    #[test]
    fn test_logger() {
        let mut logger = StdLogger::new(LogLevel::Info);
        assert_eq!(logger.level(), LogLevel::Info);

        logger.set_level(LogLevel::Error);
        assert_eq!(logger.level(), LogLevel::Error);

        // These should not panic
        logger.log(LogLevel::Error, "Error message");
        logger.log(LogLevel::Fatal, "Fatal message");
        logger.log(LogLevel::Info, "This should not be printed"); // Below threshold
    }
}
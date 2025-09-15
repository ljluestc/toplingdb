//! SST table builder implementation

use crate::options::Options;
use crate::status::Status;
use crate::compression::{CompressionFactory, CompressionContext};
use crate::util::crc::crc32c;
// Use hardcoded values for now
const FOOTER_SIZE: usize = 48;
const BLOCK_TRAILER_SIZE: usize = 5;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;

/// Builder for creating SST files
pub struct TableBuilder {
    /// Output file
    file: BufWriter<File>,
    /// Number of entries added
    num_entries: u64,
    /// Current file position
    offset: u64,
    /// Current data block
    data_block: BlockBuilder,
    /// Index block
    index_block: BlockBuilder,
    /// Options
    options: Options,
    /// Last key added
    last_key: Vec<u8>,
    /// Pending index entry
    pending_index_entry: bool,
    /// Status
    status: Status,
    /// Block handles for data blocks
    data_block_handles: Vec<BlockHandle>,
}

/// Block builder for data and index blocks
pub struct BlockBuilder {
    /// Block data
    buffer: Vec<u8>,
    /// Restart points
    restarts: Vec<u32>,
    /// Number of entries since last restart
    counter: u32,
    /// Finished
    finished: bool,
    /// Block restart interval
    restart_interval: u32,
}

/// Block handle
#[derive(Clone, Debug)]
pub struct BlockHandle {
    /// Offset in file
    offset: u64,
    /// Size of block
    size: u64,
}

impl BlockHandle {
    pub fn new(offset: u64, size: u64) -> Self {
        Self { offset, size }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

impl BlockBuilder {
    pub fn new(restart_interval: u32) -> Self {
        let mut restarts = Vec::new();
        restarts.push(0); // First restart point is at offset 0

        Self {
            buffer: Vec::new(),
            restarts,
            counter: 0,
            finished: false,
            restart_interval,
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) {
        assert!(!self.finished, "Cannot add to finished block");
        assert!(self.counter <= self.restart_interval);

        let mut shared = 0;
        if self.counter < self.restart_interval {
            // Find shared prefix with previous key
            if !self.buffer.is_empty() {
                shared = self.find_shared_prefix(key);
            }
        } else {
            // Restart compression
            self.restarts.push(self.buffer.len() as u32);
            self.counter = 0;
        }

        let non_shared = key.len() - shared;

        // Add entry: shared_len(varint) non_shared_len(varint) value_len(varint) key_delta value
        self.put_varint32(shared as u32);
        self.put_varint32(non_shared as u32);
        self.put_varint32(value.len() as u32);

        // Add key delta and value
        self.buffer.extend_from_slice(&key[shared..]);
        self.buffer.extend_from_slice(value);

        self.counter += 1;
    }

    pub fn finish(&mut self) -> Vec<u8> {
        // Add restart array
        let restarts = self.restarts.clone();
        for restart in restarts {
            self.put_fixed32(restart);
        }
        self.put_fixed32(self.restarts.len() as u32);
        self.finished = true;
        self.buffer.clone()
    }

    pub fn current_size_estimate(&self) -> usize {
        self.buffer.len() + self.restarts.len() * 4 + 4
    }

    pub fn empty(&self) -> bool {
        self.buffer.is_empty()
    }

    fn find_shared_prefix(&self, key: &[u8]) -> usize {
        // For simplicity, return 0 (no compression)
        // In a real implementation, we'd compare with the last key
        0
    }

    fn put_varint32(&mut self, mut value: u32) {
        while value >= 0x80 {
            self.buffer.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.buffer.push(value as u8);
    }

    fn put_fixed32(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }
}

impl TableBuilder {
    /// Create a new table builder
    pub fn new(options: &Options, file_path: &Path) -> Result<Self, Status> {
        let file = File::create(file_path).map_err(|e| {
            Status::io_error(Some(format!("Failed to create SST file: {}", e)))
        })?;

        let file = BufWriter::new(file);

        Ok(Self {
            file,
            num_entries: 0,
            offset: 0,
            data_block: BlockBuilder::new(16), // Default restart interval
            index_block: BlockBuilder::new(1), // Index block uses restart interval of 1
            options: options.clone(),
            last_key: Vec::new(),
            pending_index_entry: false,
            status: Status::ok(),
            data_block_handles: Vec::new(),
        })
    }

    /// Add a key-value pair
    pub fn add(&mut self, key: &[u8], value: &[u8]) -> Result<(), Status> {
        if !self.status.is_ok() {
            return Err(self.status.clone());
        }

        if self.num_entries > 0 && key <= &self.last_key {
            return Err(Status::invalid_argument(Some("Keys must be added in order".to_string())));
        }

        if self.pending_index_entry {
            // Use short separator for index key
            let separator = self.find_short_separator(&self.last_key, key);
            let handle_encoding = self.encode_block_handle(&self.data_block_handles.last().unwrap());
            self.index_block.add(&separator, &handle_encoding);
            self.pending_index_entry = false;
        }

        self.last_key = key.to_vec();
        self.num_entries += 1;
        self.data_block.add(key, value);

        // Check if we need to flush the data block
        if self.data_block.current_size_estimate() >= self.options.table_options.block_size {
            self.flush()?;
        }

        Ok(())
    }

    /// Flush current data block
    pub fn flush(&mut self) -> Result<(), Status> {
        if self.data_block.empty() {
            return Ok(());
        }

        let raw_block = self.data_block.finish();
        let compressed_block = self.compress_block(&raw_block)?;

        let handle = self.write_block(&compressed_block)?;
        self.data_block_handles.push(handle);

        self.data_block = BlockBuilder::new(16);
        self.pending_index_entry = true;

        Ok(())
    }

    /// Finish building and close the file
    pub fn finish(mut self) -> Result<(), Status> {
        // Flush any remaining data
        self.flush()?;

        // Write index block
        if self.pending_index_entry && !self.data_block_handles.is_empty() {
            let handle_encoding = self.encode_block_handle(&self.data_block_handles.last().unwrap());
            self.index_block.add(&self.last_key, &handle_encoding);
        }

        let index_block_data = self.index_block.finish();
        let compressed_index = self.compress_block(&index_block_data)?;
        let index_handle = self.write_block(&compressed_index)?;

        // Write footer
        self.write_footer(&index_handle)?;

        self.file.flush().map_err(|e| {
            Status::io_error(Some(format!("Failed to flush file: {}", e)))
        })?;

        Ok(())
    }

    fn compress_block(&self, data: &[u8]) -> Result<Vec<u8>, Status> {
        let compression_type = self.options.table_options.compression;
        let compressor = CompressionFactory::create_compressor(compression_type);
        let context = CompressionContext {
            compression_type,
            level: -1,
            dict: None,
            samples: Vec::new(),
        };

        let compressed = compressor.compress(data, &context)?;

        // Add compression type and CRC
        let mut result = compressed;
        result.push(compression_type as u8);

        // Calculate and append CRC32C
        let crc = crc32c(&result);
        result.extend_from_slice(&crc.to_le_bytes());

        Ok(result)
    }

    fn write_block(&mut self, data: &[u8]) -> Result<BlockHandle, Status> {
        let handle = BlockHandle::new(self.offset, data.len() as u64);

        self.file.write_all(data).map_err(|e| {
            Status::io_error(Some(format!("Failed to write block: {}", e)))
        })?;

        self.offset += data.len() as u64;
        Ok(handle)
    }

    fn write_footer(&mut self, index_handle: &BlockHandle) -> Result<(), Status> {
        let mut footer = Vec::with_capacity(FOOTER_SIZE);

        // Write index block handle
        footer.extend_from_slice(&self.encode_block_handle(index_handle));

        // Pad to footer size - 8 bytes for magic number
        while footer.len() < FOOTER_SIZE - 8 {
            footer.push(0);
        }

        // Write magic number
        footer.extend_from_slice(&0xdb4775248b80fb57u64.to_le_bytes());

        self.file.write_all(&footer).map_err(|e| {
            Status::io_error(Some(format!("Failed to write footer: {}", e)))
        })?;

        self.offset += footer.len() as u64;
        Ok(())
    }

    fn encode_block_handle(&self, handle: &BlockHandle) -> Vec<u8> {
        let mut result = Vec::new();
        self.put_varint64(&mut result, handle.offset);
        self.put_varint64(&mut result, handle.size);
        result
    }

    fn put_varint64(&self, buf: &mut Vec<u8>, mut value: u64) {
        while value >= 0x80 {
            buf.push((value as u8) | 0x80);
            value >>= 7;
        }
        buf.push(value as u8);
    }

    fn find_short_separator(&self, start: &[u8], limit: &[u8]) -> Vec<u8> {
        // Simple implementation: just return start
        // A real implementation would find the shortest key >= start and < limit
        start.to_vec()
    }

    /// Get number of entries
    pub fn num_entries(&self) -> u64 {
        self.num_entries
    }

    /// Get file size
    pub fn file_size(&self) -> u64 {
        self.offset
    }

    /// Check if table is empty
    pub fn empty(&self) -> bool {
        self.num_entries == 0
    }
}
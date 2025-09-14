//! File format constants and utilities

/// File format constants
pub mod constants {
    /// Magic number for SST files
    pub const SST_MAGIC_NUMBER: u64 = 0xdb4775248b80fb57;

    /// Magic number for WAL files
    pub const WAL_MAGIC_NUMBER: u64 = 0x62bd5b3e7f6a6b8d;

    /// Block size for data blocks (default 4KB)
    pub const DEFAULT_BLOCK_SIZE: usize = 4 * 1024;

    /// Default restart interval for blocks
    pub const DEFAULT_RESTART_INTERVAL: usize = 16;

    /// Footer size in bytes
    pub const FOOTER_SIZE: usize = 48;

    /// Block trailer size (checksum + type)
    pub const BLOCK_TRAILER_SIZE: usize = 5;

    /// Maximum block size
    pub const MAX_BLOCK_SIZE: usize = 128 * 1024 * 1024; // 128MB

    /// Minimum block size
    pub const MIN_BLOCK_SIZE: usize = 1024; // 1KB
}

/// Block types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    /// Data block
    Data = 0,
    /// Filter block
    Filter = 1,
    /// Meta index block
    MetaIndex = 2,
    /// Index block
    Index = 3,
    /// Range deletion block
    RangeDeletion = 4,
    /// Compression dictionary
    CompressionDictionary = 5,
}

impl BlockType {
    /// Convert from byte
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(BlockType::Data),
            1 => Some(BlockType::Filter),
            2 => Some(BlockType::MetaIndex),
            3 => Some(BlockType::Index),
            4 => Some(BlockType::RangeDeletion),
            5 => Some(BlockType::CompressionDictionary),
            _ => None,
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// Compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// No compression
    None = 0,
    /// Snappy compression
    Snappy = 1,
    /// Zlib compression
    Zlib = 2,
    /// BZip2 compression
    BZip2 = 3,
    /// LZ4 compression
    LZ4 = 4,
    /// LZ4HC compression
    LZ4HC = 5,
    /// ZSTD compression
    ZSTD = 6,
}

impl CompressionType {
    /// Convert from byte
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(CompressionType::None),
            1 => Some(CompressionType::Snappy),
            2 => Some(CompressionType::Zlib),
            3 => Some(CompressionType::BZip2),
            4 => Some(CompressionType::LZ4),
            5 => Some(CompressionType::LZ4HC),
            6 => Some(CompressionType::ZSTD),
            _ => None,
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Get name as string
    pub fn name(&self) -> &'static str {
        match self {
            CompressionType::None => "none",
            CompressionType::Snappy => "snappy",
            CompressionType::Zlib => "zlib",
            CompressionType::BZip2 => "bzip2",
            CompressionType::LZ4 => "lz4",
            CompressionType::LZ4HC => "lz4hc",
            CompressionType::ZSTD => "zstd",
        }
    }

    /// Parse from string
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "none" => Some(CompressionType::None),
            "snappy" => Some(CompressionType::Snappy),
            "zlib" => Some(CompressionType::Zlib),
            "bzip2" => Some(CompressionType::BZip2),
            "lz4" => Some(CompressionType::LZ4),
            "lz4hc" => Some(CompressionType::LZ4HC),
            "zstd" => Some(CompressionType::ZSTD),
            _ => None,
        }
    }
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::Snappy
    }
}

/// File footer format
#[derive(Debug, Clone)]
pub struct Footer {
    /// Metaindex block handle
    pub metaindex_handle: BlockHandle,
    /// Index block handle
    pub index_handle: BlockHandle,
    /// Checksum type
    pub checksum_type: ChecksumType,
    /// Magic number
    pub magic_number: u64,
}

/// Block handle (offset and size)
#[derive(Debug, Clone, Copy)]
pub struct BlockHandle {
    /// Offset in file
    pub offset: u64,
    /// Size in bytes
    pub size: u64,
}

impl BlockHandle {
    /// Create new block handle
    pub fn new(offset: u64, size: u64) -> Self {
        Self { offset, size }
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        crate::util::coding::encode_varint_64(self.offset, &mut buf);
        crate::util::coding::encode_varint_64(self.size, &mut buf);
        buf
    }

    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<(Self, usize)> {
        let (offset, offset_bytes) = crate::util::coding::decode_varint_64(data)?;
        let (size, size_bytes) = crate::util::coding::decode_varint_64(&data[offset_bytes..])?;
        Some((Self { offset, size }, offset_bytes + size_bytes))
    }

    /// Check if handle is valid
    pub fn is_valid(&self) -> bool {
        self.size > 0
    }
}

impl Default for BlockHandle {
    fn default() -> Self {
        Self { offset: 0, size: 0 }
    }
}

/// Checksum types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumType {
    /// No checksum
    None = 0,
    /// CRC32C checksum
    CRC32C = 1,
    /// xxHash checksum
    XxHash = 2,
    /// xxHash64 checksum
    XxHash64 = 3,
}

impl ChecksumType {
    /// Convert from byte
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(ChecksumType::None),
            1 => Some(ChecksumType::CRC32C),
            2 => Some(ChecksumType::XxHash),
            3 => Some(ChecksumType::XxHash64),
            _ => None,
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

impl Default for ChecksumType {
    fn default() -> Self {
        ChecksumType::CRC32C
    }
}

/// Record types for WAL (Write-Ahead Log)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    /// Zero is reserved for preallocated files
    Zero = 0,
    /// Full record
    Full = 1,
    /// First fragment of a record
    First = 2,
    /// Middle fragment of a record
    Middle = 3,
    /// Last fragment of a record
    Last = 4,
}

impl RecordType {
    /// Convert from byte
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(RecordType::Zero),
            1 => Some(RecordType::Full),
            2 => Some(RecordType::First),
            3 => Some(RecordType::Middle),
            4 => Some(RecordType::Last),
            _ => None,
        }
    }

    /// Convert to byte
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// WAL record header size
pub const WAL_HEADER_SIZE: usize = 7; // 4 bytes checksum + 2 bytes length + 1 byte type

/// Calculate block padding
pub fn calculate_padding(current_size: usize, block_size: usize) -> usize {
    let remainder = current_size % block_size;
    if remainder == 0 {
        0
    } else {
        block_size - remainder
    }
}

/// Validate magic number
pub fn is_valid_sst_magic(magic: u64) -> bool {
    magic == constants::SST_MAGIC_NUMBER
}

/// Validate WAL magic number
pub fn is_valid_wal_magic(magic: u64) -> bool {
    magic == constants::WAL_MAGIC_NUMBER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_type() {
        assert_eq!(BlockType::Data.to_byte(), 0);
        assert_eq!(BlockType::from_byte(0), Some(BlockType::Data));
        assert_eq!(BlockType::from_byte(255), None);
    }

    #[test]
    fn test_compression_type() {
        assert_eq!(CompressionType::Snappy.to_byte(), 1);
        assert_eq!(CompressionType::from_byte(1), Some(CompressionType::Snappy));
        assert_eq!(CompressionType::from_name("snappy"), Some(CompressionType::Snappy));
        assert_eq!(CompressionType::from_name("invalid"), None);
        assert_eq!(CompressionType::Snappy.name(), "snappy");
    }

    #[test]
    fn test_block_handle() {
        let handle = BlockHandle::new(1000, 500);
        assert!(handle.is_valid());

        let encoded = handle.encode();
        let (decoded, bytes_read) = BlockHandle::decode(&encoded).unwrap();
        assert_eq!(handle.offset, decoded.offset);
        assert_eq!(handle.size, decoded.size);
        assert_eq!(encoded.len(), bytes_read);

        let invalid_handle = BlockHandle::default();
        assert!(!invalid_handle.is_valid());
    }

    #[test]
    fn test_checksum_type() {
        assert_eq!(ChecksumType::CRC32C.to_byte(), 1);
        assert_eq!(ChecksumType::from_byte(1), Some(ChecksumType::CRC32C));
        assert_eq!(ChecksumType::from_byte(255), None);
    }

    #[test]
    fn test_record_type() {
        assert_eq!(RecordType::Full.to_byte(), 1);
        assert_eq!(RecordType::from_byte(1), Some(RecordType::Full));
        assert_eq!(RecordType::from_byte(255), None);
    }

    #[test]
    fn test_calculate_padding() {
        assert_eq!(calculate_padding(0, 100), 0);
        assert_eq!(calculate_padding(50, 100), 50);
        assert_eq!(calculate_padding(100, 100), 0);
        assert_eq!(calculate_padding(150, 100), 50);
    }

    #[test]
    fn test_magic_numbers() {
        assert!(is_valid_sst_magic(constants::SST_MAGIC_NUMBER));
        assert!(!is_valid_sst_magic(0));

        assert!(is_valid_wal_magic(constants::WAL_MAGIC_NUMBER));
        assert!(!is_valid_wal_magic(0));
    }
}
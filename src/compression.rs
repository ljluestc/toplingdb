//! Compression algorithms and utilities for ToplingDB

use std::fmt;
use crate::status::{Status, StatusCode};

/// Compression types supported by ToplingDB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionType {
    /// No compression
    None,
    /// Snappy compression
    Snappy,
    /// Zlib compression
    Zlib,
    /// Bzip2 compression
    BZip2,
    /// LZ4 compression
    LZ4,
    /// LZ4HC (high compression) compression
    LZ4HC,
    /// XPRESS compression (Windows only)
    Xpress,
    /// ZSTD compression
    ZSTD,
    /// All compression types (for iteration)
    All,
}

impl fmt::Display for CompressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "NoCompression"),
            Self::Snappy => write!(f, "Snappy"),
            Self::Zlib => write!(f, "Zlib"),
            Self::BZip2 => write!(f, "BZip2"),
            Self::LZ4 => write!(f, "LZ4"),
            Self::LZ4HC => write!(f, "LZ4HC"),
            Self::Xpress => write!(f, "XPRESS"),
            Self::ZSTD => write!(f, "ZSTD"),
            Self::All => write!(f, "All"),
        }
    }
}

impl Default for CompressionType {
    fn default() -> Self {
        Self::Snappy
    }
}

/// Compression context information
#[derive(Debug, Clone)]
pub struct CompressionContext {
    /// Compression type
    pub compression_type: CompressionType,
    /// Compression level (-1 for default)
    pub level: i32,
    /// Dictionary for compression
    pub dict: Option<Vec<u8>>,
    /// Sample data for training
    pub samples: Vec<Vec<u8>>,
}

impl Default for CompressionContext {
    fn default() -> Self {
        Self {
            compression_type: CompressionType::None,
            level: -1,
            dict: None,
            samples: Vec::new(),
        }
    }
}

/// Uncompression context information
#[derive(Debug, Clone, Default)]
pub struct UncompressionContext {
    /// Compression type used
    pub compression_type: CompressionType,
    /// Dictionary for decompression
    pub dict: Option<Vec<u8>>,
}

/// Compression information
#[derive(Debug, Clone)]
pub struct CompressionInfo {
    /// Original size
    pub uncompressed_size: usize,
    /// Compressed size
    pub compressed_size: usize,
    /// Compression ratio
    pub ratio: f64,
    /// Time taken for compression
    pub compression_time_ns: u64,
}

impl CompressionInfo {
    /// Create new compression info
    pub fn new(uncompressed_size: usize, compressed_size: usize, compression_time_ns: u64) -> Self {
        let ratio = if uncompressed_size > 0 {
            compressed_size as f64 / uncompressed_size as f64
        } else {
            0.0
        };

        Self {
            uncompressed_size,
            compressed_size,
            ratio,
            compression_time_ns,
        }
    }

    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        self.ratio
    }

    /// Calculate space saved
    pub fn space_saved(&self) -> i64 {
        self.uncompressed_size as i64 - self.compressed_size as i64
    }

    /// Calculate space saved percentage
    pub fn space_saved_percent(&self) -> f64 {
        if self.uncompressed_size > 0 {
            (self.space_saved() as f64 / self.uncompressed_size as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Compression trait
pub trait Compressor: Send + Sync {
    /// Compress data
    fn compress(&self, data: &[u8], context: &CompressionContext) -> Result<Vec<u8>, Status>;

    /// Get maximum compressed length for given input size
    fn max_compressed_len(&self, input_len: usize) -> usize;

    /// Get compression type
    fn compression_type(&self) -> CompressionType;

    /// Get name
    fn name(&self) -> &str;
}

/// Decompression trait
pub trait Decompressor: Send + Sync {
    /// Decompress data
    fn decompress(&self, data: &[u8], context: &UncompressionContext) -> Result<Vec<u8>, Status>;

    /// Get compression type
    fn compression_type(&self) -> CompressionType;

    /// Get name
    fn name(&self) -> &str;
}

/// No compression implementation
#[derive(Debug, Default)]
pub struct NoCompressor;

impl Compressor for NoCompressor {
    fn compress(&self, data: &[u8], _context: &CompressionContext) -> Result<Vec<u8>, Status> {
        Ok(data.to_vec())
    }

    fn max_compressed_len(&self, input_len: usize) -> usize {
        input_len
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::None
    }

    fn name(&self) -> &str {
        "NoCompression"
    }
}

impl Decompressor for NoCompressor {
    fn decompress(&self, data: &[u8], _context: &UncompressionContext) -> Result<Vec<u8>, Status> {
        Ok(data.to_vec())
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::None
    }

    fn name(&self) -> &str {
        "NoCompression"
    }
}

/// Snappy compression implementation
#[cfg(feature = "compression")]
#[derive(Debug, Default)]
pub struct SnappyCompressor;

#[cfg(feature = "compression")]
impl Compressor for SnappyCompressor {
    fn compress(&self, data: &[u8], _context: &CompressionContext) -> Result<Vec<u8>, Status> {
        snap::raw::Encoder::new()
            .compress_vec(data)
            .map_err(|e| Status::new(StatusCode::InvalidArgument, Some(format!("Snappy compression failed: {}", e))))
    }

    fn max_compressed_len(&self, input_len: usize) -> usize {
        snap::raw::max_compress_len(input_len)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Snappy
    }

    fn name(&self) -> &str {
        "Snappy"
    }
}

#[cfg(feature = "compression")]
impl Decompressor for SnappyCompressor {
    fn decompress(&self, data: &[u8], _context: &UncompressionContext) -> Result<Vec<u8>, Status> {
        snap::raw::Decoder::new()
            .decompress_vec(data)
            .map_err(|e| Status::new(StatusCode::Corruption, Some(format!("Snappy decompression failed: {}", e))))
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Snappy
    }

    fn name(&self) -> &str {
        "Snappy"
    }
}

/// LZ4 compression implementation
#[cfg(feature = "compression")]
#[derive(Debug, Default)]
pub struct Lz4Compressor;

#[cfg(feature = "compression")]
impl Compressor for Lz4Compressor {
    fn compress(&self, data: &[u8], _context: &CompressionContext) -> Result<Vec<u8>, Status> {
        lz4::block::compress(data, None, true)
            .map_err(|e| Status::new(StatusCode::InvalidArgument, Some(format!("LZ4 compression failed: {}", e))))
    }

    fn max_compressed_len(&self, input_len: usize) -> usize {
        lz4::block::compress_bound(input_len).unwrap_or(input_len * 2)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::LZ4
    }

    fn name(&self) -> &str {
        "LZ4"
    }
}

#[cfg(feature = "compression")]
impl Decompressor for Lz4Compressor {
    fn decompress(&self, data: &[u8], _context: &UncompressionContext) -> Result<Vec<u8>, Status> {
        lz4::block::decompress(data, None)
            .map_err(|e| Status::new(StatusCode::Corruption, Some(format!("LZ4 decompression failed: {}", e))))
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::LZ4
    }

    fn name(&self) -> &str {
        "LZ4"
    }
}

/// ZSTD compression implementation
#[cfg(feature = "compression")]
#[derive(Debug, Default)]
pub struct ZstdCompressor;

#[cfg(feature = "compression")]
impl Compressor for ZstdCompressor {
    fn compress(&self, data: &[u8], context: &CompressionContext) -> Result<Vec<u8>, Status> {
        let level = if context.level == -1 { 3 } else { context.level };
        zstd::bulk::compress(data, level)
            .map_err(|e| Status::new(StatusCode::InvalidArgument, Some(format!("ZSTD compression failed: {}", e))))
    }

    fn max_compressed_len(&self, input_len: usize) -> usize {
        // ZSTD compress bound calculation
        // This is a conservative estimate based on ZSTD documentation
        let bound = input_len + (input_len >> 8) + 64;
        bound.max(128)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::ZSTD
    }

    fn name(&self) -> &str {
        "ZSTD"
    }
}

#[cfg(feature = "compression")]
impl Decompressor for ZstdCompressor {
    fn decompress(&self, data: &[u8], _context: &UncompressionContext) -> Result<Vec<u8>, Status> {
        zstd::bulk::decompress(data, 1024 * 1024) // 1MB limit
            .map_err(|e| Status::new(StatusCode::Corruption, Some(format!("ZSTD decompression failed: {}", e))))
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::ZSTD
    }

    fn name(&self) -> &str {
        "ZSTD"
    }
}

/// Compression factory for creating compressors/decompressors
pub struct CompressionFactory;

impl CompressionFactory {
    /// Create a compressor for the given compression type
    pub fn create_compressor(compression_type: CompressionType) -> Box<dyn Compressor> {
        match compression_type {
            CompressionType::None => Box::new(NoCompressor),
            #[cfg(feature = "compression")]
            CompressionType::Snappy => Box::new(SnappyCompressor),
            #[cfg(feature = "compression")]
            CompressionType::LZ4 => Box::new(Lz4Compressor),
            #[cfg(feature = "compression")]
            CompressionType::ZSTD => Box::new(ZstdCompressor),
            _ => Box::new(NoCompressor), // Fallback
        }
    }

    /// Create a decompressor for the given compression type
    pub fn create_decompressor(compression_type: CompressionType) -> Box<dyn Decompressor> {
        match compression_type {
            CompressionType::None => Box::new(NoCompressor),
            #[cfg(feature = "compression")]
            CompressionType::Snappy => Box::new(SnappyCompressor),
            #[cfg(feature = "compression")]
            CompressionType::LZ4 => Box::new(Lz4Compressor),
            #[cfg(feature = "compression")]
            CompressionType::ZSTD => Box::new(ZstdCompressor),
            _ => Box::new(NoCompressor), // Fallback
        }
    }

    /// Check if compression type is supported
    pub fn is_supported(compression_type: CompressionType) -> bool {
        match compression_type {
            CompressionType::None => true,
            #[cfg(feature = "compression")]
            CompressionType::Snappy | CompressionType::LZ4 | CompressionType::ZSTD => true,
            _ => false,
        }
    }

    /// Get all supported compression types
    pub fn supported_types() -> Vec<CompressionType> {
        let mut types = vec![CompressionType::None];

        #[cfg(feature = "compression")]
        {
            types.push(CompressionType::Snappy);
            types.push(CompressionType::LZ4);
            types.push(CompressionType::ZSTD);
        }

        types
    }
}

/// Compression utilities
pub struct CompressionUtil;

impl CompressionUtil {
    /// Compress data with timing
    pub fn compress_with_timing(
        data: &[u8],
        compression_type: CompressionType,
        context: &CompressionContext,
    ) -> Result<(Vec<u8>, CompressionInfo), Status> {
        let start = std::time::Instant::now();
        let compressor = CompressionFactory::create_compressor(compression_type);
        let compressed = compressor.compress(data, context)?;
        let duration = start.elapsed();

        let info = CompressionInfo::new(
            data.len(),
            compressed.len(),
            duration.as_nanos() as u64,
        );

        Ok((compressed, info))
    }

    /// Decompress data with timing
    pub fn decompress_with_timing(
        data: &[u8],
        compression_type: CompressionType,
        context: &UncompressionContext,
    ) -> Result<(Vec<u8>, u64), Status> {
        let start = std::time::Instant::now();
        let decompressor = CompressionFactory::create_decompressor(compression_type);
        let decompressed = decompressor.decompress(data, context)?;
        let duration = start.elapsed();

        Ok((decompressed, duration.as_nanos() as u64))
    }

    /// Benchmark compression algorithms
    pub fn benchmark_compression(data: &[u8]) -> Vec<(CompressionType, CompressionInfo)> {
        let mut results = Vec::new();
        let context = CompressionContext::default();

        for compression_type in CompressionFactory::supported_types() {
            if let Ok((_, info)) = Self::compress_with_timing(
                data,
                compression_type,
                &CompressionContext {
                    compression_type,
                    ..context.clone()
                },
            ) {
                results.push((compression_type, info));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_compression() {
        let data = b"hello world";
        let context = CompressionContext::default();
        let compressor = NoCompressor;

        let compressed = compressor.compress(data, &context).unwrap();
        assert_eq!(compressed, data);

        let decompressed = compressor.decompress(&compressed, &UncompressionContext::default()).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compression_factory() {
        let compressor = CompressionFactory::create_compressor(CompressionType::None);
        assert_eq!(compressor.compression_type(), CompressionType::None);

        let decompressor = CompressionFactory::create_decompressor(CompressionType::None);
        assert_eq!(decompressor.compression_type(), CompressionType::None);
    }

    #[test]
    fn test_compression_info() {
        let info = CompressionInfo::new(1000, 500, 1000000);
        assert_eq!(info.uncompressed_size, 1000);
        assert_eq!(info.compressed_size, 500);
        assert_eq!(info.compression_ratio(), 0.5);
        assert_eq!(info.space_saved(), 500);
        assert_eq!(info.space_saved_percent(), 50.0);
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_snappy_compression() {
        let data = b"hello world hello world hello world";
        let context = CompressionContext::default();
        let compressor = SnappyCompressor;

        let compressed = compressor.compress(data, &context).unwrap();
        assert!(compressed.len() < data.len()); // Should be smaller

        let decompressed = compressor.decompress(&compressed, &UncompressionContext::default()).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compression_util_benchmark() {
        let data = b"hello world ".repeat(100);
        let results = CompressionUtil::benchmark_compression(&data);

        assert!(!results.is_empty());

        // Check that no compression is included
        assert!(results.iter().any(|(ct, _)| *ct == CompressionType::None));
    }
}
//! Configuration options for ToplingDB
//!
//! This module provides various option structures to configure database behavior.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::compression::CompressionType;
use crate::env::Env;
use crate::filter::FilterPolicy;
use crate::merge::MergeOperator;

/// Table options for SSTable configuration
#[derive(Debug, Clone)]
pub struct TableOptions {
    /// Block size for SST files
    pub block_size: usize,
    /// Compression type
    pub compression: CompressionType,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            block_size: 4 * 1024, // 4KB
            compression: CompressionType::Snappy,
        }
    }
}

/// Database options
#[derive(Debug, Clone)]
pub struct Options {
    /// Database options
    pub db_options: DBOptions,
    /// Column family options
    pub cf_options: ColumnFamilyOptions,
    /// Table options
    pub table_options: TableOptions,
}

impl Options {
    /// Create new Options with default values
    pub fn new() -> Self {
        Self {
            db_options: DBOptions::default(),
            cf_options: ColumnFamilyOptions::default(),
            table_options: TableOptions::default(),
        }
    }

    /// Set whether to create database if missing
    pub fn create_if_missing(&mut self, create: bool) {
        self.db_options.create_if_missing = create;
    }

    /// Set whether to create missing column families
    pub fn create_missing_column_families(&mut self, create: bool) {
        self.db_options.create_missing_column_families = create;
    }

    /// Set error if database exists
    pub fn error_if_exists(&mut self, error: bool) {
        self.db_options.error_if_exists = error;
    }

    /// Set paranoid checks
    pub fn paranoid_checks(&mut self, paranoid: bool) {
        self.db_options.paranoid_checks = paranoid;
    }

    /// Set environment
    pub fn env(&mut self, env: Arc<dyn Env>) {
        self.db_options.env = Some(env);
    }

    /// Set info log level
    pub fn info_log_level(&mut self, level: InfoLogLevel) {
        self.db_options.info_log_level = level;
    }

    /// Set write buffer size
    pub fn write_buffer_size(&mut self, size: usize) {
        self.cf_options.write_buffer_size = size;
    }

    /// Set max write buffer number
    pub fn max_write_buffer_number(&mut self, num: u32) {
        self.cf_options.max_write_buffer_number = num;
    }

    /// Set compression type
    pub fn compression(&mut self, compression: CompressionType) {
        self.cf_options.compression = compression;
    }

    /// Set block size
    pub fn block_size(&mut self, size: usize) {
        self.cf_options.block_size = size;
    }

    /// Set block cache size
    pub fn block_cache_size(&mut self, size: usize) {
        self.cf_options.block_cache_size = size;
    }

    /// Set filter policy
    pub fn filter_policy(&mut self, policy: Arc<dyn FilterPolicy>) {
        self.cf_options.filter_policy = Some(policy);
    }

    /// Set merge operator
    pub fn merge_operator(&mut self, merge_op: Arc<dyn MergeOperator>) {
        self.cf_options.merge_operator = Some(merge_op);
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

/// Database-specific options
#[derive(Debug, Clone)]
pub struct DBOptions {
    /// Create database if missing
    pub create_if_missing: bool,
    /// Create missing column families
    pub create_missing_column_families: bool,
    /// Error if database exists
    pub error_if_exists: bool,
    /// Enable paranoid checks
    pub paranoid_checks: bool,
    /// Environment to use
    pub env: Option<Arc<dyn Env>>,
    /// Info log level
    pub info_log_level: InfoLogLevel,
    /// Maximum number of open files
    pub max_open_files: i32,
    /// Maximum total size of WAL files
    pub max_total_wal_size: u64,
    /// Statistics update period
    pub stats_dump_period_sec: u32,
    /// Statistics persist period
    pub stats_persist_period_sec: u32,
    /// Enable statistics
    pub statistics: bool,
    /// Use fsync instead of fdatasync
    pub use_fsync: bool,
    /// Database paths for multiple directories
    pub db_paths: Vec<DbPath>,
    /// WAL directory
    pub wal_dir: Option<PathBuf>,
    /// Delete obsolete files period
    pub delete_obsolete_files_period_micros: u64,
    /// Base background compactions
    pub base_background_compactions: i32,
    /// Max background compactions
    pub max_background_compactions: i32,
    /// Max subcompactions
    pub max_subcompactions: u32,
    /// Max background flushes
    pub max_background_flushes: i32,
    /// Max log file size
    pub max_log_file_size: usize,
    /// Log file time to roll
    pub log_file_time_to_roll: usize,
    /// Keep log file number
    pub keep_log_file_num: usize,
    /// Recycle log file number
    pub recycle_log_file_num: usize,
    /// Max manifest file size
    pub max_manifest_file_size: u64,
    /// Table cache number of shard bits
    pub table_cache_numshardbits: i32,
    /// WAL ttl seconds
    pub wal_ttl_seconds: u64,
    /// WAL size limit MB
    pub wal_size_limit_mb: u64,
    /// Manifest preallocation size
    pub manifest_preallocation_size: usize,
    /// Allow mmap reads
    pub allow_mmap_reads: bool,
    /// Allow mmap writes
    pub allow_mmap_writes: bool,
    /// Use direct reads
    pub use_direct_reads: bool,
    /// Use direct IO for flush and compaction
    pub use_direct_io_for_flush_and_compaction: bool,
    /// Allow fallocate
    pub allow_fallocate: bool,
    /// Is fd close on exec
    pub is_fd_close_on_exec: bool,
    /// Advise random on open
    pub advise_random_on_open: bool,
    /// DB write buffer size
    pub db_write_buffer_size: usize,
    /// Access hint on compaction start
    pub access_hint_on_compaction_start: AccessHint,
    /// New table reader for compaction inputs
    pub new_table_reader_for_compaction_inputs: bool,
    /// Compaction readahead size
    pub compaction_readahead_size: usize,
    /// Random access max buffer size
    pub random_access_max_buffer_size: usize,
    /// Writable file max buffer size
    pub writable_file_max_buffer_size: usize,
    /// Use adaptive mutex
    pub use_adaptive_mutex: bool,
    /// Bytes per sync
    pub bytes_per_sync: u64,
    /// WAL bytes per sync
    pub wal_bytes_per_sync: u64,
    /// Enable thread tracking
    pub enable_thread_tracking: bool,
    /// Delayed write rate
    pub delayed_write_rate: u64,
    /// Enable pipelined write
    pub enable_pipelined_write: bool,
    /// Unordered write
    pub unordered_write: bool,
    /// Allow concurrent memtable write
    pub allow_concurrent_memtable_write: bool,
    /// Enable write thread adaptive yield
    pub enable_write_thread_adaptive_yield: bool,
    /// Max write batch group size bytes
    pub max_write_batch_group_size_bytes: u64,
    /// Write thread max yield usec
    pub write_thread_max_yield_usec: u64,
    /// Write thread slow yield usec
    pub write_thread_slow_yield_usec: u64,
    /// Skip stats update on db open
    pub skip_stats_update_on_db_open: bool,
    /// WAL recovery mode
    pub wal_recovery_mode: WalRecoveryMode,
    /// Allow 2pc
    pub allow_2pc: bool,
    /// Row cache
    pub row_cache: Option<Arc<dyn crate::cache::Cache>>,
    /// WAL filter
    pub wal_filter: Option<Arc<dyn WalFilter>>,
    /// Fail if options file error
    pub fail_if_options_file_error: bool,
    /// Dump malloc stats
    pub dump_malloc_stats: bool,
    /// Avoid flush during recovery
    pub avoid_flush_during_recovery: bool,
    /// Avoid flush during shutdown
    pub avoid_flush_during_shutdown: bool,
    /// Allow ingest behind
    pub allow_ingest_behind: bool,
    /// Preserve deletes
    pub preserve_deletes: bool,
    /// Two write queues
    pub two_write_queues: bool,
    /// Manual wal flush
    pub manual_wal_flush: bool,
    /// Atomic flush
    pub atomic_flush: bool,
    /// Avoid unnecessary blocking io
    pub avoid_unnecessary_blocking_io: bool,
    /// Persist stats to disk
    pub persist_stats_to_disk: bool,
    /// Write dbid to manifest
    pub write_dbid_to_manifest: bool,
    /// Log readahead size
    pub log_readahead_size: usize,
    /// File checksum gen factory
    pub file_checksum_gen_factory: Option<Arc<dyn FileChecksumGenFactory>>,
    /// Best efforts recovery
    pub best_efforts_recovery: bool,
    /// Max bgerror resume count
    pub max_bgerror_resume_count: i32,
    /// Bgerror resume retry interval
    pub bgerror_resume_retry_interval: u64,
}

impl Default for DBOptions {
    fn default() -> Self {
        Self {
            create_if_missing: false,
            create_missing_column_families: false,
            error_if_exists: false,
            paranoid_checks: true,
            env: None,
            info_log_level: InfoLogLevel::Info,
            max_open_files: -1,
            max_total_wal_size: 0,
            stats_dump_period_sec: 600,
            stats_persist_period_sec: 600,
            statistics: false,
            use_fsync: false,
            db_paths: Vec::new(),
            wal_dir: None,
            delete_obsolete_files_period_micros: 6 * 60 * 60 * 1000000, // 6 hours
            base_background_compactions: -1,
            max_background_compactions: -1,
            max_subcompactions: 1,
            max_background_flushes: -1,
            max_log_file_size: 0,
            log_file_time_to_roll: 0,
            keep_log_file_num: 1000,
            recycle_log_file_num: 0,
            max_manifest_file_size: 1024 * 1024 * 1024, // 1GB
            table_cache_numshardbits: 6,
            wal_ttl_seconds: 0,
            wal_size_limit_mb: 0,
            manifest_preallocation_size: 4 * 1024 * 1024, // 4MB
            allow_mmap_reads: false,
            allow_mmap_writes: false,
            use_direct_reads: false,
            use_direct_io_for_flush_and_compaction: false,
            allow_fallocate: true,
            is_fd_close_on_exec: true,
            advise_random_on_open: true,
            db_write_buffer_size: 0,
            access_hint_on_compaction_start: AccessHint::Normal,
            new_table_reader_for_compaction_inputs: true,
            compaction_readahead_size: 0,
            random_access_max_buffer_size: 1024 * 1024, // 1MB
            writable_file_max_buffer_size: 1024 * 1024, // 1MB
            use_adaptive_mutex: false,
            bytes_per_sync: 0,
            wal_bytes_per_sync: 0,
            enable_thread_tracking: false,
            delayed_write_rate: 16 * 1024 * 1024, // 16MB/s
            enable_pipelined_write: false,
            unordered_write: false,
            allow_concurrent_memtable_write: true,
            enable_write_thread_adaptive_yield: true,
            max_write_batch_group_size_bytes: 1024 * 1024, // 1MB
            write_thread_max_yield_usec: 100,
            write_thread_slow_yield_usec: 3,
            skip_stats_update_on_db_open: false,
            wal_recovery_mode: WalRecoveryMode::PointInTimeRecovery,
            allow_2pc: false,
            row_cache: None,
            wal_filter: None,
            fail_if_options_file_error: false,
            dump_malloc_stats: false,
            avoid_flush_during_recovery: false,
            avoid_flush_during_shutdown: false,
            allow_ingest_behind: false,
            preserve_deletes: false,
            two_write_queues: false,
            manual_wal_flush: false,
            atomic_flush: false,
            avoid_unnecessary_blocking_io: false,
            persist_stats_to_disk: false,
            write_dbid_to_manifest: false,
            log_readahead_size: 0,
            file_checksum_gen_factory: None,
            best_efforts_recovery: false,
            max_bgerror_resume_count: i32::MAX,
            bgerror_resume_retry_interval: 1000000, // 1 second
        }
    }
}

/// Column family options
#[derive(Debug, Clone)]
pub struct ColumnFamilyOptions {
    /// Write buffer size
    pub write_buffer_size: usize,
    /// Maximum write buffer number
    pub max_write_buffer_number: u32,
    /// Minimum write buffer number to merge
    pub min_write_buffer_number_to_merge: u32,
    /// Maximum write buffer number to maintain
    pub max_write_buffer_number_to_maintain: u32,
    /// Compression type
    pub compression: CompressionType,
    /// Compression per level
    pub compression_per_level: Vec<CompressionType>,
    /// Bottommost compression
    pub bottommost_compression: CompressionType,
    /// Compression options
    pub compression_opts: CompressionOptions,
    /// Bottommost compression options
    pub bottommost_compression_opts: CompressionOptions,
    /// Number of levels
    pub num_levels: u32,
    /// Level 0 file number compaction trigger
    pub level0_file_num_compaction_trigger: u32,
    /// Level 0 slowdown writes trigger
    pub level0_slowdown_writes_trigger: u32,
    /// Level 0 stop writes trigger
    pub level0_stop_writes_trigger: u32,
    /// Target file size base
    pub target_file_size_base: u64,
    /// Target file size multiplier
    pub target_file_size_multiplier: u32,
    /// Max bytes for level base
    pub max_bytes_for_level_base: u64,
    /// Level compaction dynamic level bytes
    pub level_compaction_dynamic_level_bytes: bool,
    /// Max bytes for level multiplier
    pub max_bytes_for_level_multiplier: f64,
    /// Max bytes for level multiplier additional
    pub max_bytes_for_level_multiplier_additional: Vec<u32>,
    /// Max compaction bytes
    pub max_compaction_bytes: u64,
    /// Soft pending compaction bytes limit
    pub soft_pending_compaction_bytes_limit: u64,
    /// Hard pending compaction bytes limit
    pub hard_pending_compaction_bytes_limit: u64,
    /// Compaction style
    pub compaction_style: CompactionStyle,
    /// Compaction priority
    pub compaction_priority: CompactionPriority,
    /// Compaction options universal
    pub compaction_options_universal: UniversalCompactionOptions,
    /// Compaction options FIFO
    pub compaction_options_fifo: FifoCompactionOptions,
    /// Max sequential skip in iterations
    pub max_sequential_skip_in_iterations: u64,
    /// Memtable factory
    pub memtable_factory: Option<Arc<dyn MemTableRepFactory>>,
    /// Memtable insert with hint prefix extractor
    pub memtable_insert_with_hint_prefix_extractor: Option<Arc<dyn SliceTransform>>,
    /// Memtable huge page size
    pub memtable_huge_page_size: usize,
    /// Memtable prefix bloom size ratio
    pub memtable_prefix_bloom_size_ratio: f64,
    /// Memtable whole key filtering
    pub memtable_whole_key_filtering: bool,
    /// Block size
    pub block_size: usize,
    /// Block size deviation
    pub block_size_deviation: u32,
    /// Block restart interval
    pub block_restart_interval: u32,
    /// Index block restart interval
    pub index_block_restart_interval: u32,
    /// Metadata block size
    pub metadata_block_size: u64,
    /// Partition filters
    pub partition_filters: bool,
    /// Use delta encoding
    pub use_delta_encoding: bool,
    /// Filter policy
    pub filter_policy: Option<Arc<dyn FilterPolicy>>,
    /// Whole key filtering
    pub whole_key_filtering: bool,
    /// Verify checksums in compaction
    pub verify_checksums_in_compaction: bool,
    /// Read amplification bytes per bit
    pub read_amp_bytes_per_bit: u32,
    /// Format version
    pub format_version: u32,
    /// Enable index compression
    pub enable_index_compression: bool,
    /// Block align
    pub block_align: bool,
    /// Max auto readahead size
    pub max_auto_readahead_size: usize,
    /// Prepopulate block cache
    pub prepopulate_block_cache: PrepopulateBlockCache,
    /// Initial auto readahead size
    pub initial_auto_readahead_size: usize,
    /// Cache index and filter blocks
    pub cache_index_and_filter_blocks: bool,
    /// Cache index and filter blocks with high priority
    pub cache_index_and_filter_blocks_with_high_priority: bool,
    /// Pin l0 filter and index blocks in cache
    pub pin_l0_filter_and_index_blocks_in_cache: bool,
    /// Pin top level index and filter
    pub pin_top_level_index_and_filter: bool,
    /// Index type
    pub index_type: BlockBasedIndexType,
    /// Data block index type
    pub data_block_index_type: DataBlockIndexType,
    /// Index shortening
    pub index_shortening: IndexShorteningMode,
    /// Data block hash table util ratio
    pub data_block_hash_table_util_ratio: f64,
    /// Hash index allow collision
    pub hash_index_allow_collision: bool,
    /// Checksum
    pub checksum: ChecksumType,
    /// No block cache
    pub no_block_cache: bool,
    /// Block cache
    pub block_cache: Option<Arc<dyn crate::cache::Cache>>,
    /// Block cache compressed
    pub block_cache_compressed: Option<Arc<dyn crate::cache::Cache>>,
    /// Block cache size
    pub block_cache_size: usize,
    /// Block cache compressed size
    pub block_cache_compressed_size: usize,
    /// Cache usage options
    pub cache_usage_options: CacheUsageOptions,
    /// Max file opening threads
    pub max_file_opening_threads: u32,
    /// Compaction filter
    pub compaction_filter: Option<Arc<dyn CompactionFilter>>,
    /// Compaction filter factory
    pub compaction_filter_factory: Option<Arc<dyn CompactionFilterFactory>>,
    /// Merge operator
    pub merge_operator: Option<Arc<dyn MergeOperator>>,
    /// Prefix extractor
    pub prefix_extractor: Option<Arc<dyn SliceTransform>>,
    /// Optimize filters for hits
    pub optimize_filters_for_hits: bool,
    /// Delete range iterator name
    pub delete_range_iterator_name: String,
    /// Paranoid file checks
    pub paranoid_file_checks: bool,
    /// Force consistency checks
    pub force_consistency_checks: bool,
    /// Report bg io stats
    pub report_bg_io_stats: bool,
    /// TTL
    pub ttl: u64,
    /// Periodic compaction seconds
    pub periodic_compaction_seconds: u64,
    /// Sample for compression
    pub sample_for_compression: u64,
    /// Default temperature
    pub default_temperature: Temperature,
    /// Preclude last level data seconds
    pub preclude_last_level_data_seconds: u64,
    /// Preserve internal time seconds
    pub preserve_internal_time_seconds: u64,
    /// Enable blob files
    pub enable_blob_files: bool,
    /// Minimum blob size
    pub min_blob_size: u64,
    /// Blob file size
    pub blob_file_size: u64,
    /// Blob compression type
    pub blob_compression_type: CompressionType,
    /// Enable blob garbage collection
    pub enable_blob_garbage_collection: bool,
    /// Blob garbage collection age cutoff
    pub blob_garbage_collection_age_cutoff: f64,
    /// Blob garbage collection force threshold
    pub blob_garbage_collection_force_threshold: f64,
    /// Blob compaction readahead size
    pub blob_compaction_readahead_size: u64,
    /// Blob file starting level
    pub blob_file_starting_level: u32,
    /// Use blob cache
    pub use_blob_cache: bool,
    /// Blob cache
    pub blob_cache: Option<Arc<dyn crate::cache::Cache>>,
}

impl Default for ColumnFamilyOptions {
    fn default() -> Self {
        Self {
            write_buffer_size: 64 * 1024 * 1024, // 64MB
            max_write_buffer_number: 2,
            min_write_buffer_number_to_merge: 1,
            max_write_buffer_number_to_maintain: 0,
            compression: CompressionType::Snappy,
            compression_per_level: Vec::new(),
            bottommost_compression: CompressionType::None,
            compression_opts: CompressionOptions::default(),
            bottommost_compression_opts: CompressionOptions::default(),
            num_levels: 7,
            level0_file_num_compaction_trigger: 4,
            level0_slowdown_writes_trigger: 20,
            level0_stop_writes_trigger: 36,
            target_file_size_base: 64 * 1024 * 1024, // 64MB
            target_file_size_multiplier: 1,
            max_bytes_for_level_base: 256 * 1024 * 1024, // 256MB
            level_compaction_dynamic_level_bytes: false,
            max_bytes_for_level_multiplier: 10.0,
            max_bytes_for_level_multiplier_additional: Vec::new(),
            max_compaction_bytes: 1024 * 1024 * 1024, // 1GB
            soft_pending_compaction_bytes_limit: 64 * 1024 * 1024 * 1024, // 64GB
            hard_pending_compaction_bytes_limit: 256 * 1024 * 1024 * 1024, // 256GB
            compaction_style: CompactionStyle::Level,
            compaction_priority: CompactionPriority::ByCompensatedSize,
            compaction_options_universal: UniversalCompactionOptions::default(),
            compaction_options_fifo: FifoCompactionOptions::default(),
            max_sequential_skip_in_iterations: 8,
            memtable_factory: None,
            memtable_insert_with_hint_prefix_extractor: None,
            memtable_huge_page_size: 0,
            memtable_prefix_bloom_size_ratio: 0.0,
            memtable_whole_key_filtering: false,
            block_size: 4 * 1024, // 4KB
            block_size_deviation: 10,
            block_restart_interval: 16,
            index_block_restart_interval: 1,
            metadata_block_size: 4 * 1024, // 4KB
            partition_filters: false,
            use_delta_encoding: true,
            filter_policy: None,
            whole_key_filtering: true,
            verify_checksums_in_compaction: true,
            read_amp_bytes_per_bit: 0,
            format_version: 5,
            enable_index_compression: true,
            block_align: false,
            max_auto_readahead_size: 256 * 1024, // 256KB
            prepopulate_block_cache: PrepopulateBlockCache::Disabled,
            initial_auto_readahead_size: 8 * 1024, // 8KB
            cache_index_and_filter_blocks: false,
            cache_index_and_filter_blocks_with_high_priority: true,
            pin_l0_filter_and_index_blocks_in_cache: false,
            pin_top_level_index_and_filter: true,
            index_type: BlockBasedIndexType::BinarySearch,
            data_block_index_type: DataBlockIndexType::BinarySearch,
            index_shortening: IndexShorteningMode::ShortenSeparators,
            data_block_hash_table_util_ratio: 0.75,
            hash_index_allow_collision: true,
            checksum: ChecksumType::Crc32c,
            no_block_cache: false,
            block_cache: None,
            block_cache_compressed: None,
            block_cache_size: 8 * 1024 * 1024, // 8MB
            block_cache_compressed_size: 0,
            cache_usage_options: CacheUsageOptions::default(),
            max_file_opening_threads: 16,
            compaction_filter: None,
            compaction_filter_factory: None,
            merge_operator: None,
            prefix_extractor: None,
            optimize_filters_for_hits: false,
            delete_range_iterator_name: String::new(),
            paranoid_file_checks: false,
            force_consistency_checks: true,
            report_bg_io_stats: false,
            ttl: 0,
            periodic_compaction_seconds: 0,
            sample_for_compression: 0,
            default_temperature: Temperature::Unknown,
            preclude_last_level_data_seconds: 0,
            preserve_internal_time_seconds: 0,
            enable_blob_files: false,
            min_blob_size: 0,
            blob_file_size: 256 * 1024 * 1024, // 256MB
            blob_compression_type: CompressionType::None,
            enable_blob_garbage_collection: false,
            blob_garbage_collection_age_cutoff: 0.25,
            blob_garbage_collection_force_threshold: 1.0,
            blob_compaction_readahead_size: 0,
            blob_file_starting_level: 0,
            use_blob_cache: false,
            blob_cache: None,
        }
    }
}

/// Read options
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Verify checksums
    pub verify_checksums: bool,
    /// Fill cache
    pub fill_cache: bool,
    /// Snapshot to read from
    pub snapshot: Option<crate::snapshot::Snapshot>,
    /// Read tier
    pub read_tier: ReadTier,
    /// Tailing iterator
    pub tailing: bool,
    /// Managed
    pub managed: bool,
    /// Total order seek
    pub total_order_seek: bool,
    /// Max skippable internal keys
    pub max_skippable_internal_keys: u64,
    /// Read callback
    pub read_callback: Option<Arc<dyn ReadCallback>>,
    /// Allow blob prefetching
    pub allow_blob_prefetching: bool,
    /// User timestamp
    pub timestamp: Option<Vec<u8>>,
    /// Iterate lower bound
    pub iterate_lower_bound: Option<Vec<u8>>,
    /// Iterate upper bound
    pub iterate_upper_bound: Option<Vec<u8>>,
    /// Table filter
    pub table_filter: Option<Arc<dyn TableFilter>>,
    /// Auto readahead size
    pub auto_readahead_size: usize,
    /// Async IO
    pub async_io: bool,
    /// Optimize for DB size
    pub optimize_for_size_approximation: bool,
    /// Ignore range deletions
    pub ignore_range_deletions: bool,
    /// Value size soft limit
    pub value_size_soft_limit: u64,
    /// Deadline
    pub deadline: Option<std::time::Instant>,
    /// IO timeout
    pub io_timeout: Option<Duration>,
    /// Adaptive readahead
    pub adaptive_readahead: bool,
}

/// Write options
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Sync writes to disk
    pub sync: bool,
    /// Disable WAL
    pub disablewal: bool,
    /// Ignore missing column families
    pub ignore_missing_column_families: bool,
    /// No slowdown
    pub no_slowdown: bool,
    /// Low priority
    pub low_pri: bool,
    /// Memtable insert hint per batch
    pub memtable_insert_hint_per_batch: bool,
    /// Rate limiter priority
    pub rate_limiter_priority: RateLimiterPriority,
    /// Protection bytes per key
    pub protection_bytes_per_key: u8,
}

/// Flush options
#[derive(Debug, Clone, Default)]
pub struct FlushOptions {
    /// Wait for completion
    pub wait: bool,
    /// Allow write stall
    pub allow_write_stall: bool,
}

/// Compaction options
#[derive(Debug, Clone, Default)]
pub struct CompactionOptions {
    /// Compression type
    pub compression: CompressionType,
    /// Output file size limit
    pub output_file_size_limit: u64,
    /// Max subcompactions
    pub max_subcompactions: u32,
}

// Supporting enums and structures

/// Info log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoLogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Fatal = 4,
    Header = 5,
}

/// Database path configuration
#[derive(Debug, Clone)]
pub struct DbPath {
    /// Path to database directory
    pub path: PathBuf,
    /// Target size for this path
    pub target_size: u64,
}

/// Access hints for file operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessHint {
    None,
    Normal,
    Sequential,
    WillNeed,
}

/// WAL recovery modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalRecoveryMode {
    /// Original WAL recovery
    TolerateCorruptedTailRecords,
    /// Absolute consistency mode
    AbsoluteConsistency,
    /// Point in time recovery
    PointInTimeRecovery,
    /// Skip any corrupted record
    SkipAnyCorruptedRecords,
}

/// Compaction styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStyle {
    Level,
    Universal,
    Fifo,
    None,
}

/// Compaction priorities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionPriority {
    ByCompensatedSize,
    OldestLargestSeqFirst,
    OldestSmallestSeqFirst,
    MinOverlappingRatio,
    RoundRobin,
}

/// Temperature hints for data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Temperature {
    Unknown,
    Hot,
    Warm,
    Cool,
    Cold,
}

/// Block cache prepopulate options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrepopulateBlockCache {
    Disabled,
    FlushOnly,
}

/// Index types for block-based tables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockBasedIndexType {
    BinarySearch,
    HashSearch,
    TwoLevelIndexSearch,
}

/// Data block index types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataBlockIndexType {
    BinarySearch,
    BinarySearchAndHash,
}

/// Index shortening modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexShorteningMode {
    NoShortening,
    ShortenSeparators,
    ShortenSeparatorsAndSuccessor,
}

/// Checksum types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumType {
    NoChecksum,
    Crc32c,
    Xxh64,
    Xxh3,
}

/// Read tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadTier {
    #[default]
    ReadAllTier,
    BlockCacheTier,
    PersistedTier,
    MemtableTier,
}

/// Rate limiter priorities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RateLimiterPriority {
    #[default]
    High,
    Low,
}

/// Compression options
#[derive(Debug, Clone)]
pub struct CompressionOptions {
    /// Window bits
    pub window_bits: i32,
    /// Compression level
    pub level: i32,
    /// Strategy
    pub strategy: i32,
    /// Maximum dictionary bytes
    pub max_dict_bytes: u32,
    /// Z standard maximum train bytes
    pub zstd_max_train_bytes: u32,
    /// Parallel threads
    pub parallel_threads: u32,
    /// Enabled
    pub enabled: bool,
    /// Maximum dictionary buffer bytes
    pub max_dict_buffer_bytes: u64,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            window_bits: -14,
            level: 32767,
            strategy: 0,
            max_dict_bytes: 0,
            zstd_max_train_bytes: 0,
            parallel_threads: 1,
            enabled: false,
            max_dict_buffer_bytes: 0,
        }
    }
}

/// Universal compaction options
#[derive(Debug, Clone)]
pub struct UniversalCompactionOptions {
    /// Size ratio
    pub size_ratio: u32,
    /// Minimum merge width
    pub min_merge_width: u32,
    /// Maximum merge width
    pub max_merge_width: u32,
    /// Maximum size amplification percent
    pub max_size_amplification_percent: u32,
    /// Compression size percent
    pub compression_size_percent: i32,
    /// Stop style
    pub stop_style: UniversalCompactionStopStyle,
    /// Allow trivial move
    pub allow_trivial_move: bool,
}

impl Default for UniversalCompactionOptions {
    fn default() -> Self {
        Self {
            size_ratio: 1,
            min_merge_width: 2,
            max_merge_width: u32::MAX,
            max_size_amplification_percent: 200,
            compression_size_percent: -1,
            stop_style: UniversalCompactionStopStyle::TotalSize,
            allow_trivial_move: false,
        }
    }
}

/// Universal compaction stop styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniversalCompactionStopStyle {
    SimilarSize,
    TotalSize,
}

/// FIFO compaction options
#[derive(Debug, Clone)]
pub struct FifoCompactionOptions {
    /// Maximum table files size
    pub max_table_files_size: u64,
    /// Allow compaction
    pub allow_compaction: bool,
}

impl Default for FifoCompactionOptions {
    fn default() -> Self {
        Self {
            max_table_files_size: 1024 * 1024 * 1024, // 1GB
            allow_compaction: false,
        }
    }
}

/// Cache usage options
#[derive(Debug, Clone, Default)]
pub struct CacheUsageOptions {
    /// Options overrides
    pub options_overrides: Vec<CacheEntryRoleOptions>,
}

/// Cache entry role options
#[derive(Debug, Clone)]
pub struct CacheEntryRoleOptions {
    /// Charged
    pub charged: CacheEntryRoleOptionsCharged,
}

/// Cache entry role options charged
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheEntryRoleOptionsCharged {
    Default,
    Enabled,
    Disabled,
}

// Trait definitions for interfaces

/// WAL filter trait
pub trait WalFilter: Send + Sync + std::fmt::Debug {
    /// Column family log number map
    fn column_family_log_number_map(
        &self,
        cf_lognumber_map: &std::collections::HashMap<u32, u64>,
        cf_name_id_map: &std::collections::HashMap<String, u32>,
    );

    /// Log record found
    fn log_record_found(
        &self,
        log_number: u64,
        log_file_name: &str,
        batch: &crate::write_batch::WriteBatch,
        new_batch: &mut crate::write_batch::WriteBatch,
    ) -> crate::status::Status;

    /// Name of the filter
    fn name(&self) -> &str;
}

/// File checksum generator factory
pub trait FileChecksumGenFactory: Send + Sync + std::fmt::Debug {
    /// Create file checksum generator
    fn create_file_checksum_generator(&self, context: ChecksumGenContext) -> Box<dyn FileChecksumGenerator>;

    /// Name of the factory
    fn name(&self) -> &str;
}

/// File checksum generator
pub trait FileChecksumGenerator: Send + Sync {
    /// Update checksum
    fn update(&mut self, data: &[u8]);

    /// Finalize checksum
    fn finalize(&mut self) -> String;

    /// Reset the generator
    fn reset(&mut self);

    /// Name of the generator
    fn name(&self) -> &str;
}

/// Checksum generation context
#[derive(Debug, Clone)]
pub struct ChecksumGenContext {
    /// File name
    pub file_name: String,
}

/// Memtable representation factory
pub trait MemTableRepFactory: Send + Sync + std::fmt::Debug {
    /// Create memtable representation
    fn create_memtable_rep(&self, transform: Option<&dyn SliceTransform>, arena: &mut dyn MemArena) -> Box<dyn MemTableRep>;

    /// Name of the factory
    fn name(&self) -> &str;
}

/// Memory arena trait
pub trait MemArena: Send + Sync {
    /// Allocate memory
    fn allocate(&mut self, size: usize) -> *mut u8;

    /// Allocate aligned memory
    fn allocate_aligned(&mut self, size: usize, alignment: usize) -> *mut u8;

    /// Get memory usage
    fn memory_usage(&self) -> usize;
}

/// Memtable representation trait
pub trait MemTableRep: Send + Sync {
    /// Insert key
    fn insert(&mut self, key: &[u8]);

    /// Get iterator
    fn get_iterator(&self) -> Box<dyn crate::iterator::Iterator>;

    /// Contains key
    fn contains(&self, key: &[u8]) -> bool;

    /// Approximate size
    fn approximate_size(&self, start: &[u8], end: &[u8]) -> u64;

    /// Approximate num entries
    fn approximate_num_entries(&self) -> u64;

    /// Approximate memory usage
    fn approximate_memory_usage(&self) -> usize;
}

/// Slice transform trait
pub trait SliceTransform: Send + Sync + std::fmt::Debug {
    /// Transform slice
    fn transform(&self, key: &[u8]) -> &[u8];

    /// In domain
    fn in_domain(&self, key: &[u8]) -> bool;

    /// In range
    fn in_range(&self, key: &[u8]) -> bool;

    /// Name
    fn name(&self) -> &str;
}

/// Compaction filter trait
pub trait CompactionFilter: Send + Sync + std::fmt::Debug {
    /// Filter
    fn filter(&self, level: u32, key: &[u8], value: &[u8]) -> CompactionFilterDecision;

    /// Filter merge operand
    fn filter_merge_operand(&self, level: u32, key: &[u8], operand: &[u8]) -> bool;

    /// Name
    fn name(&self) -> &str;
}

/// Compaction filter decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionFilterDecision {
    Keep,
    Remove,
    ChangeValue,
}

/// Compaction filter factory trait
pub trait CompactionFilterFactory: Send + Sync + std::fmt::Debug {
    /// Create compaction filter
    fn create_compaction_filter(&self, context: &CompactionFilterContext) -> Option<Box<dyn CompactionFilter>>;

    /// Name
    fn name(&self) -> &str;
}

/// Compaction filter context
#[derive(Debug, Clone)]
pub struct CompactionFilterContext {
    /// Is full compaction
    pub is_full_compaction: bool,
    /// Is manual compaction
    pub is_manual_compaction: bool,
    /// Column family id
    pub column_family_id: u32,
    /// Reason for compaction
    pub reason: CompactionReason,
}

/// Compaction reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionReason {
    Unknown,
    LevelL0FilesNum,
    LevelMaxLevelSize,
    UniversalSizeAmplification,
    UniversalSizeRatio,
    UniversalSortedRunNum,
    FifoMaxSize,
    FifoTtl,
    Manual,
    FilesMarkedForCompaction,
    BottommostFiles,
    Ttl,
    Flush,
    ExternalSstIngestion,
    PeriodicCompaction,
    ChangeTemperature,
    ForcedBlobGc,
}

/// Read callback trait
pub trait ReadCallback: Send + Sync + std::fmt::Debug {
    /// Allow readahead
    fn allow_readahead(&self) -> bool;
}

/// Table filter trait
pub trait TableFilter: Send + Sync + std::fmt::Debug {
    /// Filter table
    fn filter(&self, table_properties: &crate::table::TableProperties) -> bool;

    /// Name
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_creation() {
        let opts = Options::new();
        assert!(!opts.db_options.create_if_missing);
        assert_eq!(opts.cf_options.write_buffer_size, 64 * 1024 * 1024);
    }

    #[test]
    fn test_options_modification() {
        let mut opts = Options::new();
        opts.create_if_missing(true);
        opts.write_buffer_size(128 * 1024 * 1024);

        assert!(opts.db_options.create_if_missing);
        assert_eq!(opts.cf_options.write_buffer_size, 128 * 1024 * 1024);
    }

    #[test]
    fn test_read_write_options() {
        let read_opts = ReadOptions::default();
        assert!(!read_opts.verify_checksums);
        assert!(!read_opts.fill_cache);

        let write_opts = WriteOptions::default();
        assert!(!write_opts.sync);
        assert!(!write_opts.disablewal);
    }
}
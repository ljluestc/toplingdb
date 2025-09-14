//! Hash function utilities

use std::hash::{Hash, Hasher};
use xxhash_rust::xxh64::Xxh64;
use blake3::Hasher as Blake3Hasher;

/// Hash function types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    /// xxHash 64-bit
    XxHash64,
    /// BLAKE3
    Blake3,
    /// FarmHash (placeholder)
    FarmHash,
}

impl Default for HashType {
    fn default() -> Self {
        HashType::XxHash64
    }
}

/// Generic hash function interface
pub trait HashFunction {
    /// Hash a byte slice
    fn hash(&self, data: &[u8]) -> u64;

    /// Hash with a seed
    fn hash_with_seed(&self, data: &[u8], seed: u64) -> u64;
}

/// xxHash implementation
pub struct XxHasher {
    seed: u64,
}

impl XxHasher {
    /// Create new xxHash hasher
    pub fn new() -> Self {
        Self { seed: 0 }
    }

    /// Create new xxHash hasher with seed
    pub fn with_seed(seed: u64) -> Self {
        Self { seed }
    }
}

impl Default for XxHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl HashFunction for XxHasher {
    fn hash(&self, data: &[u8]) -> u64 {
        self.hash_with_seed(data, self.seed)
    }

    fn hash_with_seed(&self, data: &[u8], seed: u64) -> u64 {
        xxhash_rust::xxh64::xxh64(data, seed)
    }
}

/// BLAKE3 hasher
pub struct Blake3Hash;

impl Blake3Hash {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Blake3Hash {
    fn default() -> Self {
        Self::new()
    }
}

impl HashFunction for Blake3Hash {
    fn hash(&self, data: &[u8]) -> u64 {
        let hash = blake3::hash(data);
        let bytes = hash.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    fn hash_with_seed(&self, data: &[u8], seed: u64) -> u64 {
        let mut hasher = Blake3Hasher::new();
        hasher.update(&seed.to_le_bytes());
        hasher.update(data);
        let hash = hasher.finalize();
        let bytes = hash.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }
}

/// Get hasher by type
pub fn get_hasher(hash_type: HashType) -> Box<dyn HashFunction> {
    match hash_type {
        HashType::XxHash64 => Box::new(XxHasher::new()),
        HashType::Blake3 => Box::new(Blake3Hash::new()),
        HashType::FarmHash => {
            // Placeholder - would implement FarmHash if needed
            Box::new(XxHasher::new())
        }
    }
}

/// Hash a key using default hasher
pub fn hash_key(key: &[u8]) -> u64 {
    xxhash_rust::xxh64::xxh64(key, 0)
}

/// Hash a key with seed
pub fn hash_key_with_seed(key: &[u8], seed: u64) -> u64 {
    xxhash_rust::xxh64::xxh64(key, seed)
}

/// Hash for partitioning (ensures even distribution)
pub fn partition_hash(key: &[u8], num_partitions: usize) -> usize {
    let hash = hash_key(key);
    (hash as usize) % num_partitions
}

/// Murmur3 hash (32-bit) - simple implementation
pub fn murmur3_hash_32(key: &[u8], seed: u32) -> u32 {
    const C1: u32 = 0xcc9e2d51;
    const C2: u32 = 0x1b873593;
    const R1: u32 = 15;
    const R2: u32 = 13;
    const M: u32 = 5;
    const N: u32 = 0xe6546b64;

    let mut hash = seed;
    let mut i = 0;

    // Process 4-byte chunks
    while i + 4 <= key.len() {
        let mut k = u32::from_le_bytes([key[i], key[i + 1], key[i + 2], key[i + 3]]);
        k = k.wrapping_mul(C1);
        k = k.rotate_left(R1);
        k = k.wrapping_mul(C2);

        hash ^= k;
        hash = hash.rotate_left(R2);
        hash = hash.wrapping_mul(M).wrapping_add(N);
        i += 4;
    }

    // Process remaining bytes
    let mut k = 0u32;
    match key.len() & 3 {
        3 => k ^= (key[i + 2] as u32) << 16,
        2 => k ^= (key[i + 1] as u32) << 8,
        1 => {
            k ^= key[i] as u32;
            k = k.wrapping_mul(C1);
            k = k.rotate_left(R1);
            k = k.wrapping_mul(C2);
            hash ^= k;
        }
        _ => {}
    }

    hash ^= key.len() as u32;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xxhash() {
        let hasher = XxHasher::new();
        let data = b"hello world";

        let hash1 = hasher.hash(data);
        let hash2 = hasher.hash(data);
        assert_eq!(hash1, hash2);

        let hash3 = hasher.hash_with_seed(data, 123);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_blake3() {
        let hasher = Blake3Hash::new();
        let data = b"test data";

        let hash1 = hasher.hash(data);
        let hash2 = hasher.hash(data);
        assert_eq!(hash1, hash2);

        let hash3 = hasher.hash_with_seed(data, 456);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_key() {
        let key = b"test_key";
        let hash1 = hash_key(key);
        let hash2 = hash_key(key);
        assert_eq!(hash1, hash2);

        let hash3 = hash_key_with_seed(key, 789);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_partition_hash() {
        let key = b"partition_test";
        let partition = partition_hash(key, 10);
        assert!(partition < 10);

        // Same key should always go to same partition
        let partition2 = partition_hash(key, 10);
        assert_eq!(partition, partition2);
    }

    #[test]
    fn test_murmur3() {
        let data = b"murmur test";
        let hash1 = murmur3_hash_32(data, 0);
        let hash2 = murmur3_hash_32(data, 0);
        assert_eq!(hash1, hash2);

        let hash3 = murmur3_hash_32(data, 123);
        assert_ne!(hash1, hash3);

        // Test with empty data
        let empty_hash = murmur3_hash_32(&[], 0);
        assert_eq!(empty_hash, 0);
    }

    #[test]
    fn test_get_hasher() {
        let hasher1 = get_hasher(HashType::XxHash64);
        let hasher2 = get_hasher(HashType::Blake3);

        let data = b"hasher test";
        let hash1 = hasher1.hash(data);
        let hash2 = hasher2.hash(data);

        // Different hashers should produce different results
        assert_ne!(hash1, hash2);
    }
}
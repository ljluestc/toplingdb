//! Bloom filter implementation

use crate::util::hash::{hash_key_with_seed, murmur3_hash_32};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Bloom filter for fast set membership testing
pub struct BloomFilter {
    /// Bit array
    bits: Vec<u8>,
    /// Number of hash functions
    num_hash_functions: usize,
    /// Number of bits
    num_bits: usize,
    /// Number of keys added
    num_keys: usize,
}

impl BloomFilter {
    /// Create a new bloom filter
    ///
    /// # Arguments
    /// * `expected_keys` - Expected number of keys
    /// * `false_positive_rate` - Desired false positive rate (0.0 to 1.0)
    pub fn new(expected_keys: usize, false_positive_rate: f64) -> Self {
        let num_bits = Self::calculate_num_bits(expected_keys, false_positive_rate);
        let num_hash_functions = Self::calculate_num_hash_functions(expected_keys, num_bits);

        Self {
            bits: vec![0u8; (num_bits + 7) / 8], // Round up to byte boundary
            num_hash_functions,
            num_bits,
            num_keys: 0,
        }
    }

    /// Create bloom filter with specific parameters
    pub fn with_parameters(num_bits: usize, num_hash_functions: usize) -> Self {
        Self {
            bits: vec![0u8; (num_bits + 7) / 8],
            num_hash_functions,
            num_bits,
            num_keys: 0,
        }
    }

    /// Add a key to the bloom filter
    pub fn add(&mut self, key: &[u8]) {
        for i in 0..self.num_hash_functions {
            let hash = self.hash_key(key, i);
            let bit_index = hash % self.num_bits;
            self.set_bit(bit_index);
        }
        self.num_keys += 1;
    }

    /// Test if a key might be in the set
    pub fn contains(&self, key: &[u8]) -> bool {
        for i in 0..self.num_hash_functions {
            let hash = self.hash_key(key, i);
            let bit_index = hash % self.num_bits;
            if !self.get_bit(bit_index) {
                return false;
            }
        }
        true
    }

    /// Get the number of keys added
    pub fn num_keys(&self) -> usize {
        self.num_keys
    }

    /// Get the number of bits
    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    /// Get the number of hash functions
    pub fn num_hash_functions(&self) -> usize {
        self.num_hash_functions
    }

    /// Clear the bloom filter
    pub fn clear(&mut self) {
        self.bits.fill(0);
        self.num_keys = 0;
    }

    /// Estimate current false positive rate
    pub fn false_positive_rate(&self) -> f64 {
        if self.num_keys == 0 {
            return 0.0;
        }

        let bits_set = self.count_set_bits();
        let fraction_set = bits_set as f64 / self.num_bits as f64;
        fraction_set.powi(self.num_hash_functions as i32)
    }

    /// Get filter data for serialization
    pub fn data(&self) -> &[u8] {
        &self.bits
    }

    /// Create from existing data
    pub fn from_data(data: Vec<u8>, num_hash_functions: usize) -> Self {
        let num_bits = data.len() * 8;
        Self {
            bits: data,
            num_hash_functions,
            num_bits,
            num_keys: 0, // Can't determine from serialized data
        }
    }

    fn hash_key(&self, key: &[u8], hash_index: usize) -> usize {
        let seed = hash_index as u32;
        murmur3_hash_32(key, seed) as usize
    }

    fn set_bit(&mut self, bit_index: usize) {
        let byte_index = bit_index / 8;
        let bit_offset = bit_index % 8;
        if byte_index < self.bits.len() {
            self.bits[byte_index] |= 1 << bit_offset;
        }
    }

    fn get_bit(&self, bit_index: usize) -> bool {
        let byte_index = bit_index / 8;
        let bit_offset = bit_index % 8;
        if byte_index < self.bits.len() {
            (self.bits[byte_index] & (1 << bit_offset)) != 0
        } else {
            false
        }
    }

    fn count_set_bits(&self) -> usize {
        self.bits.iter().map(|&byte| byte.count_ones() as usize).sum()
    }

    fn calculate_num_bits(expected_keys: usize, false_positive_rate: f64) -> usize {
        if expected_keys == 0 || false_positive_rate <= 0.0 || false_positive_rate >= 1.0 {
            return 1024; // Default size
        }

        let ln2 = std::f64::consts::LN_2;
        let num_bits = -(expected_keys as f64 * false_positive_rate.ln()) / (ln2 * ln2);
        (num_bits.ceil() as usize).max(1)
    }

    fn calculate_num_hash_functions(expected_keys: usize, num_bits: usize) -> usize {
        if expected_keys == 0 {
            return 1;
        }

        let ln2 = std::f64::consts::LN_2;
        let num_funcs = (num_bits as f64 / expected_keys as f64) * ln2;
        (num_funcs.round() as usize).max(1).min(10) // Cap at 10 hash functions
    }
}

/// Bloom filter builder for easy construction
pub struct BloomFilterBuilder {
    expected_keys: Option<usize>,
    false_positive_rate: Option<f64>,
    num_bits: Option<usize>,
    num_hash_functions: Option<usize>,
}

impl BloomFilterBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            expected_keys: None,
            false_positive_rate: None,
            num_bits: None,
            num_hash_functions: None,
        }
    }

    /// Set expected number of keys
    pub fn expected_keys(mut self, keys: usize) -> Self {
        self.expected_keys = Some(keys);
        self
    }

    /// Set desired false positive rate
    pub fn false_positive_rate(mut self, rate: f64) -> Self {
        self.false_positive_rate = Some(rate);
        self
    }

    /// Set number of bits directly
    pub fn num_bits(mut self, bits: usize) -> Self {
        self.num_bits = Some(bits);
        self
    }

    /// Set number of hash functions directly
    pub fn num_hash_functions(mut self, funcs: usize) -> Self {
        self.num_hash_functions = Some(funcs);
        self
    }

    /// Build the bloom filter
    pub fn build(self) -> BloomFilter {
        match (self.num_bits, self.num_hash_functions) {
            (Some(bits), Some(funcs)) => {
                BloomFilter::with_parameters(bits, funcs)
            }
            _ => {
                let expected_keys = self.expected_keys.unwrap_or(1000);
                let fpr = self.false_positive_rate.unwrap_or(0.01);
                BloomFilter::new(expected_keys, fpr)
            }
        }
    }
}

impl Default for BloomFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic() {
        let mut filter = BloomFilter::new(100, 0.01);

        // Add some keys
        filter.add(b"key1");
        filter.add(b"key2");
        filter.add(b"key3");

        // Test contains
        assert!(filter.contains(b"key1"));
        assert!(filter.contains(b"key2"));
        assert!(filter.contains(b"key3"));

        // Key not added should likely return false
        assert!(!filter.contains(b"key4"));
    }

    #[test]
    fn test_bloom_filter_false_positives() {
        let mut filter = BloomFilter::new(10, 0.5); // High false positive rate

        // Add a few keys
        for i in 0..5 {
            let key = format!("key{}", i);
            filter.add(key.as_bytes());
        }

        // Check that added keys are found
        for i in 0..5 {
            let key = format!("key{}", i);
            assert!(filter.contains(key.as_bytes()));
        }

        // Some non-added keys might return true (false positives)
        let mut false_positives = 0;
        for i in 100..200 {
            let key = format!("key{}", i);
            if filter.contains(key.as_bytes()) {
                false_positives += 1;
            }
        }

        // With 50% FPR and small filter, we expect some false positives
        println!("False positives: {} out of 100", false_positives);
    }

    #[test]
    fn test_bloom_filter_clear() {
        let mut filter = BloomFilter::new(100, 0.01);

        filter.add(b"test_key");
        assert!(filter.contains(b"test_key"));
        assert_eq!(filter.num_keys(), 1);

        filter.clear();
        assert!(!filter.contains(b"test_key"));
        assert_eq!(filter.num_keys(), 0);
    }

    #[test]
    fn test_bloom_filter_serialization() {
        let mut filter = BloomFilter::new(50, 0.02);
        filter.add(b"serialize_key");

        let data = filter.data().to_vec();
        let restored_filter = BloomFilter::from_data(data, filter.num_hash_functions());

        assert!(restored_filter.contains(b"serialize_key"));
    }

    #[test]
    fn test_bloom_filter_builder() {
        let filter = BloomFilterBuilder::new()
            .expected_keys(500)
            .false_positive_rate(0.05)
            .build();

        assert_eq!(filter.num_keys(), 0);
        assert!(filter.num_bits() > 0);
        assert!(filter.num_hash_functions() > 0);
    }

    #[test]
    fn test_bloom_filter_builder_direct_params() {
        let filter = BloomFilterBuilder::new()
            .num_bits(1024)
            .num_hash_functions(3)
            .build();

        assert_eq!(filter.num_bits(), 1024);
        assert_eq!(filter.num_hash_functions(), 3);
    }

    #[test]
    fn test_empty_bloom_filter() {
        let filter = BloomFilter::new(0, 0.01);
        assert!(!filter.contains(b"any_key"));
        assert_eq!(filter.false_positive_rate(), 0.0);
    }
}
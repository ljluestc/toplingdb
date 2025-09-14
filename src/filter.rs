//! Filter policies for reducing disk reads

use crate::slice::Slice;

/// Filter policy interface
pub trait FilterPolicy: Send + Sync + std::fmt::Debug {
    /// Create a filter for the given keys
    fn create_filter(&self, keys: &[&[u8]]) -> Vec<u8>;

    /// Check if the key might be present in the filter
    fn key_may_match(&self, key: &[u8], filter: &[u8]) -> bool;

    /// Name of the filter policy
    fn name(&self) -> &str;
}

/// Bloom filter implementation
#[derive(Debug)]
pub struct BloomFilter {
    bits_per_key: u32,
    hash_count: u32,
}

impl BloomFilter {
    /// Create a new bloom filter with the specified bits per key
    pub fn new(bits_per_key: u32) -> Self {
        // Calculate optimal number of hash functions
        let hash_count = ((bits_per_key as f64) * 0.69) as u32; // ln(2) ≈ 0.69
        let hash_count = hash_count.clamp(1, 30);

        Self {
            bits_per_key,
            hash_count,
        }
    }

    fn hash(&self, key: &[u8]) -> u32 {
        // Simple hash function (should use better hash in production)
        let mut h = 0u32;
        for &b in key {
            h = h.wrapping_mul(31).wrapping_add(u32::from(b));
        }
        h
    }

    fn hash2(&self, h: u32) -> u32 {
        (h >> 17) | (h << 15)
    }
}

impl FilterPolicy for BloomFilter {
    fn create_filter(&self, keys: &[&[u8]]) -> Vec<u8> {
        if keys.is_empty() {
            return vec![0]; // Empty filter
        }

        let bits = (keys.len() as u32 * self.bits_per_key).max(64);
        let bytes = (bits + 7) / 8;
        let bits = bytes * 8; // Round up to byte boundary

        let mut filter = vec![0u8; bytes as usize + 1];
        filter[bytes as usize] = self.hash_count as u8; // Store hash count

        for key in keys {
            let mut h = self.hash(key);
            let delta = self.hash2(h);

            for _ in 0..self.hash_count {
                let bit_pos = (h % bits) as usize;
                filter[bit_pos / 8] |= 1 << (bit_pos % 8);
                h = h.wrapping_add(delta);
            }
        }

        filter
    }

    fn key_may_match(&self, key: &[u8], filter: &[u8]) -> bool {
        if filter.len() <= 1 {
            return false; // Empty filter
        }

        let bits = ((filter.len() - 1) * 8) as u32;
        let hash_count = filter[filter.len() - 1] as u32;

        if hash_count > 30 {
            return true; // Corrupted filter data
        }

        let mut h = self.hash(key);
        let delta = self.hash2(h);

        for _ in 0..hash_count {
            let bit_pos = (h % bits) as usize;
            if (filter[bit_pos / 8] & (1 << (bit_pos % 8))) == 0 {
                return false;
            }
            h = h.wrapping_add(delta);
        }

        true
    }

    fn name(&self) -> &str {
        "rocksdb.BloomFilter"
    }
}

/// No-op filter policy for testing
#[derive(Debug)]
pub struct NoFilter;

impl FilterPolicy for NoFilter {
    fn create_filter(&self, _keys: &[&[u8]]) -> Vec<u8> {
        Vec::new()
    }

    fn key_may_match(&self, _key: &[u8], _filter: &[u8]) -> bool {
        true
    }

    fn name(&self) -> &str {
        "NoFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter() {
        let filter = BloomFilter::new(10);
        let keys: Vec<&[u8]> = vec![b"hello", b"world", b"test"];
        let filter_data = filter.create_filter(&keys);

        // Keys that were added should match
        assert!(filter.key_may_match(b"hello", &filter_data));
        assert!(filter.key_may_match(b"world", &filter_data));
        assert!(filter.key_may_match(b"test", &filter_data));

        // Keys that weren't added might not match (though false positives are possible)
        // We can't guarantee this will be false due to the nature of bloom filters
    }

    #[test]
    fn test_no_filter() {
        let filter = NoFilter;
        let keys: Vec<&[u8]> = vec![b"hello", b"world"];
        let filter_data = filter.create_filter(&keys);

        assert!(filter_data.is_empty());
        assert!(filter.key_may_match(b"anything", &filter_data));
    }

    #[test]
    fn test_empty_filter() {
        let filter = BloomFilter::new(10);
        let filter_data = filter.create_filter(&[]);

        assert!(!filter.key_may_match(b"test", &filter_data));
    }
}
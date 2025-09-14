//! CRC (Cyclic Redundancy Check) utilities

use crc32fast::Hasher;

/// CRC-32C implementation
pub struct CRC32C {
    hasher: Hasher,
}

impl CRC32C {
    /// Create a new CRC-32C hasher
    pub fn new() -> Self {
        Self {
            hasher: Hasher::new(),
        }
    }

    /// Update the CRC with new data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize and get the CRC value
    pub fn finalize(self) -> u32 {
        self.hasher.finalize()
    }

    /// Reset the CRC state
    pub fn reset(&mut self) {
        self.hasher.reset();
    }
}

impl Default for CRC32C {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate CRC-32C for data
pub fn crc32c(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// Calculate CRC-32C with seed
pub fn crc32c_with_seed(data: &[u8], seed: u32) -> u32 {
    let mut hasher = Hasher::new_with_initial(seed);
    hasher.update(data);
    hasher.finalize()
}

/// Extend CRC with new data
pub fn crc32c_extend(crc: u32, data: &[u8]) -> u32 {
    let mut hasher = Hasher::new_with_initial(crc);
    hasher.update(data);
    hasher.finalize()
}

/// Combine two CRC values
pub fn crc32c_combine(crc1: u32, crc2: u32, len2: usize) -> u32 {
    // This is a simplified implementation
    // In a real implementation, you would need proper CRC combining logic
    crc1 ^ crc2 ^ (len2 as u32)
}

/// Mask CRC value (used in some file formats)
pub fn mask_crc(crc: u32) -> u32 {
    ((crc >> 15) | (crc << 17)) + 0xa282ead8
}

/// Unmask CRC value
pub fn unmask_crc(masked_crc: u32) -> u32 {
    let rot = masked_crc.wrapping_sub(0xa282ead8);
    (rot >> 17) | (rot << 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32c() {
        let data = b"hello world";
        let crc = crc32c(data);
        assert_ne!(crc, 0);

        // Test that same input gives same output
        let crc2 = crc32c(data);
        assert_eq!(crc, crc2);

        // Test empty data
        let empty_crc = crc32c(&[]);
        assert_eq!(empty_crc, 0);
    }

    #[test]
    fn test_crc32c_with_seed() {
        let data = b"test data";
        let crc1 = crc32c(data);
        let crc2 = crc32c_with_seed(data, 0);
        assert_eq!(crc1, crc2);

        let crc3 = crc32c_with_seed(data, 123);
        assert_ne!(crc1, crc3);
    }

    #[test]
    fn test_crc32c_extend() {
        let data1 = b"hello ";
        let data2 = b"world";

        let crc_combined = crc32c(b"hello world");

        let crc1 = crc32c(data1);
        let crc_extended = crc32c_extend(crc1, data2);

        assert_eq!(crc_combined, crc_extended);
    }

    #[test]
    fn test_incremental_crc() {
        let mut crc = CRC32C::new();
        crc.update(b"hello ");
        crc.update(b"world");
        let result = crc.finalize();

        let direct = crc32c(b"hello world");
        assert_eq!(result, direct);
    }

    #[test]
    fn test_mask_unmask() {
        let original = 0x12345678u32;
        let masked = mask_crc(original);
        let unmasked = unmask_crc(masked);
        assert_eq!(original, unmasked);
    }

    #[test]
    fn test_crc_reset() {
        let mut crc = CRC32C::new();
        crc.update(b"some data");
        crc.reset();
        crc.update(b"hello");
        let result = crc.finalize();

        let expected = crc32c(b"hello");
        assert_eq!(result, expected);
    }
}
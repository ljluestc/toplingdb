//! Slice module for efficient byte array handling
//!
//! A Slice is a simple structure containing a pointer into some external
//! storage and a size. It is similar to Go's slice type but for bytes.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, Range};

/// A Slice points to a contiguous range of bytes
#[derive(Clone)]
pub struct Slice<'a> {
    data: &'a [u8],
}

impl<'a> Slice<'a> {
    /// Create a new empty Slice
    pub fn new() -> Self {
        Self { data: &[] }
    }

    /// Create a Slice from a byte array
    pub fn from_bytes(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Create a Slice from a string
    pub fn from_str(s: &'a str) -> Self {
        Self { data: s.as_bytes() }
    }

    /// Get the length of the slice
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the slice is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the underlying data as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.data
    }

    /// Get the underlying data as a byte slice
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// Get a specific byte at the given index
    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    /// Get the byte at the given index without bounds checking
    ///
    /// # Safety
    /// The caller must ensure that `index` is less than `self.len()`
    pub unsafe fn get_unchecked(&self, index: usize) -> u8 {
        *self.data.get_unchecked(index)
    }

    /// Create a new Slice that is a substring of this slice
    pub fn slice(&self, range: Range<usize>) -> Self {
        Self {
            data: &self.data[range],
        }
    }

    /// Remove the first `n` bytes from this slice
    pub fn remove_prefix(&mut self, n: usize) {
        if n <= self.data.len() {
            self.data = &self.data[n..];
        } else {
            self.data = &[];
        }
    }

    /// Remove the last `n` bytes from this slice
    pub fn remove_suffix(&mut self, n: usize) {
        if n <= self.data.len() {
            let new_len = self.data.len() - n;
            self.data = &self.data[..new_len];
        } else {
            self.data = &[];
        }
    }

    /// Compare this slice with another slice
    pub fn compare(&self, other: &Slice) -> Ordering {
        self.data.cmp(other.data)
    }

    /// Check if this slice starts with the given prefix
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.data.starts_with(prefix)
    }

    /// Check if this slice ends with the given suffix
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        self.data.ends_with(suffix)
    }

    /// Convert to a string if the slice contains valid UTF-8
    pub fn to_string_lossy(&self) -> std::borrow::Cow<str> {
        String::from_utf8_lossy(self.data)
    }

    /// Try to convert to a string
    pub fn to_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data.to_vec())
    }

    /// Convert to a vector
    pub fn to_vec(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    /// Create an iterator over the bytes
    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.data.iter()
    }

    /// Clear the slice
    pub fn clear(&mut self) {
        self.data = &[];
    }

    /// Check if two slices are equal
    pub fn equals(&self, other: &Slice) -> bool {
        self.data == other.data
    }
}

impl<'a> Default for Slice<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a [u8]> for Slice<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self::from_bytes(data)
    }
}

impl<'a> From<&'a str> for Slice<'a> {
    fn from(s: &'a str) -> Self {
        Self::from_str(s)
    }
}

impl<'a> From<&'a Vec<u8>> for Slice<'a> {
    fn from(vec: &'a Vec<u8>) -> Self {
        Self::from_bytes(vec.as_slice())
    }
}

impl<'a> From<&'a String> for Slice<'a> {
    fn from(s: &'a String) -> Self {
        Self::from_str(s.as_str())
    }
}

impl<'a> Deref for Slice<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> AsRef<[u8]> for Slice<'a> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'a> PartialEq for Slice<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<'a> Eq for Slice<'a> {}

impl<'a> PartialOrd for Slice<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Slice<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(other.data)
    }
}

impl<'a> Hash for Slice<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<'a> fmt::Debug for Slice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Slice({:?})", self.data)
    }
}

impl<'a> fmt::Display for Slice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}

/// Owned version of Slice for cases where we need to own the data
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnedSlice {
    data: Vec<u8>,
}

impl OwnedSlice {
    /// Create a new empty OwnedSlice
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create an OwnedSlice from a vector
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Create an OwnedSlice from a byte slice
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Create an OwnedSlice from a string
    pub fn from_string(s: String) -> Self {
        Self {
            data: s.into_bytes(),
        }
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get as a slice
    pub fn as_slice(&self) -> Slice {
        Slice::from_bytes(&self.data)
    }

    /// Get the underlying data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Take ownership of the underlying vector
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    /// Clear the contents
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Append bytes
    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}

impl Default for OwnedSlice {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<u8>> for OwnedSlice {
    fn from(data: Vec<u8>) -> Self {
        Self::from_vec(data)
    }
}

impl From<String> for OwnedSlice {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&[u8]> for OwnedSlice {
    fn from(data: &[u8]) -> Self {
        Self::from_bytes(data)
    }
}

impl From<&str> for OwnedSlice {
    fn from(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }
}

impl AsRef<[u8]> for OwnedSlice {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Deref for OwnedSlice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_basic() {
        let data = b"hello world";
        let slice = Slice::from_bytes(data);
        assert_eq!(slice.len(), 11);
        assert!(!slice.is_empty());
        assert_eq!(slice.as_bytes(), data);
    }

    #[test]
    fn test_slice_comparison() {
        let slice1 = Slice::from_str("abc");
        let slice2 = Slice::from_str("abc");
        let slice3 = Slice::from_str("def");

        assert_eq!(slice1, slice2);
        assert!(slice1 < slice3);
        assert_eq!(slice1.compare(&slice2), Ordering::Equal);
        assert_eq!(slice1.compare(&slice3), Ordering::Less);
    }

    #[test]
    fn test_slice_prefix_suffix() {
        let mut slice = Slice::from_str("hello world");
        assert!(slice.starts_with(b"hello"));
        assert!(slice.ends_with(b"world"));

        slice.remove_prefix(6);
        assert_eq!(slice.as_bytes(), b"world");

        slice.remove_suffix(2);
        assert_eq!(slice.as_bytes(), b"wor");
    }

    #[test]
    fn test_owned_slice() {
        let owned = OwnedSlice::from_string("test".to_string());
        assert_eq!(owned.len(), 4);
        assert_eq!(owned.as_bytes(), b"test");

        let slice = owned.as_slice();
        assert_eq!(slice.as_bytes(), b"test");
    }

    #[test]
    fn test_slice_substring() {
        let slice = Slice::from_str("hello world");
        let sub = slice.slice(6..11);
        assert_eq!(sub.as_bytes(), b"world");
    }
}
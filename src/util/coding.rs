//! Binary encoding and decoding utilities

/// Encode a fixed 32-bit value in little-endian format
pub fn encode_fixed_32(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Decode a fixed 32-bit value from little-endian format
pub fn decode_fixed_32(data: &[u8]) -> u32 {
    assert!(data.len() >= 4);
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// Encode a fixed 64-bit value in little-endian format
pub fn encode_fixed_64(value: u64) -> [u8; 8] {
    value.to_le_bytes()
}

/// Decode a fixed 64-bit value from little-endian format
pub fn decode_fixed_64(data: &[u8]) -> u64 {
    assert!(data.len() >= 8);
    u64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ])
}

/// Encode a varint (variable-length integer)
pub fn encode_varint_32(value: u32, buf: &mut Vec<u8>) {
    let mut v = value;
    while v >= 0x80 {
        buf.push((v & 0x7F) as u8 | 0x80);
        v >>= 7;
    }
    buf.push(v as u8);
}

/// Encode a varint (variable-length integer) for 64-bit values
pub fn encode_varint_64(value: u64, buf: &mut Vec<u8>) {
    let mut v = value;
    while v >= 0x80 {
        buf.push((v & 0x7F) as u8 | 0x80);
        v >>= 7;
    }
    buf.push(v as u8);
}

/// Decode a 32-bit varint from buffer
pub fn decode_varint_32(data: &[u8]) -> Option<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;

    for (i, &byte) in data.iter().enumerate() {
        if shift >= 32 {
            return None; // Overflow
        }

        result |= ((byte & 0x7F) as u32) << shift;

        if (byte & 0x80) == 0 {
            return Some((result, i + 1));
        }

        shift += 7;
    }

    None // Incomplete varint
}

/// Decode a 64-bit varint from buffer
pub fn decode_varint_64(data: &[u8]) -> Option<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0;

    for (i, &byte) in data.iter().enumerate() {
        if shift >= 64 {
            return None; // Overflow
        }

        result |= ((byte & 0x7F) as u64) << shift;

        if (byte & 0x80) == 0 {
            return Some((result, i + 1));
        }

        shift += 7;
    }

    None // Incomplete varint
}

/// Encode a length-prefixed string
pub fn encode_string(s: &str, buf: &mut Vec<u8>) {
    encode_varint_32(s.len() as u32, buf);
    buf.extend_from_slice(s.as_bytes());
}

/// Encode length-prefixed bytes
pub fn encode_bytes(bytes: &[u8], buf: &mut Vec<u8>) {
    encode_varint_32(bytes.len() as u32, buf);
    buf.extend_from_slice(bytes);
}

/// Decode a length-prefixed string
pub fn decode_string(data: &[u8]) -> Option<(String, usize)> {
    let (len, len_bytes) = decode_varint_32(data)?;
    let len = len as usize;

    if data.len() < len_bytes + len {
        return None;
    }

    let string_data = &data[len_bytes..len_bytes + len];
    match String::from_utf8(string_data.to_vec()) {
        Ok(s) => Some((s, len_bytes + len)),
        Err(_) => None,
    }
}

/// Decode length-prefixed bytes
pub fn decode_bytes(data: &[u8]) -> Option<(Vec<u8>, usize)> {
    let (len, len_bytes) = decode_varint_32(data)?;
    let len = len as usize;

    if data.len() < len_bytes + len {
        return None;
    }

    let bytes = data[len_bytes..len_bytes + len].to_vec();
    Some((bytes, len_bytes + len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_32() {
        let value = 0x12345678u32;
        let encoded = encode_fixed_32(value);
        let decoded = decode_fixed_32(&encoded);
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_fixed_64() {
        let value = 0x123456789ABCDEFu64;
        let encoded = encode_fixed_64(value);
        let decoded = decode_fixed_64(&encoded);
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_varint_32() {
        let test_values = [0u32, 127, 128, 16383, 16384, u32::MAX];

        for &value in &test_values {
            let mut buf = Vec::new();
            encode_varint_32(value, &mut buf);
            let (decoded, bytes_read) = decode_varint_32(&buf).unwrap();
            assert_eq!(value, decoded);
            assert_eq!(buf.len(), bytes_read);
        }
    }

    #[test]
    fn test_varint_64() {
        let test_values = [0u64, 127, 128, 16383, 16384, u64::MAX];

        for &value in &test_values {
            let mut buf = Vec::new();
            encode_varint_64(value, &mut buf);
            let (decoded, bytes_read) = decode_varint_64(&buf).unwrap();
            assert_eq!(value, decoded);
            assert_eq!(buf.len(), bytes_read);
        }
    }

    #[test]
    fn test_string_encoding() {
        let test_strings = ["", "hello", "world", "🦀 rust"];

        for &s in &test_strings {
            let mut buf = Vec::new();
            encode_string(s, &mut buf);
            let (decoded, bytes_read) = decode_string(&buf).unwrap();
            assert_eq!(s, decoded);
            assert_eq!(buf.len(), bytes_read);
        }
    }

    #[test]
    fn test_bytes_encoding() {
        let test_data = [
            vec![],
            vec![0, 1, 2, 3],
            vec![255; 1000],
        ];

        for data in &test_data {
            let mut buf = Vec::new();
            encode_bytes(data, &mut buf);
            let (decoded, bytes_read) = decode_bytes(&buf).unwrap();
            assert_eq!(data, &decoded);
            assert_eq!(buf.len(), bytes_read);
        }
    }
}
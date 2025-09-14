//! Merge operators for combining values

use crate::slice::Slice;
use crate::status::Status;

/// Merge operator interface
pub trait MergeOperator: Send + Sync + std::fmt::Debug {
    /// Merge operation
    fn merge(
        &self,
        key: &[u8],
        existing_value: Option<&[u8]>,
        operands: &[&[u8]],
    ) -> Result<Vec<u8>, Status>;

    /// Partial merge for optimization
    fn partial_merge(
        &self,
        key: &[u8],
        left_operand: &[u8],
        right_operand: &[u8],
    ) -> Result<Option<Vec<u8>>, Status> {
        // Default implementation: no partial merge
        Ok(None)
    }

    /// Name of the merge operator
    fn name(&self) -> &str;
}

/// String append merge operator
#[derive(Debug)]
pub struct StringAppendOperator {
    delimiter: String,
}

impl StringAppendOperator {
    pub fn new(delimiter: String) -> Self {
        Self { delimiter }
    }
}

impl MergeOperator for StringAppendOperator {
    fn merge(
        &self,
        _key: &[u8],
        existing_value: Option<&[u8]>,
        operands: &[&[u8]],
    ) -> Result<Vec<u8>, Status> {
        let mut result = if let Some(existing) = existing_value {
            String::from_utf8_lossy(existing).into_owned()
        } else {
            String::new()
        };

        for operand in operands {
            if !result.is_empty() {
                result.push_str(&self.delimiter);
            }
            result.push_str(&String::from_utf8_lossy(operand));
        }

        Ok(result.into_bytes())
    }

    fn name(&self) -> &str {
        "StringAppendOperator"
    }
}

/// Counter merge operator (adds numbers)
#[derive(Debug)]
pub struct CounterOperator;

impl MergeOperator for CounterOperator {
    fn merge(
        &self,
        _key: &[u8],
        existing_value: Option<&[u8]>,
        operands: &[&[u8]],
    ) -> Result<Vec<u8>, Status> {
        let mut counter = if let Some(existing) = existing_value {
            if existing.len() != 8 {
                return Err(Status::invalid_argument(Some("Invalid counter value".to_string())));
            }
            i64::from_le_bytes([
                existing[0], existing[1], existing[2], existing[3],
                existing[4], existing[5], existing[6], existing[7],
            ])
        } else {
            0
        };

        for operand in operands {
            if operand.len() != 8 {
                return Err(Status::invalid_argument(Some("Invalid operand".to_string())));
            }
            let delta = i64::from_le_bytes([
                operand[0], operand[1], operand[2], operand[3],
                operand[4], operand[5], operand[6], operand[7],
            ]);
            counter = counter.wrapping_add(delta);
        }

        Ok(counter.to_le_bytes().to_vec())
    }

    fn name(&self) -> &str {
        "CounterOperator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_append_operator() {
        let op = StringAppendOperator::new(",".to_string());

        // Test merge with existing value
        let result = op.merge(b"key", Some(b"hello"), &[b"world"]).unwrap();
        assert_eq!(result, b"hello,world");

        // Test merge without existing value
        let result = op.merge(b"key", None, &[b"hello", b"world"]).unwrap();
        assert_eq!(result, b"hello,world");

        // Test with empty operands
        let result = op.merge(b"key", Some(b"hello"), &[]).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_counter_operator() {
        let op = CounterOperator;

        // Test with existing value
        let existing = 10i64.to_le_bytes();
        let operand1 = 5i64.to_le_bytes();
        let operand2 = (-3i64).to_le_bytes();

        let result = op.merge(b"key", Some(&existing), &[&operand1, &operand2]).unwrap();
        let final_count = i64::from_le_bytes([
            result[0], result[1], result[2], result[3],
            result[4], result[5], result[6], result[7],
        ]);
        assert_eq!(final_count, 12); // 10 + 5 + (-3)

        // Test without existing value
        let result = op.merge(b"key", None, &[&operand1]).unwrap();
        let final_count = i64::from_le_bytes([
            result[0], result[1], result[2], result[3],
            result[4], result[5], result[6], result[7],
        ]);
        assert_eq!(final_count, 5);
    }

    #[test]
    fn test_counter_operator_invalid_input() {
        let op = CounterOperator;

        // Test with invalid existing value
        let result = op.merge(b"key", Some(b"invalid"), &[]);
        assert!(result.is_err());

        // Test with invalid operand
        let existing = 10i64.to_le_bytes();
        let result = op.merge(b"key", Some(&existing), &[b"invalid"]);
        assert!(result.is_err());
    }
}
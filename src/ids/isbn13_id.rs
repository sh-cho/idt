use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment, ValidationResult,
};
use crate::utils::check_digit::{
    compute_isbn10_check, parse_digits, strip_formatting, validate_mod10,
};
use serde_json::json;

/// Parsed ISBN-13 value
pub struct ParsedIsbn13 {
    digits: Vec<u8>,
    input: String,
}

impl ParsedIsbn13 {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed);

        let digits = parse_digits(&cleaned)
            .ok_or_else(|| IdtError::ParseError("ISBN-13 must contain only digits".to_string()))?;

        if digits.len() != 13 {
            return Err(IdtError::ParseError(format!(
                "ISBN-13 must be exactly 13 digits, got {}",
                digits.len()
            )));
        }

        // Must start with 978 or 979
        let prefix = digits[0] as u32 * 100 + digits[1] as u32 * 10 + digits[2] as u32;
        if prefix != 978 && prefix != 979 {
            return Err(IdtError::ParseError(
                "ISBN-13 must start with 978 or 979".to_string(),
            ));
        }

        if !validate_mod10(&digits) {
            return Err(IdtError::ParseError(
                "ISBN-13 check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            digits,
            input: input_trimmed.to_string(),
        })
    }

    /// Convert to ISBN-10 if prefix is 978.
    /// Returns None if prefix is 979 (no ISBN-10 equivalent).
    pub fn to_isbn10(&self) -> Option<String> {
        if self.digits[0..3] != [9, 7, 8] {
            return None;
        }
        // Take digits 3..12 (9 digits), compute ISBN-10 check digit
        let payload = &self.digits[3..12];
        let check = compute_isbn10_check(payload);
        let mut result: String = payload.iter().map(|d| (b'0' + d) as char).collect();
        result.push(check);
        Some(result)
    }

    fn prefix(&self) -> String {
        self.digits[0..3]
            .iter()
            .map(|d| (b'0' + d) as char)
            .collect()
    }
}

impl ParsedId for ParsedIsbn13 {
    fn kind(&self) -> IdKind {
        IdKind::Isbn13
    }

    fn canonical(&self) -> String {
        self.digits.iter().map(|d| (b'0' + d) as char).collect()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.digits.clone()
    }

    fn timestamp(&self) -> Option<crate::core::id::Timestamp> {
        None
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let canonical = self.canonical();

        let mut components = json!({
            "prefix": self.prefix(),
            "check_digit": self.digits[12].to_string(),
        });

        if let Some(isbn10) = self.to_isbn10() {
            components["isbn10"] = json!(isbn10);
        }

        InspectionResult {
            id_type: "isbn13".to_string(),
            input: self.input.clone(),
            canonical: canonical.clone(),
            valid: true,
            timestamp: None,
            timestamp_iso: None,
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: None,
            components: Some(components),
            structure: Some(vec![
                StructureSegment {
                    name: "GS1 Prefix".to_string(),
                    size: 3,
                    unit: SizeUnit::Digits,
                    value: Some(self.prefix()),
                    description: "GS1 Bookland prefix (978 or 979)".to_string(),
                },
                StructureSegment {
                    name: "Registration Group".to_string(),
                    size: 9,
                    unit: SizeUnit::Digits,
                    value: Some(
                        self.digits[3..12]
                            .iter()
                            .map(|d| (b'0' + d) as char)
                            .collect(),
                    ),
                    description: "Registration group, registrant, and publication elements"
                        .to_string(),
                },
                StructureSegment {
                    name: "Check Digit".to_string(),
                    size: 1,
                    unit: SizeUnit::Digits,
                    value: Some(self.digits[12].to_string()),
                    description: "Mod-10 check digit".to_string(),
                },
            ]),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: encode_base32(&bytes),
                base58: encode_base58(&bytes),
                base64: encode_base64(&bytes),
                int: None,
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        ValidationResult::valid("isbn13")
    }

    fn encode(&self, format: EncodingFormat) -> String {
        let bytes = self.as_bytes();
        match format {
            EncodingFormat::Canonical => self.canonical(),
            EncodingFormat::Hex => encode_hex(&bytes),
            EncodingFormat::HexUpper => encode_hex_upper(&bytes),
            EncodingFormat::Base32 => encode_base32(&bytes),
            EncodingFormat::Base32Hex => encode_base32(&bytes),
            EncodingFormat::Base58 => encode_base58(&bytes),
            EncodingFormat::Base64 => encode_base64(&bytes),
            EncodingFormat::Base64Url => encode_base64_url(&bytes),
            EncodingFormat::Binary => String::from_utf8_lossy(&bytes).to_string(),
            EncodingFormat::Bits => encode_bits(&bytes),
            EncodingFormat::Int => self.canonical(),
            EncodingFormat::Bytes => encode_bytes_spaced(&bytes),
        }
    }
}

/// Check if a string looks like an ISBN-13
pub fn is_isbn13(input: &str) -> bool {
    ParsedIsbn13::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedIsbn13::parse("9780306406157").unwrap();
        assert_eq!(parsed.canonical(), "9780306406157");
    }

    #[test]
    fn test_parse_with_hyphens() {
        let parsed = ParsedIsbn13::parse("978-0-306-40615-7").unwrap();
        assert_eq!(parsed.canonical(), "9780306406157");
    }

    #[test]
    fn test_parse_979_prefix() {
        // 979-10-90636-07-1
        let parsed = ParsedIsbn13::parse("9791090636071").unwrap();
        assert_eq!(parsed.prefix(), "979");
    }

    #[test]
    fn test_parse_invalid_prefix() {
        assert!(ParsedIsbn13::parse("1234567890128").is_err());
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedIsbn13::parse("9780306406158").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedIsbn13::parse("978030640615").is_err());
    }

    #[test]
    fn test_to_isbn10() {
        let parsed = ParsedIsbn13::parse("9780306406157").unwrap();
        assert_eq!(parsed.to_isbn10(), Some("0306406152".to_string()));
    }

    #[test]
    fn test_to_isbn10_with_x() {
        let parsed = ParsedIsbn13::parse("9780804429573").unwrap();
        assert_eq!(parsed.to_isbn10(), Some("080442957X".to_string()));
    }

    #[test]
    fn test_to_isbn10_979_prefix() {
        let parsed = ParsedIsbn13::parse("9791090636071").unwrap();
        assert_eq!(parsed.to_isbn10(), None);
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedIsbn13::parse("9780306406157").unwrap();
        assert_eq!(parsed.kind(), IdKind::Isbn13);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedIsbn13::parse("9780306406157").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "isbn13");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["prefix"], "978");
        assert_eq!(components["check_digit"], "7");
        assert_eq!(components["isbn10"], "0306406152");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedIsbn13::parse("9780306406157").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_is_isbn13() {
        assert!(is_isbn13("9780306406157"));
        assert!(is_isbn13("978-0-306-40615-7"));
        assert!(!is_isbn13("not-an-isbn"));
        assert!(!is_isbn13("9780306406158"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedIsbn13::parse("9780306406157").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "9780306406157");
        assert!(!parsed.encode(EncodingFormat::Hex).is_empty());
        assert!(!parsed.encode(EncodingFormat::HexUpper).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base32).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base32Hex).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base58).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base64).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base64Url).is_empty());
        assert!(!parsed.encode(EncodingFormat::Binary).is_empty());
        assert!(!parsed.encode(EncodingFormat::Bits).is_empty());
        assert!(!parsed.encode(EncodingFormat::Int).is_empty());
        assert!(!parsed.encode(EncodingFormat::Bytes).is_empty());
    }

    #[test]
    fn test_as_bytes() {
        let parsed = ParsedIsbn13::parse("9780306406157").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 13);
    }
}

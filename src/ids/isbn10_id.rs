use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment, ValidationResult,
};
use crate::utils::check_digit::{compute_mod10_check_digit, strip_formatting, validate_isbn10};
use serde_json::json;

/// Parsed ISBN-10 value
pub struct ParsedIsbn10 {
    /// The 10 characters (9 digits + check digit as char)
    chars: Vec<char>,
    input: String,
}

impl ParsedIsbn10 {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed);

        if cleaned.len() != 10 {
            return Err(IdtError::ParseError(format!(
                "ISBN-10 must be exactly 10 characters, got {}",
                cleaned.len()
            )));
        }

        let chars: Vec<char> = cleaned.chars().collect();

        // First 9 must be digits
        if !chars[..9].iter().all(|c| c.is_ascii_digit()) {
            return Err(IdtError::ParseError(
                "ISBN-10 first 9 characters must be digits".to_string(),
            ));
        }

        // Last can be digit or X/x
        if !chars[9].is_ascii_digit() && chars[9] != 'X' && chars[9] != 'x' {
            return Err(IdtError::ParseError(
                "ISBN-10 check digit must be 0-9 or X".to_string(),
            ));
        }

        if !validate_isbn10(&cleaned) {
            return Err(IdtError::ParseError(
                "ISBN-10 check digit is invalid".to_string(),
            ));
        }

        // Normalize X to uppercase
        let mut chars = chars;
        if chars[9] == 'x' {
            chars[9] = 'X';
        }

        Ok(Self {
            chars,
            input: input_trimmed.to_string(),
        })
    }

    /// Convert to ISBN-13 by prepending 978 and computing new check digit.
    pub fn to_isbn13(&self) -> String {
        let mut digits: Vec<u8> = vec![9, 7, 8];
        for &c in &self.chars[..9] {
            digits.push(c as u8 - b'0');
        }
        let check = compute_mod10_check_digit(&digits);
        digits.push(check);
        digits.iter().map(|d| (b'0' + d) as char).collect()
    }

    fn check_digit(&self) -> char {
        self.chars[9]
    }
}

impl ParsedId for ParsedIsbn10 {
    fn kind(&self) -> IdKind {
        IdKind::Isbn10
    }

    fn canonical(&self) -> String {
        self.chars.iter().collect()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.canonical().into_bytes()
    }

    fn timestamp(&self) -> Option<crate::core::id::Timestamp> {
        None
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let canonical = self.canonical();
        let isbn13 = self.to_isbn13();

        let components = json!({
            "check_digit": self.check_digit().to_string(),
            "isbn13": isbn13,
        });

        InspectionResult {
            id_type: "isbn10".to_string(),
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
                    name: "Body".to_string(),
                    size: 9,
                    unit: SizeUnit::Digits,
                    value: Some(self.chars[..9].iter().collect()),
                    description: "Registration group, registrant, and publication elements"
                        .to_string(),
                },
                StructureSegment {
                    name: "Check Digit".to_string(),
                    size: 1,
                    unit: SizeUnit::Chars,
                    value: Some(self.check_digit().to_string()),
                    description: "Mod-11 check digit (0-9 or X)".to_string(),
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
        ValidationResult::valid("isbn10")
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

/// Check if a string looks like an ISBN-10
pub fn is_isbn10(input: &str) -> bool {
    ParsedIsbn10::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedIsbn10::parse("0306406152").unwrap();
        assert_eq!(parsed.canonical(), "0306406152");
    }

    #[test]
    fn test_parse_with_x() {
        let parsed = ParsedIsbn10::parse("080442957X").unwrap();
        assert_eq!(parsed.canonical(), "080442957X");
        assert_eq!(parsed.check_digit(), 'X');
    }

    #[test]
    fn test_parse_with_lowercase_x() {
        let parsed = ParsedIsbn10::parse("080442957x").unwrap();
        assert_eq!(parsed.canonical(), "080442957X");
    }

    #[test]
    fn test_parse_with_hyphens() {
        let parsed = ParsedIsbn10::parse("0-306-40615-2").unwrap();
        assert_eq!(parsed.canonical(), "0306406152");
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedIsbn10::parse("0306406153").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedIsbn10::parse("030640615").is_err());
        assert!(ParsedIsbn10::parse("03064061522").is_err());
    }

    #[test]
    fn test_parse_non_digit() {
        assert!(ParsedIsbn10::parse("030640615a").is_err());
    }

    #[test]
    fn test_to_isbn13() {
        let parsed = ParsedIsbn10::parse("0306406152").unwrap();
        assert_eq!(parsed.to_isbn13(), "9780306406157");
    }

    #[test]
    fn test_to_isbn13_with_x() {
        let parsed = ParsedIsbn10::parse("080442957X").unwrap();
        assert_eq!(parsed.to_isbn13(), "9780804429573");
    }

    #[test]
    fn test_roundtrip_isbn10_isbn13() {
        // ISBN-10 -> ISBN-13 -> ISBN-10
        let isbn10 = ParsedIsbn10::parse("0306406152").unwrap();
        let isbn13_str = isbn10.to_isbn13();
        let isbn13 = crate::ids::isbn13_id::ParsedIsbn13::parse(&isbn13_str).unwrap();
        assert_eq!(isbn13.to_isbn10(), Some("0306406152".to_string()));
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedIsbn10::parse("0306406152").unwrap();
        assert_eq!(parsed.kind(), IdKind::Isbn10);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedIsbn10::parse("0306406152").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "isbn10");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["check_digit"], "2");
        assert_eq!(components["isbn13"], "9780306406157");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedIsbn10::parse("0306406152").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_is_isbn10() {
        assert!(is_isbn10("0306406152"));
        assert!(is_isbn10("080442957X"));
        assert!(!is_isbn10("not-isbn"));
        assert!(!is_isbn10("0306406153"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedIsbn10::parse("0306406152").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "0306406152");
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
        let parsed = ParsedIsbn10::parse("0306406152").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 10);
        assert_eq!(bytes, b"0306406152");
    }
}

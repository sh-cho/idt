use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment, ValidationResult,
};
use crate::utils::check_digit::strip_formatting;
use serde_json::json;

/// Parsed ASIN value
pub struct ParsedAsin {
    /// The 10-character ASIN (uppercase alphanumeric)
    value: String,
    input: String,
}

impl ParsedAsin {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed).to_uppercase();

        if cleaned.len() != 10 {
            return Err(IdtError::ParseError(format!(
                "ASIN must be exactly 10 characters, got {}",
                cleaned.len()
            )));
        }

        // All characters must be alphanumeric
        if !cleaned.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(IdtError::ParseError(
                "ASIN must contain only alphanumeric characters".to_string(),
            ));
        }

        // Must start with 'B' or a digit (legacy ASINs like ISBNs start with digit)
        let first = cleaned.chars().next().unwrap();
        if !first.is_ascii_digit() && first != 'B' {
            return Err(IdtError::ParseError(
                "ASIN must start with 'B' or a digit".to_string(),
            ));
        }

        Ok(Self {
            value: cleaned,
            input: input_trimmed.to_string(),
        })
    }
}

impl ParsedId for ParsedAsin {
    fn kind(&self) -> IdKind {
        IdKind::Asin
    }

    fn canonical(&self) -> String {
        self.value.clone()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.value.as_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<crate::core::id::Timestamp> {
        None
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();

        let is_isbn = self
            .value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit());

        let components = json!({
            "format": if is_isbn { "isbn-based" } else { "standard" },
        });

        InspectionResult {
            id_type: "asin".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: None,
            timestamp_iso: None,
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: None,
            components: Some(components),
            structure: Some(vec![StructureSegment {
                name: "Identifier".to_string(),
                size: 10,
                unit: SizeUnit::Chars,
                value: Some(self.value.clone()),
                description: if is_isbn {
                    "ISBN-10 based product identifier".to_string()
                } else {
                    "Amazon-assigned alphanumeric identifier (B0 prefix)".to_string()
                },
            }]),
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
        ValidationResult::valid("asin")
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

/// Check if a string looks like an ASIN
pub fn is_asin(input: &str) -> bool {
    ParsedAsin::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedAsin::parse("B08N5WRWNW").unwrap();
        assert_eq!(parsed.canonical(), "B08N5WRWNW");
    }

    #[test]
    fn test_parse_lowercase() {
        let parsed = ParsedAsin::parse("b08n5wrwnw").unwrap();
        assert_eq!(parsed.canonical(), "B08N5WRWNW");
    }

    #[test]
    fn test_parse_isbn_based() {
        let parsed = ParsedAsin::parse("0306406152").unwrap();
        assert_eq!(parsed.canonical(), "0306406152");
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedAsin::parse("B08N5WRWN").is_err());
        assert!(ParsedAsin::parse("B08N5WRWNWX").is_err());
    }

    #[test]
    fn test_parse_invalid_start() {
        assert!(ParsedAsin::parse("X08N5WRWNW").is_err());
    }

    #[test]
    fn test_parse_invalid_chars() {
        assert!(ParsedAsin::parse("B08N5WR!NW").is_err());
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedAsin::parse("B08N5WRWNW").unwrap();
        assert_eq!(parsed.kind(), IdKind::Asin);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedAsin::parse("B08N5WRWNW").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "asin");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["format"], "standard");
    }

    #[test]
    fn test_inspect_isbn_based() {
        let parsed = ParsedAsin::parse("0306406152").unwrap();
        let result = parsed.inspect();
        let components = result.components.unwrap();
        assert_eq!(components["format"], "isbn-based");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedAsin::parse("B08N5WRWNW").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_is_asin() {
        assert!(is_asin("B08N5WRWNW"));
        assert!(is_asin("0306406152"));
        assert!(!is_asin("not-an-asin"));
        assert!(!is_asin("X08N5WRWNW"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedAsin::parse("B08N5WRWNW").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "B08N5WRWNW");
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
        let parsed = ParsedAsin::parse("B08N5WRWNW").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 10);
        assert_eq!(bytes, b"B08N5WRWNW");
    }
}

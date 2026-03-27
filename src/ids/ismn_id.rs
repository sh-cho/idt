use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdEncodings, IdKind, InspectionResult, ParsedId, ValidationResult};
use crate::utils::check_digit::{parse_digits, strip_formatting, validate_mod10};
use serde_json::json;

/// Parsed ISMN value
pub struct ParsedIsmn {
    digits: Vec<u8>,
    input: String,
}

impl ParsedIsmn {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed);

        let digits = parse_digits(&cleaned)
            .ok_or_else(|| IdtError::ParseError("ISMN must contain only digits".to_string()))?;

        if digits.len() != 13 {
            return Err(IdtError::ParseError(format!(
                "ISMN must be exactly 13 digits, got {}",
                digits.len()
            )));
        }

        // Must start with 979-0
        if digits[0..4] != [9, 7, 9, 0] {
            return Err(IdtError::ParseError(
                "ISMN must start with 979-0".to_string(),
            ));
        }

        if !validate_mod10(&digits) {
            return Err(IdtError::ParseError(
                "ISMN check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            digits,
            input: input_trimmed.to_string(),
        })
    }

    fn prefix(&self) -> String {
        self.digits[0..4]
            .iter()
            .map(|d| (b'0' + d) as char)
            .collect()
    }
}

impl ParsedId for ParsedIsmn {
    fn kind(&self) -> IdKind {
        IdKind::Ismn
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

        let components = json!({
            "prefix": self.prefix(),
            "check_digit": self.digits[12].to_string(),
        });

        InspectionResult {
            id_type: "ismn".to_string(),
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
        ValidationResult::valid("ismn")
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

/// Check if a string looks like an ISMN
pub fn is_ismn(input: &str) -> bool {
    ParsedIsmn::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedIsmn::parse("9790060115615").unwrap();
        assert_eq!(parsed.canonical(), "9790060115615");
    }

    #[test]
    fn test_parse_with_hyphens() {
        let parsed = ParsedIsmn::parse("979-0-060-11561-5").unwrap();
        assert_eq!(parsed.canonical(), "9790060115615");
    }

    #[test]
    fn test_parse_invalid_prefix() {
        assert!(ParsedIsmn::parse("9780306406157").is_err()); // ISBN-13, not ISMN
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedIsmn::parse("9790060115616").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedIsmn::parse("979006011561").is_err());
    }

    #[test]
    fn test_prefix() {
        let parsed = ParsedIsmn::parse("9790060115615").unwrap();
        assert_eq!(parsed.prefix(), "9790");
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedIsmn::parse("9790060115615").unwrap();
        assert_eq!(parsed.kind(), IdKind::Ismn);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedIsmn::parse("9790060115615").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "ismn");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["prefix"], "9790");
        assert_eq!(components["check_digit"], "5");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedIsmn::parse("9790060115615").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_is_ismn() {
        assert!(is_ismn("9790060115615"));
        assert!(is_ismn("979-0-060-11561-5"));
        assert!(!is_ismn("not-an-ismn"));
        assert!(!is_ismn("9790060115616"));
    }
}

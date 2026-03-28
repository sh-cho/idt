use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment, ValidationResult,
};
use crate::utils::check_digit::{strip_formatting, validate_issn};
use serde_json::json;

/// Parsed ISSN value
pub struct ParsedIssn {
    /// The 8-character ISSN (digits + optional X)
    value: String,
    input: String,
}

impl ParsedIssn {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed).to_uppercase();

        if cleaned.len() != 8 {
            return Err(IdtError::ParseError(format!(
                "ISSN must be exactly 8 characters, got {}",
                cleaned.len()
            )));
        }

        // First 7 must be digits
        if !cleaned[..7].chars().all(|c| c.is_ascii_digit()) {
            return Err(IdtError::ParseError(
                "ISSN first 7 characters must be digits".to_string(),
            ));
        }

        // Last character must be digit or X
        let last = cleaned.chars().nth(7).unwrap();
        if !last.is_ascii_digit() && last != 'X' {
            return Err(IdtError::ParseError(
                "ISSN check digit must be 0-9 or X".to_string(),
            ));
        }

        if !validate_issn(&cleaned) {
            return Err(IdtError::ParseError(
                "ISSN check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            value: cleaned,
            input: input_trimmed.to_string(),
        })
    }

    fn check_digit(&self) -> char {
        self.value.chars().nth(7).unwrap()
    }

    /// Return the ISSN in standard hyphenated format: XXXX-XXXX
    fn formatted(&self) -> String {
        format!("{}-{}", &self.value[..4], &self.value[4..])
    }
}

impl ParsedId for ParsedIssn {
    fn kind(&self) -> IdKind {
        IdKind::Issn
    }

    fn canonical(&self) -> String {
        self.formatted()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.value.as_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<crate::core::id::Timestamp> {
        None
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();

        let components = json!({
            "check_digit": self.check_digit().to_string(),
        });

        InspectionResult {
            id_type: "issn".to_string(),
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
            structure: Some(vec![
                StructureSegment {
                    name: "Serial Number".to_string(),
                    size: 7,
                    unit: SizeUnit::Digits,
                    value: Some(self.value[..7].to_string()),
                    description: "Serial number identifying the publication".to_string(),
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
        ValidationResult::valid("issn")
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

/// Check if a string looks like an ISSN
pub fn is_issn(input: &str) -> bool {
    ParsedIssn::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedIssn::parse("0378-5955").unwrap();
        assert_eq!(parsed.canonical(), "0378-5955");
    }

    #[test]
    fn test_parse_without_hyphen() {
        let parsed = ParsedIssn::parse("03785955").unwrap();
        assert_eq!(parsed.canonical(), "0378-5955");
    }

    #[test]
    fn test_parse_with_x() {
        let parsed = ParsedIssn::parse("0000-006X").unwrap();
        assert_eq!(parsed.canonical(), "0000-006X");
        assert_eq!(parsed.check_digit(), 'X');
    }

    #[test]
    fn test_parse_lowercase_x() {
        let parsed = ParsedIssn::parse("0000-006x").unwrap();
        assert_eq!(parsed.canonical(), "0000-006X");
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedIssn::parse("0378-5956").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedIssn::parse("0378-595").is_err());
        assert!(ParsedIssn::parse("0378-59551").is_err());
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedIssn::parse("0378-5955").unwrap();
        assert_eq!(parsed.kind(), IdKind::Issn);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedIssn::parse("0378-5955").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "issn");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["check_digit"], "5");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedIssn::parse("0378-5955").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_encode_canonical() {
        let parsed = ParsedIssn::parse("03785955").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "0378-5955");
    }

    #[test]
    fn test_is_issn() {
        assert!(is_issn("0378-5955"));
        assert!(is_issn("0000-006X"));
        assert!(!is_issn("not-an-issn"));
        assert!(!is_issn("0378-5956"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedIssn::parse("0378-5955").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "0378-5955");
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
        let parsed = ParsedIssn::parse("0378-5955").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes, b"03785955");
    }
}

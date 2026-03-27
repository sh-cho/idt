use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdEncodings, IdKind, InspectionResult, ParsedId, ValidationResult};
use crate::utils::check_digit::{strip_formatting, validate_iso7064_mod11_2};
use serde_json::json;

/// Parsed ISNI value
pub struct ParsedIsni {
    /// The 16-character ISNI (digits, last can be X)
    value: String,
    input: String,
}

impl ParsedIsni {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed).to_uppercase();

        if cleaned.len() != 16 {
            return Err(IdtError::ParseError(format!(
                "ISNI must be exactly 16 characters, got {}",
                cleaned.len()
            )));
        }

        // First 15 must be digits
        if !cleaned[..15].chars().all(|c| c.is_ascii_digit()) {
            return Err(IdtError::ParseError(
                "ISNI first 15 characters must be digits".to_string(),
            ));
        }

        // Last character must be digit or X
        let last = cleaned.chars().nth(15).unwrap();
        if !last.is_ascii_digit() && last != 'X' {
            return Err(IdtError::ParseError(
                "ISNI check digit must be 0-9 or X".to_string(),
            ));
        }

        if !validate_iso7064_mod11_2(&cleaned) {
            return Err(IdtError::ParseError(
                "ISNI check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            value: cleaned,
            input: input_trimmed.to_string(),
        })
    }

    fn check_digit(&self) -> char {
        self.value.chars().nth(15).unwrap()
    }

    /// Return ISNI in standard spaced format: XXXX XXXX XXXX XXXX
    fn formatted(&self) -> String {
        format!(
            "{} {} {} {}",
            &self.value[0..4],
            &self.value[4..8],
            &self.value[8..12],
            &self.value[12..16]
        )
    }
}

impl ParsedId for ParsedIsni {
    fn kind(&self) -> IdKind {
        IdKind::Isni
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
            id_type: "isni".to_string(),
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
        ValidationResult::valid("isni")
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

/// Check if a string looks like an ISNI
pub fn is_isni(input: &str) -> bool {
    ParsedIsni::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedIsni::parse("0000000121032683").unwrap();
        assert_eq!(parsed.canonical(), "0000 0001 2103 2683");
    }

    #[test]
    fn test_parse_with_spaces() {
        let parsed = ParsedIsni::parse("0000 0001 2103 2683").unwrap();
        assert_eq!(parsed.canonical(), "0000 0001 2103 2683");
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedIsni::parse("0000000121032684").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedIsni::parse("000000012103268").is_err());
        assert!(ParsedIsni::parse("00000001210326831").is_err());
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedIsni::parse("0000000121032683").unwrap();
        assert_eq!(parsed.kind(), IdKind::Isni);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedIsni::parse("0000000121032683").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "isni");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["check_digit"], "3");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedIsni::parse("0000000121032683").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_encode_canonical() {
        let parsed = ParsedIsni::parse("0000000121032683").unwrap();
        assert_eq!(
            parsed.encode(EncodingFormat::Canonical),
            "0000 0001 2103 2683"
        );
    }

    #[test]
    fn test_is_isni() {
        assert!(is_isni("0000000121032683"));
        assert!(is_isni("0000 0001 2103 2683"));
        assert!(!is_isni("not-an-isni"));
        assert!(!is_isni("0000000121032684"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedIsni::parse("0000000121032683").unwrap();
        assert_eq!(
            parsed.encode(EncodingFormat::Canonical),
            "0000 0001 2103 2683"
        );
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
        let parsed = ParsedIsni::parse("0000000121032683").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes, b"0000000121032683");
    }
}

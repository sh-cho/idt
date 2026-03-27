use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdEncodings, IdKind, InspectionResult, ParsedId, ValidationResult};
use crate::utils::check_digit::{
    compute_mod10_check_digit, parse_digits, strip_formatting, validate_mod10,
};
use serde_json::json;

/// Parsed UPC-A value
pub struct ParsedUpcA {
    digits: Vec<u8>,
    input: String,
}

impl ParsedUpcA {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed);

        let digits = parse_digits(&cleaned)
            .ok_or_else(|| IdtError::ParseError("UPC-A must contain only digits".to_string()))?;

        if digits.len() != 12 {
            return Err(IdtError::ParseError(format!(
                "UPC-A must be exactly 12 digits, got {}",
                digits.len()
            )));
        }

        if !validate_mod10(&digits) {
            return Err(IdtError::ParseError(
                "UPC-A check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            digits,
            input: input_trimmed.to_string(),
        })
    }

    /// Convert UPC-A to EAN-13 by prepending a leading zero.
    pub fn to_ean13(&self) -> String {
        let mut ean13_payload = vec![0u8];
        ean13_payload.extend_from_slice(&self.digits[..11]);
        let check = compute_mod10_check_digit(&ean13_payload);
        ean13_payload.push(check);
        ean13_payload.iter().map(|d| (b'0' + d) as char).collect()
    }
}

impl ParsedId for ParsedUpcA {
    fn kind(&self) -> IdKind {
        IdKind::UpcA
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
            "check_digit": self.digits[11].to_string(),
            "ean13": self.to_ean13(),
        });

        InspectionResult {
            id_type: "upca".to_string(),
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
        ValidationResult::valid("upca")
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

/// Check if a string looks like a UPC-A
pub fn is_upca(input: &str) -> bool {
    ParsedUpcA::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedUpcA::parse("036000291452").unwrap();
        assert_eq!(parsed.canonical(), "036000291452");
    }

    #[test]
    fn test_parse_valid_2() {
        let parsed = ParsedUpcA::parse("012345678905").unwrap();
        assert_eq!(parsed.canonical(), "012345678905");
    }

    #[test]
    fn test_parse_with_hyphens() {
        let parsed = ParsedUpcA::parse("0-36000-29145-2").unwrap();
        assert_eq!(parsed.canonical(), "036000291452");
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedUpcA::parse("036000291453").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedUpcA::parse("03600029145").is_err());
        assert!(ParsedUpcA::parse("0360002914521").is_err());
    }

    #[test]
    fn test_to_ean13() {
        let parsed = ParsedUpcA::parse("036000291452").unwrap();
        let ean13 = parsed.to_ean13();
        assert_eq!(ean13, "0036000291452");
        assert_eq!(ean13.len(), 13);
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedUpcA::parse("036000291452").unwrap();
        assert_eq!(parsed.kind(), IdKind::UpcA);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedUpcA::parse("036000291452").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "upca");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["check_digit"], "2");
        assert!(components["ean13"].as_str().unwrap().len() == 13);
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedUpcA::parse("036000291452").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_is_upca() {
        assert!(is_upca("036000291452"));
        assert!(is_upca("012345678905"));
        assert!(!is_upca("not-a-upc"));
        assert!(!is_upca("036000291453"));
    }
}

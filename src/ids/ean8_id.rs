use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment, ValidationResult,
};
use crate::utils::check_digit::{parse_digits, strip_formatting, validate_mod10};
use serde_json::json;

/// Parsed EAN-8 value
pub struct ParsedEan8 {
    digits: Vec<u8>,
    input: String,
}

impl ParsedEan8 {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed);

        let digits = parse_digits(&cleaned)
            .ok_or_else(|| IdtError::ParseError("EAN-8 must contain only digits".to_string()))?;

        if digits.len() != 8 {
            return Err(IdtError::ParseError(format!(
                "EAN-8 must be exactly 8 digits, got {}",
                digits.len()
            )));
        }

        if !validate_mod10(&digits) {
            return Err(IdtError::ParseError(
                "EAN-8 check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            digits,
            input: input_trimmed.to_string(),
        })
    }
}

impl ParsedId for ParsedEan8 {
    fn kind(&self) -> IdKind {
        IdKind::Ean8
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
            "check_digit": self.digits[7].to_string(),
        });

        InspectionResult {
            id_type: "ean8".to_string(),
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
                    value: Some(
                        self.digits[0..3]
                            .iter()
                            .map(|d| (b'0' + d) as char)
                            .collect(),
                    ),
                    description: "GS1 country/region prefix".to_string(),
                },
                StructureSegment {
                    name: "Item Reference".to_string(),
                    size: 4,
                    unit: SizeUnit::Digits,
                    value: Some(
                        self.digits[3..7]
                            .iter()
                            .map(|d| (b'0' + d) as char)
                            .collect(),
                    ),
                    description: "Item reference number".to_string(),
                },
                StructureSegment {
                    name: "Check Digit".to_string(),
                    size: 1,
                    unit: SizeUnit::Digits,
                    value: Some(self.digits[7].to_string()),
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
        ValidationResult::valid("ean8")
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

/// Check if a string looks like an EAN-8
pub fn is_ean8(input: &str) -> bool {
    ParsedEan8::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedEan8::parse("96385074").unwrap();
        assert_eq!(parsed.canonical(), "96385074");
    }

    #[test]
    fn test_parse_valid_2() {
        let parsed = ParsedEan8::parse("55123457").unwrap();
        assert_eq!(parsed.canonical(), "55123457");
    }

    #[test]
    fn test_parse_with_hyphens() {
        let parsed = ParsedEan8::parse("9638-5074").unwrap();
        assert_eq!(parsed.canonical(), "96385074");
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedEan8::parse("96385075").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedEan8::parse("9638507").is_err());
        assert!(ParsedEan8::parse("963850741").is_err());
    }

    #[test]
    fn test_parse_non_digit() {
        assert!(ParsedEan8::parse("9638507a").is_err());
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedEan8::parse("96385074").unwrap();
        assert_eq!(parsed.kind(), IdKind::Ean8);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedEan8::parse("96385074").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "ean8");
        assert!(result.valid);
        assert!(result.timestamp.is_none());
        assert!(result.components.is_some());
        let components = result.components.unwrap();
        assert_eq!(components["check_digit"], "4");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedEan8::parse("96385074").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_encode_canonical() {
        let parsed = ParsedEan8::parse("96385074").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "96385074");
    }

    #[test]
    fn test_is_ean8() {
        assert!(is_ean8("96385074"));
        assert!(is_ean8("55123457"));
        assert!(!is_ean8("not-an-ean"));
        assert!(!is_ean8("96385075"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedEan8::parse("96385074").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "96385074");
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
        let parsed = ParsedEan8::parse("96385074").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 8);
    }
}

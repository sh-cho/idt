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

/// Parsed GTIN-14 value
pub struct ParsedGtin14 {
    digits: Vec<u8>,
    input: String,
}

impl ParsedGtin14 {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed);

        let digits = parse_digits(&cleaned)
            .ok_or_else(|| IdtError::ParseError("GTIN-14 must contain only digits".to_string()))?;

        if digits.len() != 14 {
            return Err(IdtError::ParseError(format!(
                "GTIN-14 must be exactly 14 digits, got {}",
                digits.len()
            )));
        }

        if !validate_mod10(&digits) {
            return Err(IdtError::ParseError(
                "GTIN-14 check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            digits,
            input: input_trimmed.to_string(),
        })
    }

    fn packaging_indicator(&self) -> u8 {
        self.digits[0]
    }
}

impl ParsedId for ParsedGtin14 {
    fn kind(&self) -> IdKind {
        IdKind::Gtin14
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
            "packaging_indicator": self.packaging_indicator().to_string(),
            "check_digit": self.digits[13].to_string(),
        });

        InspectionResult {
            id_type: "gtin14".to_string(),
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
                    name: "Packaging Indicator".to_string(),
                    size: 1,
                    unit: SizeUnit::Digits,
                    value: Some(self.packaging_indicator().to_string()),
                    description: "Packaging level indicator".to_string(),
                },
                StructureSegment {
                    name: "Item Reference".to_string(),
                    size: 12,
                    unit: SizeUnit::Digits,
                    value: Some(
                        self.digits[1..13]
                            .iter()
                            .map(|d| (b'0' + d) as char)
                            .collect(),
                    ),
                    description: "GS1 company prefix and item reference".to_string(),
                },
                StructureSegment {
                    name: "Check Digit".to_string(),
                    size: 1,
                    unit: SizeUnit::Digits,
                    value: Some(self.digits[13].to_string()),
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
        ValidationResult::valid("gtin14")
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

/// Check if a string looks like a GTIN-14
pub fn is_gtin14(input: &str) -> bool {
    ParsedGtin14::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedGtin14::parse("10614141000415").unwrap();
        assert_eq!(parsed.canonical(), "10614141000415");
    }

    #[test]
    fn test_parse_with_spaces() {
        let parsed = ParsedGtin14::parse("1 0614141 000415").unwrap();
        assert_eq!(parsed.canonical(), "10614141000415");
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedGtin14::parse("10614141000416").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedGtin14::parse("1061414100041").is_err());
        assert!(ParsedGtin14::parse("106141410004151").is_err());
    }

    #[test]
    fn test_packaging_indicator() {
        let parsed = ParsedGtin14::parse("10614141000415").unwrap();
        assert_eq!(parsed.packaging_indicator(), 1);
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedGtin14::parse("10614141000415").unwrap();
        assert_eq!(parsed.kind(), IdKind::Gtin14);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedGtin14::parse("10614141000415").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "gtin14");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["packaging_indicator"], "1");
        assert_eq!(components["check_digit"], "5");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedGtin14::parse("10614141000415").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_is_gtin14() {
        assert!(is_gtin14("10614141000415"));
        assert!(!is_gtin14("not-a-gtin"));
        assert!(!is_gtin14("10614141000416"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedGtin14::parse("10614141000415").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "10614141000415");
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
        let parsed = ParsedGtin14::parse("10614141000415").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 14);
    }
}

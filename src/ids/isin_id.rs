use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment, ValidationResult,
};
use crate::utils::check_digit::{strip_formatting, validate_isin_luhn};
use serde_json::json;

/// Parsed ISIN value
pub struct ParsedIsin {
    /// The 12-character ISIN (uppercase)
    value: String,
    input: String,
}

impl ParsedIsin {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let cleaned = strip_formatting(input_trimmed).to_uppercase();

        if cleaned.len() != 12 {
            return Err(IdtError::ParseError(format!(
                "ISIN must be exactly 12 characters, got {}",
                cleaned.len()
            )));
        }

        // First 2 characters must be letters (country code)
        let chars: Vec<char> = cleaned.chars().collect();
        if !chars[0].is_ascii_uppercase() || !chars[1].is_ascii_uppercase() {
            return Err(IdtError::ParseError(
                "ISIN must start with a 2-letter country code".to_string(),
            ));
        }

        // Characters 3-11 must be alphanumeric
        if !chars[2..11].iter().all(|c| c.is_ascii_alphanumeric()) {
            return Err(IdtError::ParseError(
                "ISIN characters 3-11 must be alphanumeric".to_string(),
            ));
        }

        // Last character must be a digit (check digit)
        if !chars[11].is_ascii_digit() {
            return Err(IdtError::ParseError(
                "ISIN check digit must be a digit".to_string(),
            ));
        }

        if !validate_isin_luhn(&cleaned) {
            return Err(IdtError::ParseError(
                "ISIN check digit is invalid".to_string(),
            ));
        }

        Ok(Self {
            value: cleaned,
            input: input_trimmed.to_string(),
        })
    }

    fn country_code(&self) -> &str {
        &self.value[0..2]
    }

    fn nsin(&self) -> &str {
        &self.value[2..11]
    }

    fn check_digit(&self) -> char {
        self.value.chars().nth(11).unwrap()
    }
}

impl ParsedId for ParsedIsin {
    fn kind(&self) -> IdKind {
        IdKind::Isin
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

        let components = json!({
            "country_code": self.country_code(),
            "nsin": self.nsin(),
            "check_digit": self.check_digit().to_string(),
        });

        InspectionResult {
            id_type: "isin".to_string(),
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
                    name: "Country Code".to_string(),
                    size: 2,
                    unit: SizeUnit::Chars,
                    value: Some(self.country_code().to_string()),
                    description: "ISO 3166-1 alpha-2 country code".to_string(),
                },
                StructureSegment {
                    name: "NSIN".to_string(),
                    size: 9,
                    unit: SizeUnit::Chars,
                    value: Some(self.nsin().to_string()),
                    description: "National Securities Identifying Number".to_string(),
                },
                StructureSegment {
                    name: "Check Digit".to_string(),
                    size: 1,
                    unit: SizeUnit::Chars,
                    value: Some(self.check_digit().to_string()),
                    description: "Luhn algorithm check digit".to_string(),
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
        ValidationResult::valid("isin")
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

/// Check if a string looks like an ISIN
pub fn is_isin(input: &str) -> bool {
    ParsedIsin::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_apple() {
        let parsed = ParsedIsin::parse("US0378331005").unwrap();
        assert_eq!(parsed.canonical(), "US0378331005");
        assert_eq!(parsed.country_code(), "US");
        assert_eq!(parsed.nsin(), "037833100");
        assert_eq!(parsed.check_digit(), '5');
    }

    #[test]
    fn test_parse_australian() {
        let parsed = ParsedIsin::parse("AU0000XVGZA3").unwrap();
        assert_eq!(parsed.canonical(), "AU0000XVGZA3");
        assert_eq!(parsed.country_code(), "AU");
    }

    #[test]
    fn test_parse_british() {
        let parsed = ParsedIsin::parse("GB0002634946").unwrap();
        assert_eq!(parsed.canonical(), "GB0002634946");
        assert_eq!(parsed.country_code(), "GB");
    }

    #[test]
    fn test_parse_lowercase() {
        let parsed = ParsedIsin::parse("us0378331005").unwrap();
        assert_eq!(parsed.canonical(), "US0378331005");
    }

    #[test]
    fn test_parse_invalid_check_digit() {
        assert!(ParsedIsin::parse("US0378331006").is_err());
    }

    #[test]
    fn test_parse_wrong_length() {
        assert!(ParsedIsin::parse("US037833100").is_err());
        assert!(ParsedIsin::parse("US03783310055").is_err());
    }

    #[test]
    fn test_parse_no_country_code() {
        assert!(ParsedIsin::parse("120378331005").is_err());
    }

    #[test]
    fn test_kind() {
        let parsed = ParsedIsin::parse("US0378331005").unwrap();
        assert_eq!(parsed.kind(), IdKind::Isin);
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedIsin::parse("US0378331005").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "isin");
        assert!(result.valid);
        let components = result.components.unwrap();
        assert_eq!(components["country_code"], "US");
        assert_eq!(components["nsin"], "037833100");
        assert_eq!(components["check_digit"], "5");
    }

    #[test]
    fn test_validate() {
        let parsed = ParsedIsin::parse("US0378331005").unwrap();
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_encode_canonical() {
        let parsed = ParsedIsin::parse("US0378331005").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "US0378331005");
    }

    #[test]
    fn test_is_isin() {
        assert!(is_isin("US0378331005"));
        assert!(is_isin("AU0000XVGZA3"));
        assert!(is_isin("GB0002634946"));
        assert!(!is_isin("not-an-isin"));
        assert!(!is_isin("US0378331006"));
    }

    #[test]
    fn test_encode_all_formats() {
        let parsed = ParsedIsin::parse("US0378331005").unwrap();
        assert_eq!(parsed.encode(EncodingFormat::Canonical), "US0378331005");
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
        let parsed = ParsedIsin::parse("US0378331005").unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 12);
        assert_eq!(bytes, b"US0378331005");
    }
}

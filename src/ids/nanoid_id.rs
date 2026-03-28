use crate::core::encoding::{EncodingFormat, encode_base64, encode_hex};
use crate::core::error::Result;
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use serde_json::json;

/// Default NanoID alphabet (URL-safe)
pub const DEFAULT_ALPHABET: &str =
    "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz-";

/// Default NanoID length
pub const DEFAULT_LENGTH: usize = 21;

/// NanoID generator with configurable alphabet and length
pub struct NanoIdGenerator {
    pub alphabet: String,
    pub length: usize,
}

impl Default for NanoIdGenerator {
    fn default() -> Self {
        Self {
            alphabet: DEFAULT_ALPHABET.to_string(),
            length: DEFAULT_LENGTH,
        }
    }
}

impl NanoIdGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn with_alphabet(mut self, alphabet: &str) -> Self {
        self.alphabet = alphabet.to_string();
        self
    }
}

impl IdGenerator for NanoIdGenerator {
    fn generate(&self) -> Result<String> {
        let alphabet: Vec<char> = self.alphabet.chars().collect();
        Ok(nanoid::format(
            nanoid::rngs::default,
            &alphabet,
            self.length,
        ))
    }
}

/// Parsed NanoID value
pub struct ParsedNanoId {
    value: String,
    input: String,
}

impl ParsedNanoId {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();

        // NanoID validation is lenient since it can have custom alphabets
        // We just check that it's not empty
        if input_trimmed.is_empty() {
            return Err(crate::core::error::IdtError::ParseError(
                "Empty NanoID".to_string(),
            ));
        }

        Ok(Self {
            value: input_trimmed.to_string(),
            input: input_trimmed.to_string(),
        })
    }

    /// Check if the input matches the default NanoID format
    pub fn is_default_format(input: &str) -> bool {
        if input.len() != DEFAULT_LENGTH {
            return false;
        }
        input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }
}

impl ParsedId for ParsedNanoId {
    fn kind(&self) -> IdKind {
        IdKind::NanoId
    }

    fn canonical(&self) -> String {
        self.value.clone()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.value.as_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        // NanoID doesn't contain timestamp
        None
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let entropy_bits = (self.value.len() as f64 * 6.0) as u32; // Approximate

        let components = json!({
            "length": self.value.len(),
            "charset": "URL-safe (default)",
        });

        InspectionResult {
            id_type: "nanoid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: None,
            timestamp_iso: None,
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: Some(entropy_bits),
            components: Some(components),
            structure: None,
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: String::new(), // Not meaningful for NanoID
                base58: String::new(),
                base64: encode_base64(&bytes),
                int: None,
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        if Self::is_default_format(&self.value) {
            ValidationResult::valid("nanoid")
        } else {
            ValidationResult::valid("nanoid").with_hint("Non-standard length or alphabet")
        }
    }

    fn encode(&self, format: EncodingFormat) -> String {
        let bytes = self.as_bytes();
        match format {
            EncodingFormat::Canonical => self.canonical(),
            EncodingFormat::Hex => encode_hex(&bytes),
            EncodingFormat::Base64 => encode_base64(&bytes),
            _ => self.canonical(), // Most encodings don't make sense for NanoID
        }
    }
}

/// Check if a string looks like a NanoID
pub fn is_nanoid(input: &str) -> bool {
    ParsedNanoId::is_default_format(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default() {
        let generator = NanoIdGenerator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), DEFAULT_LENGTH);
    }

    #[test]
    fn test_generate_custom_length() {
        let generator = NanoIdGenerator::new().with_length(32);
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_generate_hex_alphabet() {
        let generator = NanoIdGenerator::new()
            .with_alphabet("0123456789abcdef")
            .with_length(16);
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_is_default_format() {
        assert!(ParsedNanoId::is_default_format("V1StGXR8_Z5jdHi6B-myT"));
        assert!(!ParsedNanoId::is_default_format("too-short"));
    }

    #[test]
    fn test_parse_valid() {
        let parsed = ParsedNanoId::parse("V1StGXR8_Z5jdHi6B-myT").unwrap();
        assert_eq!(parsed.kind(), IdKind::NanoId);
    }

    #[test]
    fn test_parse_empty() {
        assert!(ParsedNanoId::parse("").is_err());
        assert!(ParsedNanoId::parse("   ").is_err());
    }

    #[test]
    fn test_parse_trims_whitespace() {
        let parsed = ParsedNanoId::parse("  V1StGXR8_Z5jdHi6B-myT  ").unwrap();
        assert_eq!(parsed.canonical(), "V1StGXR8_Z5jdHi6B-myT");
    }

    #[test]
    fn test_canonical() {
        let parsed = ParsedNanoId::parse("V1StGXR8_Z5jdHi6B-myT").unwrap();
        assert_eq!(parsed.canonical(), "V1StGXR8_Z5jdHi6B-myT");
    }

    #[test]
    fn test_as_bytes() {
        let parsed = ParsedNanoId::parse("abc").unwrap();
        assert_eq!(parsed.as_bytes(), b"abc");
    }

    #[test]
    fn test_timestamp_is_none() {
        let parsed = ParsedNanoId::parse("V1StGXR8_Z5jdHi6B-myT").unwrap();
        assert!(parsed.timestamp().is_none());
    }

    #[test]
    fn test_inspect() {
        let parsed = ParsedNanoId::parse("V1StGXR8_Z5jdHi6B-myT").unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "nanoid");
        assert!(result.valid);
        assert!(result.timestamp.is_none());
        assert!(result.random_bits.is_some());
        assert!(result.components.is_some());
        assert!(!result.encodings.hex.is_empty());
        assert!(!result.encodings.base64.is_empty());
    }

    #[test]
    fn test_validate_default_format() {
        let parsed = ParsedNanoId::parse("V1StGXR8_Z5jdHi6B-myT").unwrap();
        let result = parsed.validate();
        assert!(result.valid);
        assert!(result.hint.is_none());
    }

    #[test]
    fn test_validate_non_default_format() {
        let parsed = ParsedNanoId::parse("short").unwrap();
        let result = parsed.validate();
        assert!(result.valid);
        assert!(result.hint.is_some());
        assert!(result.hint.unwrap().contains("Non-standard"));
    }

    #[test]
    fn test_encode_canonical() {
        let parsed = ParsedNanoId::parse("V1StGXR8_Z5jdHi6B-myT").unwrap();
        assert_eq!(
            parsed.encode(EncodingFormat::Canonical),
            "V1StGXR8_Z5jdHi6B-myT"
        );
    }

    #[test]
    fn test_encode_hex() {
        let parsed = ParsedNanoId::parse("abc").unwrap();
        let hex = parsed.encode(EncodingFormat::Hex);
        assert_eq!(hex, "616263");
    }

    #[test]
    fn test_encode_base64() {
        let parsed = ParsedNanoId::parse("abc").unwrap();
        let b64 = parsed.encode(EncodingFormat::Base64);
        assert_eq!(b64, "YWJj");
    }

    #[test]
    fn test_encode_fallback() {
        let parsed = ParsedNanoId::parse("V1StGXR8_Z5jdHi6B-myT").unwrap();
        let result = parsed.encode(EncodingFormat::Base58);
        assert_eq!(result, "V1StGXR8_Z5jdHi6B-myT");
    }

    #[test]
    fn test_is_nanoid() {
        assert!(is_nanoid("V1StGXR8_Z5jdHi6B-myT"));
        assert!(!is_nanoid("too-short"));
        assert!(!is_nanoid(""));
    }

    #[test]
    fn test_generate_many() {
        let generator = NanoIdGenerator::new();
        let ids = generator.generate_many(5).unwrap();
        assert_eq!(ids.len(), 5);
        for id in &ids {
            assert_eq!(id.len(), DEFAULT_LENGTH);
        }
        // All unique
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 5);
    }
}

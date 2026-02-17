use crate::core::encoding::{encode_base64, encode_hex, EncodingFormat};
use crate::core::error::Result;
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp,
    ValidationResult,
};
use serde_json::json;

/// Default NanoID alphabet (URL-safe)
pub const DEFAULT_ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz-";

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
        Ok(nanoid::format(nanoid::rngs::default, &alphabet, self.length))
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
}

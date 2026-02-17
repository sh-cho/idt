use crate::core::encoding::{
    encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper, EncodingFormat,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp,
    ValidationResult,
};
use serde_json::json;
use ulid::Ulid;

/// ULID generator
#[derive(Default)]
pub struct UlidGenerator;

impl UlidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for UlidGenerator {
    fn generate(&self) -> Result<String> {
        let ulid = Ulid::new();
        Ok(ulid.to_string())
    }
}

/// Parsed ULID value
pub struct ParsedUlid {
    ulid: Ulid,
    input: String,
}

impl ParsedUlid {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();

        // ULID is case-insensitive
        let ulid = Ulid::from_string(input_trimmed)
            .map_err(|e| IdtError::ParseError(format!("Invalid ULID: {}", e)))?;

        Ok(Self {
            ulid,
            input: input_trimmed.to_string(),
        })
    }
}

impl ParsedId for ParsedUlid {
    fn kind(&self) -> IdKind {
        IdKind::Ulid
    }

    fn canonical(&self) -> String {
        self.ulid.to_string()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.ulid.to_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        Some(Timestamp::new(self.ulid.timestamp_ms()))
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let timestamp = self.timestamp().unwrap();
        let random_bytes = &bytes[6..]; // Last 10 bytes are random

        let components = json!({
            "timestamp_ms": timestamp.millis,
            "random_hex": encode_hex(random_bytes),
        });

        InspectionResult {
            id_type: "ulid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: Some(timestamp.clone()),
            timestamp_iso: Some(timestamp.to_iso8601()),
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: Some(80), // 10 bytes * 8 bits
            components: Some(components),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: encode_base32(&bytes),
                base58: encode_base58(&bytes),
                base64: encode_base64(&bytes),
                int: Some(u128::from_be_bytes(bytes.try_into().unwrap()).to_string()),
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        ValidationResult::valid("ulid")
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
            EncodingFormat::Int => u128::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            EncodingFormat::Bytes => encode_bytes_spaced(&bytes),
        }
    }
}

/// Check if a string can be parsed as ULID
pub fn is_ulid(input: &str) -> bool {
    ParsedUlid::parse(input).is_ok()
}

/// Convert ULID to UUID (they share the same 128-bit space)
pub fn ulid_to_uuid(ulid: &Ulid) -> uuid::Uuid {
    uuid::Uuid::from_bytes(ulid.to_bytes())
}

/// Convert UUID to ULID
pub fn uuid_to_ulid(uuid: &uuid::Uuid) -> Ulid {
    Ulid::from_bytes(*uuid.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = UlidGenerator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 26);
        assert!(is_ulid(&id));
    }

    #[test]
    fn test_parse() {
        let input = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
        let parsed = ParsedUlid::parse(input).unwrap();
        assert_eq!(parsed.kind(), IdKind::Ulid);
        assert!(parsed.timestamp().is_some());
    }

    #[test]
    fn test_case_insensitive() {
        let upper = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
        let lower = "01arz3ndektsv4rrffq69g5fav";

        let parsed_upper = ParsedUlid::parse(upper).unwrap();
        let parsed_lower = ParsedUlid::parse(lower).unwrap();

        assert_eq!(parsed_upper.canonical(), parsed_lower.canonical());
    }

    #[test]
    fn test_ulid_uuid_conversion() {
        let ulid = Ulid::new();
        let uuid = ulid_to_uuid(&ulid);
        let back = uuid_to_ulid(&uuid);
        assert_eq!(ulid, back);
    }
}

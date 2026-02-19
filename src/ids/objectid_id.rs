use crate::core::encoding::{
    encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper, EncodingFormat,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use rand::Rng;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

/// 5-byte random value, unique per process
static PROCESS_RANDOM: OnceLock<[u8; 5]> = OnceLock::new();

/// 3-byte counter, starting from a random value
static COUNTER: AtomicU32 = AtomicU32::new(0);
static COUNTER_INIT: OnceLock<()> = OnceLock::new();

fn process_random() -> &'static [u8; 5] {
    PROCESS_RANDOM.get_or_init(|| {
        let mut rng = rand::thread_rng();
        let mut buf = [0u8; 5];
        rng.fill(&mut buf);
        buf
    })
}

fn next_counter() -> u32 {
    COUNTER_INIT.get_or_init(|| {
        let mut rng = rand::thread_rng();
        COUNTER.store(rng.r#gen::<u32>() & 0xFF_FFFF, Ordering::SeqCst);
    });
    COUNTER.fetch_add(1, Ordering::SeqCst) & 0xFF_FFFF
}

/// MongoDB ObjectId generator
pub struct ObjectIdGenerator;

impl ObjectIdGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for ObjectIdGenerator {
    fn generate(&self) -> Result<String> {
        let now = chrono::Utc::now().timestamp() as u32;
        let random = process_random();
        let counter = next_counter();

        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&now.to_be_bytes());
        bytes[4..9].copy_from_slice(random);
        bytes[9] = ((counter >> 16) & 0xFF) as u8;
        bytes[10] = ((counter >> 8) & 0xFF) as u8;
        bytes[11] = (counter & 0xFF) as u8;

        Ok(encode_hex(&bytes))
    }
}

/// Parsed MongoDB ObjectId
pub struct ParsedObjectId {
    bytes: [u8; 12],
    input: String,
}

impl ParsedObjectId {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        if input_trimmed.len() != 24 {
            return Err(IdtError::ParseError(
                "ObjectId must be 24 hex characters".to_string(),
            ));
        }
        if !input_trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(IdtError::ParseError(
                "ObjectId must contain only hex characters".to_string(),
            ));
        }

        let decoded = hex::decode(input_trimmed.to_lowercase())
            .map_err(|e| IdtError::ParseError(format!("Invalid ObjectId hex: {}", e)))?;

        let bytes: [u8; 12] = decoded
            .try_into()
            .map_err(|_| IdtError::ParseError("ObjectId must be 12 bytes".to_string()))?;

        Ok(Self {
            bytes,
            input: input_trimmed.to_string(),
        })
    }

    fn timestamp_secs(&self) -> u32 {
        u32::from_be_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    fn random_bytes(&self) -> &[u8] {
        &self.bytes[4..9]
    }

    fn counter(&self) -> u32 {
        ((self.bytes[9] as u32) << 16) | ((self.bytes[10] as u32) << 8) | (self.bytes[11] as u32)
    }
}

impl ParsedId for ParsedObjectId {
    fn kind(&self) -> IdKind {
        IdKind::ObjectId
    }

    fn canonical(&self) -> String {
        encode_hex(&self.bytes)
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        Some(Timestamp::from_secs(self.timestamp_secs() as u64))
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let timestamp = self.timestamp().unwrap();

        let components = json!({
            "timestamp_secs": self.timestamp_secs(),
            "random_hex": encode_hex(self.random_bytes()),
            "counter": self.counter(),
        });

        InspectionResult {
            id_type: "objectid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: Some(timestamp),
            timestamp_iso: Some(timestamp.to_iso8601()),
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: Some(40),
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
        let ts = self.timestamp_secs();
        let now = chrono::Utc::now().timestamp() as u32;
        if ts > now + 86400 {
            ValidationResult::invalid("Timestamp is in the future")
        } else {
            ValidationResult::valid("objectid")
        }
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
            EncodingFormat::Int => {
                // 96-bit value
                let mut val: u128 = 0;
                for &b in &bytes {
                    val = (val << 8) | b as u128;
                }
                val.to_string()
            }
            EncodingFormat::Bytes => encode_bytes_spaced(&bytes),
        }
    }
}

/// Check if a string looks like a MongoDB ObjectId
pub fn is_objectid(input: &str) -> bool {
    ParsedObjectId::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = ObjectIdGenerator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 24);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_parse_known() {
        let parsed = ParsedObjectId::parse("507f1f77bcf86cd799439011").unwrap();
        assert_eq!(parsed.kind(), IdKind::ObjectId);
        assert!(parsed.timestamp().is_some());
        assert_eq!(parsed.timestamp_secs(), 0x507f1f77);
    }

    #[test]
    fn test_roundtrip() {
        let generator = ObjectIdGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedObjectId::parse(&id).unwrap();
        assert_eq!(parsed.canonical(), id);
    }

    #[test]
    fn test_counter_increments() {
        let generator = ObjectIdGenerator::new();
        let id1 = generator.generate().unwrap();
        let id2 = generator.generate().unwrap();
        assert_ne!(id1, id2);
    }
}

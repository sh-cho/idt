use crate::core::encoding::{EncodingFormat, encode_base64, encode_bits, encode_hex};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use rand::Rng;
use serde_json::json;

/// Crockford Base32 alphabet
const CROCKFORD: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// TSID generator
pub struct TsidGenerator;

impl Default for TsidGenerator {
    fn default() -> Self {
        Self
    }
}

impl TsidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for TsidGenerator {
    fn generate(&self) -> Result<String> {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let mut rng = rand::thread_rng();
        let random_bits: u64 = rng.r#gen::<u64>() & 0x3F_FFFF; // 22 bits

        let value = (now_ms << 22) | random_bits;
        Ok(tsid_encode(value))
    }
}

/// Encode u64 as 13-char Crockford Base32
fn tsid_encode(value: u64) -> String {
    let mut result = [0u8; 13];
    let mut v = value;
    for i in (0..13).rev() {
        result[i] = CROCKFORD[(v & 0x1F) as usize];
        v >>= 5;
    }
    String::from_utf8(result.to_vec()).unwrap()
}

/// Decode 13-char Crockford Base32 to u64
fn tsid_decode(s: &str) -> Result<u64> {
    if s.len() != 13 {
        return Err(IdtError::ParseError(
            "TSID must be 13 characters".to_string(),
        ));
    }

    let mut value: u64 = 0;
    for ch in s.chars() {
        let v = crockford_char_value(ch).ok_or_else(|| {
            IdtError::ParseError(format!("Invalid Crockford Base32 character: '{}'", ch))
        })?;
        value = (value << 5) | (v as u64);
    }
    Ok(value)
}

fn crockford_char_value(c: char) -> Option<u8> {
    match c.to_ascii_uppercase() {
        '0' | 'O' => Some(0),
        '1' | 'I' | 'L' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'A' => Some(10),
        'B' => Some(11),
        'C' => Some(12),
        'D' => Some(13),
        'E' => Some(14),
        'F' => Some(15),
        'G' => Some(16),
        'H' => Some(17),
        'J' => Some(18),
        'K' => Some(19),
        'M' => Some(20),
        'N' => Some(21),
        'P' => Some(22),
        'Q' => Some(23),
        'R' => Some(24),
        'S' => Some(25),
        'T' => Some(26),
        'V' => Some(27),
        'W' => Some(28),
        'X' => Some(29),
        'Y' => Some(30),
        'Z' => Some(31),
        _ => None,
    }
}

/// Parsed TSID value
pub struct ParsedTsid {
    value: u64,
    input: String,
}

impl ParsedTsid {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let value = tsid_decode(input_trimmed)?;
        Ok(Self {
            value,
            input: input_trimmed.to_string(),
        })
    }

    fn timestamp_ms(&self) -> u64 {
        self.value >> 22
    }

    fn random_bits(&self) -> u64 {
        self.value & 0x3F_FFFF
    }
}

impl ParsedId for ParsedTsid {
    fn kind(&self) -> IdKind {
        IdKind::Tsid
    }

    fn canonical(&self) -> String {
        tsid_encode(self.value)
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.value.to_be_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        Some(Timestamp::new(self.timestamp_ms()))
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let timestamp = self.timestamp().unwrap();

        let components = json!({
            "timestamp_ms": self.timestamp_ms(),
            "random_bits": self.random_bits(),
            "numeric_value": self.value,
        });

        InspectionResult {
            id_type: "tsid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: Some(timestamp),
            timestamp_iso: Some(timestamp.to_iso8601()),
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: Some(22),
            components: Some(components),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: String::new(),
                base58: String::new(),
                base64: encode_base64(&bytes),
                int: Some(self.value.to_string()),
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        let ts_ms = self.timestamp_ms();
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        if ts_ms > now_ms + 86_400_000 {
            ValidationResult::invalid("Timestamp is in the future")
        } else {
            ValidationResult::valid("tsid")
        }
    }

    fn encode(&self, format: EncodingFormat) -> String {
        let bytes = self.as_bytes();
        match format {
            EncodingFormat::Canonical => self.canonical(),
            EncodingFormat::Hex => encode_hex(&bytes),
            EncodingFormat::Base64 => encode_base64(&bytes),
            EncodingFormat::Bits => encode_bits(&bytes),
            EncodingFormat::Int => self.value.to_string(),
            _ => self.canonical(),
        }
    }
}

/// Check if a string looks like a TSID
pub fn is_tsid(input: &str) -> bool {
    ParsedTsid::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 13);
    }

    #[test]
    fn test_roundtrip() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();
        assert_eq!(parsed.canonical(), id);
    }

    #[test]
    fn test_has_timestamp() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();
        let ts = parsed.timestamp().unwrap();
        let now = chrono::Utc::now().timestamp_millis() as u64;
        assert!(now.abs_diff(ts.millis) < 5000);
    }

    #[test]
    fn test_encode_decode() {
        let value: u64 = 0x0123456789ABCDEF;
        let encoded = tsid_encode(value);
        let decoded = tsid_decode(&encoded).unwrap();
        assert_eq!(value, decoded);
    }
}

use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use rand::RngExt;
use serde_json::json;

/// KSUID epoch offset: 14e8 seconds (2014-05-13T16:53:20Z)
const KSUID_EPOCH: u64 = 1_400_000_000;

/// Base62 alphabet
const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// KSUID generator
pub struct KsuidGenerator;

impl Default for KsuidGenerator {
    fn default() -> Self {
        Self
    }
}

impl KsuidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for KsuidGenerator {
    fn generate(&self) -> Result<String> {
        let now = chrono::Utc::now().timestamp() as u64;
        let offset = now.saturating_sub(KSUID_EPOCH);

        let mut bytes = [0u8; 20];
        bytes[0..4].copy_from_slice(&(offset as u32).to_be_bytes());

        let mut rng = rand::rng();
        rng.fill(&mut bytes[4..20]);

        Ok(encode_base62(&bytes))
    }
}

/// Encode 20 bytes as 27-char base62 string
fn encode_base62(bytes: &[u8; 20]) -> String {
    // Convert bytes to a big integer (as a Vec<u8> for divmod)
    let mut num = bytes.to_vec();
    let mut result = Vec::with_capacity(27);

    for _ in 0..27 {
        let remainder = divmod_base62(&mut num, 62);
        result.push(BASE62[remainder as usize]);
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Divide a big-endian byte array by `divisor`, returning the remainder.
/// Modifies `num` in-place to contain the quotient.
fn divmod_base62(num: &mut [u8], divisor: u16) -> u8 {
    let mut remainder: u16 = 0;
    for byte in num.iter_mut() {
        let acc = (remainder << 8) | (*byte as u16);
        *byte = (acc / divisor) as u8;
        remainder = acc % divisor;
    }
    remainder as u8
}

/// Decode 27-char base62 string into 20 bytes
fn decode_base62(s: &str) -> Result<[u8; 20]> {
    if s.len() != 27 {
        return Err(IdtError::ParseError(
            "KSUID must be 27 characters".to_string(),
        ));
    }

    // Convert base62 string to big integer bytes
    let mut num = vec![0u8; 20];

    for ch in s.chars() {
        let val = base62_char_value(ch)
            .ok_or_else(|| IdtError::ParseError(format!("Invalid base62 character: '{}'", ch)))?;

        // Multiply num by 62 and add val
        let mut carry: u16 = val as u16;
        for byte in num.iter_mut().rev() {
            let acc = (*byte as u16) * 62 + carry;
            *byte = (acc & 0xFF) as u8;
            carry = acc >> 8;
        }
    }

    let result: [u8; 20] = num
        .try_into()
        .map_err(|_| IdtError::ParseError("Failed to decode KSUID bytes".to_string()))?;
    Ok(result)
}

fn base62_char_value(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'A'..='Z' => Some(c as u8 - b'A' + 10),
        'a'..='z' => Some(c as u8 - b'a' + 36),
        _ => None,
    }
}

/// Parsed KSUID value
pub struct ParsedKsuid {
    bytes: [u8; 20],
    input: String,
}

impl ParsedKsuid {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let bytes = decode_base62(input_trimmed)?;
        Ok(Self {
            bytes,
            input: input_trimmed.to_string(),
        })
    }

    fn timestamp_offset(&self) -> u32 {
        u32::from_be_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    fn unix_timestamp_secs(&self) -> u64 {
        self.timestamp_offset() as u64 + KSUID_EPOCH
    }

    fn payload(&self) -> &[u8] {
        &self.bytes[4..20]
    }
}

impl ParsedId for ParsedKsuid {
    fn kind(&self) -> IdKind {
        IdKind::Ksuid
    }

    fn canonical(&self) -> String {
        encode_base62(&self.bytes)
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        Some(Timestamp::from_secs(self.unix_timestamp_secs()))
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let timestamp = self.timestamp().unwrap();

        let components = json!({
            "timestamp_secs": self.unix_timestamp_secs(),
            "ksuid_epoch_offset": self.timestamp_offset(),
            "payload_hex": encode_hex(self.payload()),
        });

        InspectionResult {
            id_type: "ksuid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: Some(timestamp),
            timestamp_iso: Some(timestamp.to_iso8601()),
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: Some(128),
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
        ValidationResult::valid("ksuid")
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
                let mut val: u128 = 0;
                for &b in bytes.iter() {
                    val = (val << 8) | b as u128;
                }
                val.to_string()
            }
            EncodingFormat::Bytes => encode_bytes_spaced(&bytes),
        }
    }
}

/// Check if a string looks like a KSUID
pub fn is_ksuid(input: &str) -> bool {
    ParsedKsuid::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 27);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_roundtrip() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&id).unwrap();
        assert_eq!(parsed.canonical(), id);
    }

    #[test]
    fn test_has_timestamp() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&id).unwrap();
        let ts = parsed.timestamp().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        // Timestamp should be within 10 seconds of now
        assert!((now * 1000).abs_diff(ts.millis) < 10_000);
    }

    #[test]
    fn test_base62_encode_decode() {
        let mut bytes = [0u8; 20];
        bytes[0] = 0x0B;
        bytes[1] = 0x5F;
        bytes[2] = 0x03;
        bytes[3] = 0x58;
        // Fill rest with known values
        for (i, byte) in bytes.iter_mut().enumerate().skip(4) {
            *byte = (i * 13) as u8;
        }

        let encoded = encode_base62(&bytes);
        assert_eq!(encoded.len(), 27);
        let decoded = decode_base62(&encoded).unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn test_parse_error_wrong_length() {
        assert!(ParsedKsuid::parse("too_short").is_err());
        assert!(ParsedKsuid::parse("").is_err());
    }

    #[test]
    fn test_parse_error_invalid_chars() {
        assert!(ParsedKsuid::parse("!!!!!!!!!!!!!!!!!!!!!!!!!!!").is_err());
    }

    #[test]
    fn test_parse_trims_whitespace() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&format!("  {}  ", id)).unwrap();
        assert_eq!(parsed.canonical(), id);
    }

    #[test]
    fn test_kind() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::Ksuid);
    }

    #[test]
    fn test_as_bytes() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&id).unwrap();
        assert_eq!(parsed.as_bytes().len(), 20);
    }

    #[test]
    fn test_inspect() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&id).unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "ksuid");
        assert!(result.valid);
        assert!(result.timestamp.is_some());
        assert!(result.components.is_some());
        assert_eq!(result.random_bits, Some(128));
        assert!(!result.encodings.hex.is_empty());
        assert!(!result.encodings.base32.is_empty());
        assert!(!result.encodings.base58.is_empty());
        assert!(!result.encodings.base64.is_empty());
    }

    #[test]
    fn test_validate() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&id).unwrap();
        let result = parsed.validate();
        assert!(result.valid);
    }

    #[test]
    fn test_encode_formats() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedKsuid::parse(&id).unwrap();

        assert_eq!(parsed.encode(EncodingFormat::Canonical), id);
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
    fn test_is_ksuid() {
        let generator = KsuidGenerator::new();
        let id = generator.generate().unwrap();
        assert!(is_ksuid(&id));
        assert!(!is_ksuid("not-a-ksuid"));
        assert!(!is_ksuid(""));
    }

    #[test]
    fn test_base62_char_value() {
        assert_eq!(base62_char_value('0'), Some(0));
        assert_eq!(base62_char_value('9'), Some(9));
        assert_eq!(base62_char_value('A'), Some(10));
        assert_eq!(base62_char_value('Z'), Some(35));
        assert_eq!(base62_char_value('a'), Some(36));
        assert_eq!(base62_char_value('z'), Some(61));
        assert_eq!(base62_char_value('!'), None);
    }
}

use crate::core::encoding::{EncodingFormat, encode_base64, encode_bits, encode_hex};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment,
    Timestamp, ValidationResult,
};
use rand::RngExt;
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
        let mut rng = rand::rng();
        let random_bits: u64 = rng.random::<u64>() & 0x3F_FFFF; // 22 bits

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
    String::from_utf8(result.to_vec()).expect("CROCKFORD alphabet is valid UTF-8")
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
        let timestamp = self.timestamp().expect("TSID always has a timestamp");

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
            structure: Some(vec![
                StructureSegment {
                    name: "Timestamp".to_string(),
                    size: 42,
                    unit: SizeUnit::Bits,
                    value: Some(self.timestamp_ms().to_string()),
                    description: "Unix timestamp in milliseconds".to_string(),
                },
                StructureSegment {
                    name: "Random".to_string(),
                    size: 22,
                    unit: SizeUnit::Bits,
                    value: Some(self.random_bits().to_string()),
                    description: "Random bits for uniqueness".to_string(),
                },
            ]),
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

    #[test]
    fn test_parse_error_wrong_length() {
        assert!(ParsedTsid::parse("too_short").is_err());
        assert!(ParsedTsid::parse("").is_err());
        assert!(ParsedTsid::parse("12345678901234").is_err()); // 14 chars
    }

    #[test]
    fn test_parse_error_invalid_chars() {
        // 'U' is not valid in Crockford Base32
        assert!(ParsedTsid::parse("UUUUUUUUUUUUU").is_err());
    }

    #[test]
    fn test_parse_trims_whitespace() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&format!("  {}  ", id)).unwrap();
        assert_eq!(parsed.canonical(), id);
    }

    #[test]
    fn test_kind() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::Tsid);
    }

    #[test]
    fn test_as_bytes() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();
        assert_eq!(parsed.as_bytes().len(), 8);
    }

    #[test]
    fn test_inspect() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();
        let result = parsed.inspect();
        assert_eq!(result.id_type, "tsid");
        assert!(result.valid);
        assert!(result.timestamp.is_some());
        assert!(result.components.is_some());
        assert_eq!(result.random_bits, Some(22));
        assert!(!result.encodings.hex.is_empty());
        assert!(!result.encodings.base64.is_empty());
        assert!(result.encodings.int.is_some());
    }

    #[test]
    fn test_validate() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();
        let result = parsed.validate();
        assert!(result.valid);
    }

    #[test]
    fn test_encode_formats() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();

        assert_eq!(parsed.encode(EncodingFormat::Canonical), id);
        assert!(!parsed.encode(EncodingFormat::Hex).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base64).is_empty());
        assert!(!parsed.encode(EncodingFormat::Bits).is_empty());
        assert!(!parsed.encode(EncodingFormat::Int).is_empty());
        // Fallback formats return canonical
        assert_eq!(parsed.encode(EncodingFormat::Base58), id);
    }

    #[test]
    fn test_is_tsid() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        assert!(is_tsid(&id));
        assert!(!is_tsid("not-a-tsid"));
        assert!(!is_tsid(""));
    }

    #[test]
    fn test_crockford_char_value() {
        assert_eq!(crockford_char_value('0'), Some(0));
        assert_eq!(crockford_char_value('O'), Some(0)); // alias
        assert_eq!(crockford_char_value('o'), Some(0)); // lowercase alias
        assert_eq!(crockford_char_value('1'), Some(1));
        assert_eq!(crockford_char_value('I'), Some(1)); // alias
        assert_eq!(crockford_char_value('L'), Some(1)); // alias
        assert_eq!(crockford_char_value('l'), Some(1)); // lowercase alias
        assert_eq!(crockford_char_value('Z'), Some(31));
        assert_eq!(crockford_char_value('z'), Some(31)); // lowercase
        assert_eq!(crockford_char_value('U'), None); // not in Crockford
        assert_eq!(crockford_char_value('!'), None);
    }

    #[test]
    fn test_random_bits() {
        let generator = TsidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedTsid::parse(&id).unwrap();
        // 22-bit random, should be within range
        assert!(parsed.random_bits() < (1 << 22));
    }
}

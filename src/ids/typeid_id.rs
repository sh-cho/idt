use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use serde_json::json;

/// Modified Crockford Base32 alphabet for TypeID (lowercase, no padding)
const TYPEID_ALPHABET: &[u8] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// TypeID generator
pub struct TypeIdGenerator {
    prefix: String,
}

impl TypeIdGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
}

impl IdGenerator for TypeIdGenerator {
    fn generate(&self) -> Result<String> {
        // Generate a UUIDv7
        let uuid = uuid::Uuid::now_v7();
        let bytes = uuid.as_bytes();
        let encoded = typeid_base32_encode(bytes);

        if self.prefix.is_empty() {
            Ok(encoded)
        } else {
            Ok(format!("{}_{}", self.prefix, encoded))
        }
    }
}

/// Encode 16 bytes as 26-char modified Crockford Base32 (TypeID-specific)
fn typeid_base32_encode(bytes: &[u8; 16]) -> String {
    // 128 bits -> 26 base32 chars (26 * 5 = 130 bits, 2 padding)
    // Encode from MSB to LSB
    let mut result = [0u8; 26];

    // Convert 16 bytes to 26 base32 characters
    // Process the 128-bit value as two u64s for easier manipulation
    let hi = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
    let lo = u64::from_be_bytes(bytes[8..16].try_into().unwrap());

    // Encode from least significant to most significant
    let mut val = (hi as u128) << 64 | lo as u128;

    for i in (0..26).rev() {
        result[i] = TYPEID_ALPHABET[(val & 0x1F) as usize];
        val >>= 5;
    }

    String::from_utf8(result.to_vec()).unwrap()
}

/// Decode 26-char modified Crockford Base32 to 16 bytes
fn typeid_base32_decode(s: &str) -> Result<[u8; 16]> {
    if s.len() != 26 {
        return Err(IdtError::ParseError(
            "TypeID suffix must be 26 characters".to_string(),
        ));
    }

    let mut val: u128 = 0;
    for ch in s.chars() {
        let v = typeid_char_value(ch).ok_or_else(|| {
            IdtError::ParseError(format!("Invalid TypeID Base32 character: '{}'", ch))
        })?;
        val = (val << 5) | (v as u128);
    }

    Ok(val.to_be_bytes())
}

fn typeid_char_value(c: char) -> Option<u8> {
    match c {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'a' => Some(10),
        'b' => Some(11),
        'c' => Some(12),
        'd' => Some(13),
        'e' => Some(14),
        'f' => Some(15),
        'g' => Some(16),
        'h' => Some(17),
        'j' => Some(18),
        'k' => Some(19),
        'm' => Some(20),
        'n' => Some(21),
        'p' => Some(22),
        'q' => Some(23),
        'r' => Some(24),
        's' => Some(25),
        't' => Some(26),
        'v' => Some(27),
        'w' => Some(28),
        'x' => Some(29),
        'y' => Some(30),
        'z' => Some(31),
        _ => None,
    }
}

/// Parsed TypeID value
pub struct ParsedTypeId {
    prefix: String,
    uuid_bytes: [u8; 16],
    input: String,
}

impl ParsedTypeId {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();

        // Find the last underscore to split prefix and suffix
        let (prefix, suffix) = if let Some(pos) = input_trimmed.rfind('_') {
            let p = &input_trimmed[..pos];
            let s = &input_trimmed[pos + 1..];

            // Validate prefix: lowercase letters and underscores only
            if !p.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                return Err(IdtError::ParseError(
                    "TypeID prefix must contain only lowercase letters and underscores".to_string(),
                ));
            }

            (p.to_string(), s)
        } else {
            // No prefix, just the base32 suffix
            (String::new(), input_trimmed)
        };

        let uuid_bytes = typeid_base32_decode(suffix)?;

        Ok(Self {
            prefix,
            uuid_bytes,
            input: input_trimmed.to_string(),
        })
    }

    fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.uuid_bytes)
    }

    fn timestamp_ms(&self) -> Option<u64> {
        // Extract timestamp from UUIDv7: first 48 bits are ms since epoch
        let uuid = self.uuid();
        let version = uuid.get_version_num();
        if version == 7 {
            let bytes = uuid.as_bytes();
            let ms = ((bytes[0] as u64) << 40)
                | ((bytes[1] as u64) << 32)
                | ((bytes[2] as u64) << 24)
                | ((bytes[3] as u64) << 16)
                | ((bytes[4] as u64) << 8)
                | (bytes[5] as u64);
            Some(ms)
        } else {
            None
        }
    }
}

impl ParsedId for ParsedTypeId {
    fn kind(&self) -> IdKind {
        IdKind::TypeId
    }

    fn canonical(&self) -> String {
        let suffix = typeid_base32_encode(&self.uuid_bytes);
        if self.prefix.is_empty() {
            suffix
        } else {
            format!("{}_{}", self.prefix, suffix)
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.uuid_bytes.to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp_ms().map(Timestamp::new)
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let timestamp = self.timestamp();
        let uuid = self.uuid();

        let components = json!({
            "prefix": self.prefix,
            "uuid": uuid.to_string(),
            "uuid_version": uuid.get_version_num(),
            "timestamp_ms": self.timestamp_ms(),
        });

        InspectionResult {
            id_type: "typeid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp,
            timestamp_iso: timestamp.as_ref().map(|ts| ts.to_iso8601()),
            timestamp_local_iso: None,
            version: Some(format!("UUIDv{}", uuid.get_version_num())),
            variant: Some(self.prefix.clone()),
            random_bits: Some(62),
            components: Some(components),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: encode_base32(&bytes),
                base58: encode_base58(&bytes),
                base64: encode_base64(&bytes),
                int: Some(u128::from_be_bytes(self.uuid_bytes).to_string()),
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        // Check that the embedded UUID is valid v7
        let uuid = self.uuid();
        let version = uuid.get_version_num();
        if version != 7 {
            ValidationResult::valid("typeid")
                .with_hint(&format!("Embedded UUID is v{}, expected v7", version))
        } else {
            ValidationResult::valid("typeid")
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
            EncodingFormat::Int => u128::from_be_bytes(self.uuid_bytes).to_string(),
            EncodingFormat::Bytes => encode_bytes_spaced(&bytes),
        }
    }
}

/// Check if a string looks like a TypeID
pub fn is_typeid(input: &str) -> bool {
    ParsedTypeId::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_with_prefix() {
        let generator = TypeIdGenerator::new("user");
        let id = generator.generate().unwrap();
        assert!(id.starts_with("user_"));
        assert_eq!(id.len(), 5 + 26); // prefix + _ + 26 base32 chars
    }

    #[test]
    fn test_generate_no_prefix() {
        let generator = TypeIdGenerator::new("");
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 26);
    }

    #[test]
    fn test_roundtrip() {
        let generator = TypeIdGenerator::new("order");
        let id = generator.generate().unwrap();
        let parsed = ParsedTypeId::parse(&id).unwrap();
        assert_eq!(parsed.canonical(), id);
        assert_eq!(parsed.prefix, "order");
    }

    #[test]
    fn test_has_timestamp() {
        let generator = TypeIdGenerator::new("test");
        let id = generator.generate().unwrap();
        let parsed = ParsedTypeId::parse(&id).unwrap();
        assert!(parsed.timestamp().is_some());
    }

    #[test]
    fn test_base32_encode_decode() {
        let uuid = uuid::Uuid::now_v7();
        let bytes = *uuid.as_bytes();
        let encoded = typeid_base32_encode(&bytes);
        let decoded = typeid_base32_decode(&encoded).unwrap();
        assert_eq!(bytes, decoded);
    }
}

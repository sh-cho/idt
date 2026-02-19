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

/// Xid base32hex alphabet
const XID_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuv";

/// 3-byte machine ID (random, fixed per process)
static MACHINE_ID: OnceLock<[u8; 3]> = OnceLock::new();

/// Counter starting from a random value
static XID_COUNTER: AtomicU32 = AtomicU32::new(0);
static XID_COUNTER_INIT: OnceLock<()> = OnceLock::new();

fn machine_id() -> &'static [u8; 3] {
    MACHINE_ID.get_or_init(|| {
        let mut rng = rand::thread_rng();
        let mut buf = [0u8; 3];
        rng.fill(&mut buf);
        buf
    })
}

fn next_xid_counter() -> u32 {
    XID_COUNTER_INIT.get_or_init(|| {
        let mut rng = rand::thread_rng();
        XID_COUNTER.store(rng.r#gen::<u32>() & 0xFF_FFFF, Ordering::SeqCst);
    });
    XID_COUNTER.fetch_add(1, Ordering::SeqCst) & 0xFF_FFFF
}

/// Xid generator
pub struct XidGenerator;

impl XidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for XidGenerator {
    fn generate(&self) -> Result<String> {
        let now = chrono::Utc::now().timestamp() as u32;
        let mid = machine_id();
        let pid = (std::process::id() & 0xFFFF) as u16;
        let counter = next_xid_counter();

        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&now.to_be_bytes());
        bytes[4..7].copy_from_slice(mid);
        bytes[7..9].copy_from_slice(&pid.to_be_bytes());
        bytes[9] = ((counter >> 16) & 0xFF) as u8;
        bytes[10] = ((counter >> 8) & 0xFF) as u8;
        bytes[11] = (counter & 0xFF) as u8;

        Ok(xid_encode(&bytes))
    }
}

/// Xid-specific base32hex encoding (custom bit extraction, NOT standard base32)
fn xid_encode(bytes: &[u8; 12]) -> String {
    let mut dst = [0u8; 20];

    // Xid encodes 12 bytes (96 bits) into 20 base32hex chars (5 bits each = 100 bits, 4 padding)
    // Using specific bit extraction pattern from the xid spec
    dst[19] = XID_ALPHABET[(bytes[11] & 0x1f) as usize];
    dst[18] = XID_ALPHABET[((bytes[11] >> 5) | (bytes[10] << 3)) as usize & 0x1f];
    dst[17] = XID_ALPHABET[((bytes[10] >> 2) & 0x1f) as usize];
    dst[16] = XID_ALPHABET[((bytes[10] >> 7) | (bytes[9] << 1)) as usize & 0x1f];
    dst[15] = XID_ALPHABET[((bytes[9] >> 4) | (bytes[8] << 4)) as usize & 0x1f];

    dst[14] = XID_ALPHABET[(bytes[8] >> 1 & 0x1f) as usize];
    dst[13] = XID_ALPHABET[((bytes[8] >> 6) | (bytes[7] << 2)) as usize & 0x1f];
    dst[12] = XID_ALPHABET[((bytes[7] >> 3) & 0x1f) as usize];

    dst[11] = XID_ALPHABET[(bytes[6] & 0x1f) as usize];
    dst[10] = XID_ALPHABET[((bytes[6] >> 5) | (bytes[5] << 3)) as usize & 0x1f];
    dst[9] = XID_ALPHABET[((bytes[5] >> 2) & 0x1f) as usize];
    dst[8] = XID_ALPHABET[((bytes[5] >> 7) | (bytes[4] << 1)) as usize & 0x1f];
    dst[7] = XID_ALPHABET[((bytes[4] >> 4) | (bytes[3] << 4)) as usize & 0x1f];

    dst[6] = XID_ALPHABET[((bytes[3] >> 1) & 0x1f) as usize];
    dst[5] = XID_ALPHABET[((bytes[3] >> 6) | (bytes[2] << 2)) as usize & 0x1f];
    dst[4] = XID_ALPHABET[((bytes[2] >> 3) & 0x1f) as usize];

    dst[3] = XID_ALPHABET[(bytes[1] & 0x1f) as usize];
    dst[2] = XID_ALPHABET[((bytes[1] >> 5) | (bytes[0] << 3)) as usize & 0x1f];
    dst[1] = XID_ALPHABET[((bytes[0] >> 2) & 0x1f) as usize];
    dst[0] = XID_ALPHABET[(bytes[0] >> 7) as usize];

    String::from_utf8(dst.to_vec()).unwrap()
}

/// Decode xid base32hex to 12 bytes
fn xid_decode(s: &str) -> Result<[u8; 12]> {
    if s.len() != 20 {
        return Err(IdtError::ParseError("Xid must be 20 characters".to_string()));
    }

    let src: Vec<u8> = s
        .chars()
        .map(|c| xid_char_value(c).ok_or_else(|| IdtError::ParseError(format!("Invalid xid character: '{}'", c))))
        .collect::<Result<Vec<u8>>>()?;

    let mut bytes = [0u8; 12];

    bytes[11] = (src[19]) | (src[18] << 5);
    bytes[10] = (src[18] >> 3) | (src[17] << 2) | (src[16] << 7);
    bytes[9] = (src[16] >> 1) | (src[15] << 4);
    bytes[8] = (src[15] >> 4) | (src[14] << 1) | (src[13] << 6);
    bytes[7] = (src[13] >> 2) | (src[12] << 3);

    bytes[6] = (src[11]) | (src[10] << 5);
    bytes[5] = (src[10] >> 3) | (src[9] << 2) | (src[8] << 7);
    bytes[4] = (src[8] >> 1) | (src[7] << 4);
    bytes[3] = (src[7] >> 4) | (src[6] << 1) | (src[5] << 6);
    bytes[2] = (src[5] >> 2) | (src[4] << 3);

    bytes[1] = (src[3]) | (src[2] << 5);
    bytes[0] = (src[2] >> 3) | (src[1] << 2) | (src[0] << 7);

    Ok(bytes)
}

fn xid_char_value(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='v' => Some(c as u8 - b'a' + 10),
        _ => None,
    }
}

/// Parsed Xid value
pub struct ParsedXid {
    bytes: [u8; 12],
    input: String,
}

impl ParsedXid {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();
        let bytes = xid_decode(input_trimmed)?;
        Ok(Self {
            bytes,
            input: input_trimmed.to_string(),
        })
    }

    fn timestamp_secs(&self) -> u32 {
        u32::from_be_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    fn machine_id_bytes(&self) -> &[u8] {
        &self.bytes[4..7]
    }

    fn process_id(&self) -> u16 {
        u16::from_be_bytes([self.bytes[7], self.bytes[8]])
    }

    fn counter(&self) -> u32 {
        ((self.bytes[9] as u32) << 16) | ((self.bytes[10] as u32) << 8) | (self.bytes[11] as u32)
    }
}

impl ParsedId for ParsedXid {
    fn kind(&self) -> IdKind {
        IdKind::Xid
    }

    fn canonical(&self) -> String {
        xid_encode(&self.bytes)
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
            "machine_id_hex": encode_hex(self.machine_id_bytes()),
            "process_id": self.process_id(),
            "counter": self.counter(),
        });

        InspectionResult {
            id_type: "xid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: Some(timestamp),
            timestamp_iso: Some(timestamp.to_iso8601()),
            timestamp_local_iso: None,
            version: None,
            variant: None,
            random_bits: None,
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
            ValidationResult::valid("xid")
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

/// Check if a string looks like an Xid
pub fn is_xid(input: &str) -> bool {
    ParsedXid::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = XidGenerator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 20);
        assert!(id.chars().all(|c| matches!(c, '0'..='9' | 'a'..='v')));
    }

    #[test]
    fn test_roundtrip() {
        let generator = XidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedXid::parse(&id).unwrap();
        assert_eq!(parsed.canonical(), id);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let mut bytes = [0u8; 12];
        for i in 0..12 {
            bytes[i] = (i * 17 + 3) as u8;
        }
        let encoded = xid_encode(&bytes);
        let decoded = xid_decode(&encoded).unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn test_has_timestamp() {
        let generator = XidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedXid::parse(&id).unwrap();
        let ts = parsed.timestamp().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        assert!((now * 1000).abs_diff(ts.millis) < 10_000);
    }
}

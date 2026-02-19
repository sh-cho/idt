use crate::core::encoding::{encode_base64, encode_hex, EncodingFormat};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use rand::Rng;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};

/// Base36 alphabet
const BASE36: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// CUID counter
static CUID_COUNTER: AtomicU32 = AtomicU32::new(0);

/// CUID v1 generator
pub struct CuidGenerator;

impl CuidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for CuidGenerator {
    fn generate(&self) -> Result<String> {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let counter = CUID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let mut rng = rand::thread_rng();

        // CUID v1: c + base36(timestamp, 8) + base36(counter, 4) + base36(fingerprint, 4) + base36(random, 8)
        let ts_str = pad_base36(now_ms, 8);
        let counter_str = pad_base36(counter as u64, 4);

        // Fingerprint: combination of pid and hostname hash
        let pid = std::process::id() as u64;
        let hostname_hash: u64 = {
            let name = "localhost"; // Simplified fingerprint
            let mut h: u64 = 0;
            for b in name.bytes() {
                h = h.wrapping_mul(36).wrapping_add(b as u64);
            }
            h
        };
        let fingerprint = pad_base36(pid.wrapping_add(hostname_hash), 4);

        let random_val: u64 = rng.r#gen::<u64>() % 36u64.pow(8);
        let random_str = pad_base36(random_val, 8);

        Ok(format!("c{}{}{}{}", ts_str, counter_str, fingerprint, random_str))
    }
}

/// Encode u64 as base36 string, left-padded to `width` characters
fn pad_base36(mut value: u64, width: usize) -> String {
    if value == 0 {
        return "0".repeat(width);
    }

    let mut result = Vec::new();
    while value > 0 {
        result.push(BASE36[(value % 36) as usize]);
        value /= 36;
    }
    result.reverse();

    let s = String::from_utf8(result).unwrap();
    if s.len() >= width {
        s[s.len() - width..].to_string()
    } else {
        format!("{:0>width$}", s, width = width)
    }
}

/// Decode base36 string to u64
fn decode_base36(s: &str) -> Option<u64> {
    let mut value: u64 = 0;
    for ch in s.chars() {
        let v = match ch {
            '0'..='9' => (ch as u64) - ('0' as u64),
            'a'..='z' => (ch as u64) - ('a' as u64) + 10,
            _ => return None,
        };
        value = value.checked_mul(36)?.checked_add(v)?;
    }
    Some(value)
}

/// Parsed CUID v1 value
pub struct ParsedCuid {
    value: String,
    input: String,
}

impl ParsedCuid {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();

        if input_trimmed.len() != 25 {
            return Err(IdtError::ParseError(
                "CUID must be 25 characters".to_string(),
            ));
        }
        if !input_trimmed.starts_with('c') {
            return Err(IdtError::ParseError(
                "CUID must start with 'c'".to_string(),
            ));
        }
        if !input_trimmed.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(IdtError::ParseError(
                "CUID must contain only lowercase alphanumeric characters".to_string(),
            ));
        }

        Ok(Self {
            value: input_trimmed.to_string(),
            input: input_trimmed.to_string(),
        })
    }

    fn timestamp_ms(&self) -> Option<u64> {
        // Chars 1..9 are base36 timestamp
        decode_base36(&self.value[1..9])
    }

    fn counter_str(&self) -> &str {
        &self.value[9..13]
    }

    fn fingerprint_str(&self) -> &str {
        &self.value[13..17]
    }

    fn random_str(&self) -> &str {
        &self.value[17..25]
    }
}

impl ParsedId for ParsedCuid {
    fn kind(&self) -> IdKind {
        IdKind::Cuid
    }

    fn canonical(&self) -> String {
        self.value.clone()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.value.as_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp_ms().map(Timestamp::new)
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let timestamp = self.timestamp();

        let components = json!({
            "timestamp_ms": self.timestamp_ms(),
            "counter": self.counter_str(),
            "fingerprint": self.fingerprint_str(),
            "random": self.random_str(),
        });

        InspectionResult {
            id_type: "cuid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp,
            timestamp_iso: timestamp.as_ref().map(|ts| ts.to_iso8601()),
            timestamp_local_iso: None,
            version: Some("1".to_string()),
            variant: None,
            random_bits: None,
            components: Some(components),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: String::new(),
                base58: String::new(),
                base64: encode_base64(&bytes),
                int: None,
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        ValidationResult::valid("cuid")
            .with_hint("CUID v1 is deprecated; consider CUID2")
    }

    fn encode(&self, format: EncodingFormat) -> String {
        let bytes = self.as_bytes();
        match format {
            EncodingFormat::Canonical => self.canonical(),
            EncodingFormat::Hex => encode_hex(&bytes),
            EncodingFormat::Base64 => encode_base64(&bytes),
            _ => self.canonical(),
        }
    }
}

/// Check if a string looks like a CUID v1
pub fn is_cuid(input: &str) -> bool {
    ParsedCuid::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = CuidGenerator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 25);
        assert!(id.starts_with('c'));
        assert!(id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_parse() {
        let generator = CuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedCuid::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::Cuid);
        assert!(parsed.timestamp().is_some());
    }

    #[test]
    fn test_validate_hint() {
        let generator = CuidGenerator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedCuid::parse(&id).unwrap();
        let result = parsed.validate();
        assert!(result.valid);
        assert!(result.hint.as_ref().unwrap().contains("deprecated"));
    }

    #[test]
    fn test_pad_base36() {
        assert_eq!(pad_base36(0, 4), "0000");
        assert_eq!(pad_base36(35, 4), "000z");
        assert_eq!(pad_base36(36, 4), "0010");
    }
}

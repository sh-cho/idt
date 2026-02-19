use crate::core::encoding::{encode_base64, encode_hex, EncodingFormat};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use rand::Rng;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

/// Default CUID2 length
const DEFAULT_LENGTH: usize = 24;

/// CUID2 counter
static CUID2_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Base36 alphabet
const BASE36: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// CUID2 generator
pub struct Cuid2Generator {
    length: usize,
}

impl Cuid2Generator {
    pub fn new() -> Self {
        Self {
            length: DEFAULT_LENGTH,
        }
    }
}

impl IdGenerator for Cuid2Generator {
    fn generate(&self) -> Result<String> {
        let mut rng = rand::thread_rng();

        // Gather entropy sources
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        let counter = CUID2_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Generate random salt
        let salt: u64 = rng.r#gen();

        // Fingerprint from pid
        let pid = std::process::id() as u64;

        // Additional random data
        let random1: u64 = rng.r#gen();
        let random2: u64 = rng.r#gen();

        // Hash all entropy together
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_le_bytes());
        hasher.update(counter.to_le_bytes());
        hasher.update(salt.to_le_bytes());
        hasher.update(pid.to_le_bytes());
        hasher.update(random1.to_le_bytes());
        hasher.update(random2.to_le_bytes());
        let hash = hasher.finalize();

        // Convert hash to base36
        let base36_str = bytes_to_base36(&hash);

        // Take the desired length, ensure first char is a letter
        let mut result: String = base36_str.chars().take(self.length).collect();

        // Ensure first character is a letter (a-z)
        if let Some(first) = result.chars().next() {
            if first.is_ascii_digit() {
                let letter = (b'a' + (first as u8 - b'0') % 26) as char;
                result.replace_range(0..1, &letter.to_string());
            }
        }

        // Pad if needed
        while result.len() < self.length {
            let extra: u8 = rng.r#gen();
            result.push(BASE36[(extra % 36) as usize] as char);
        }

        Ok(result)
    }
}

/// Convert bytes to base36 string
fn bytes_to_base36(bytes: &[u8]) -> String {
    // Convert bytes to a big number and then to base36
    let mut num = bytes.to_vec();
    let mut result = Vec::new();

    loop {
        let mut all_zero = true;
        let mut remainder: u16 = 0;
        for byte in num.iter_mut() {
            let acc = (remainder << 8) | (*byte as u16);
            *byte = (acc / 36) as u8;
            remainder = acc % 36;
            if *byte != 0 {
                all_zero = false;
            }
        }
        result.push(BASE36[remainder as usize]);
        if all_zero {
            break;
        }
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Parsed CUID2 value
pub struct ParsedCuid2 {
    value: String,
    input: String,
}

impl ParsedCuid2 {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();

        // CUID2 default length is 24, starts with a letter, all lowercase alphanumeric
        if input_trimmed.is_empty() {
            return Err(IdtError::ParseError("Empty CUID2".to_string()));
        }

        let first = input_trimmed.chars().next().unwrap();
        if !first.is_ascii_lowercase() {
            return Err(IdtError::ParseError(
                "CUID2 must start with a lowercase letter".to_string(),
            ));
        }

        if !input_trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err(IdtError::ParseError(
                "CUID2 must contain only lowercase alphanumeric characters".to_string(),
            ));
        }

        Ok(Self {
            value: input_trimmed.to_string(),
            input: input_trimmed.to_string(),
        })
    }
}

impl ParsedId for ParsedCuid2 {
    fn kind(&self) -> IdKind {
        IdKind::Cuid2
    }

    fn canonical(&self) -> String {
        self.value.clone()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.value.as_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        // CUID2 is opaque - no extractable timestamp
        None
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();

        let components = json!({
            "length": self.value.len(),
            "note": "CUID2 is opaque — no components extractable",
        });

        InspectionResult {
            id_type: "cuid2".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: None,
            timestamp_iso: None,
            timestamp_local_iso: None,
            version: Some("2".to_string()),
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
        if self.value.len() == DEFAULT_LENGTH {
            ValidationResult::valid("cuid2")
        } else {
            ValidationResult::valid("cuid2")
                .with_hint(&format!("Non-standard length: {} (default is {})", self.value.len(), DEFAULT_LENGTH))
        }
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

/// Check if a string looks like a CUID2
pub fn is_cuid2(input: &str) -> bool {
    ParsedCuid2::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = Cuid2Generator::new();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), DEFAULT_LENGTH);
        assert!(id.chars().next().unwrap().is_ascii_lowercase());
        assert!(id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_parse() {
        let generator = Cuid2Generator::new();
        let id = generator.generate().unwrap();
        let parsed = ParsedCuid2::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::Cuid2);
        assert!(parsed.timestamp().is_none()); // CUID2 is opaque
    }

    #[test]
    fn test_uniqueness() {
        let generator = Cuid2Generator::new();
        let ids: Vec<String> = (0..100).map(|_| generator.generate().unwrap()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }
}

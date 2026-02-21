use crate::core::encoding::{EncodingFormat, encode_base64, encode_bits, encode_hex};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

/// Twitter Snowflake epoch (Nov 04, 2010 01:42:54 UTC) in milliseconds
pub const TWITTER_EPOCH: u64 = 1288834974657;

/// Discord epoch (Jan 01, 2015 00:00:00 UTC) in milliseconds
pub const DISCORD_EPOCH: u64 = 1420070400000;

/// Default epoch (Unix epoch)
pub const DEFAULT_EPOCH: u64 = 0;

/// Snowflake ID structure:
/// - 1 bit: sign (always 0)
/// - 41 bits: timestamp (milliseconds since epoch)
/// - 10 bits: machine ID (5 bits datacenter + 5 bits worker)
/// - 12 bits: sequence number
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// Snowflake generator
pub struct SnowflakeGenerator {
    pub epoch: u64,
    pub machine_id: u16,
    pub datacenter_id: u16,
}

impl Default for SnowflakeGenerator {
    fn default() -> Self {
        Self {
            epoch: DEFAULT_EPOCH,
            machine_id: 0,
            datacenter_id: 0,
        }
    }
}

impl SnowflakeGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn twitter() -> Self {
        Self {
            epoch: TWITTER_EPOCH,
            machine_id: 0,
            datacenter_id: 0,
        }
    }

    pub fn discord() -> Self {
        Self {
            epoch: DISCORD_EPOCH,
            machine_id: 0,
            datacenter_id: 0,
        }
    }

    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.epoch = epoch;
        self
    }

    pub fn with_machine_id(mut self, machine_id: u16) -> Self {
        self.machine_id = machine_id & 0x1F; // 5 bits max
        self
    }

    pub fn with_datacenter_id(mut self, datacenter_id: u16) -> Self {
        self.datacenter_id = datacenter_id & 0x1F; // 5 bits max
        self
    }

    fn current_timestamp(&self) -> u64 {
        chrono::Utc::now().timestamp_millis() as u64 - self.epoch
    }

    fn next_sequence(&self, timestamp: u64) -> u64 {
        let last = LAST_TIMESTAMP.swap(timestamp, Ordering::SeqCst);
        if timestamp == last {
            // Same millisecond, increment sequence
            SEQUENCE.fetch_add(1, Ordering::SeqCst) & 0xFFF
        } else {
            // New millisecond, reset sequence
            SEQUENCE.store(1, Ordering::SeqCst);
            0
        }
    }
}

impl IdGenerator for SnowflakeGenerator {
    fn generate(&self) -> Result<String> {
        let timestamp = self.current_timestamp();
        let sequence = self.next_sequence(timestamp);

        // Build Snowflake ID
        let id: u64 = (timestamp << 22)
            | ((self.datacenter_id as u64 & 0x1F) << 17)
            | ((self.machine_id as u64 & 0x1F) << 12)
            | (sequence & 0xFFF);

        Ok(id.to_string())
    }
}

/// Parsed Snowflake ID
pub struct ParsedSnowflake {
    id: u64,
    epoch: u64,
    input: String,
}

impl ParsedSnowflake {
    pub fn parse(input: &str) -> Result<Self> {
        Self::parse_with_epoch(input, DEFAULT_EPOCH)
    }

    pub fn parse_twitter(input: &str) -> Result<Self> {
        Self::parse_with_epoch(input, TWITTER_EPOCH)
    }

    pub fn parse_discord(input: &str) -> Result<Self> {
        Self::parse_with_epoch(input, DISCORD_EPOCH)
    }

    pub fn parse_with_epoch(input: &str, epoch: u64) -> Result<Self> {
        let input_trimmed = input.trim();

        let id = input_trimmed
            .parse::<u64>()
            .map_err(|e| IdtError::ParseError(format!("Invalid Snowflake ID: {}", e)))?;

        Ok(Self {
            id,
            epoch,
            input: input_trimmed.to_string(),
        })
    }

    pub fn timestamp_ms(&self) -> u64 {
        (self.id >> 22) + self.epoch
    }

    pub fn datacenter_id(&self) -> u16 {
        ((self.id >> 17) & 0x1F) as u16
    }

    pub fn machine_id(&self) -> u16 {
        ((self.id >> 12) & 0x1F) as u16
    }

    pub fn sequence(&self) -> u16 {
        (self.id & 0xFFF) as u16
    }
}

impl ParsedId for ParsedSnowflake {
    fn kind(&self) -> IdKind {
        IdKind::Snowflake
    }

    fn canonical(&self) -> String {
        self.id.to_string()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.id.to_be_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        Some(Timestamp::new(self.timestamp_ms()))
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let timestamp = self.timestamp().unwrap();

        let components = json!({
            "timestamp_ms": self.timestamp_ms(),
            "datacenter_id": self.datacenter_id(),
            "machine_id": self.machine_id(),
            "sequence": self.sequence(),
            "epoch": self.epoch,
        });

        InspectionResult {
            id_type: "snowflake".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: Some(timestamp),
            timestamp_iso: Some(timestamp.to_iso8601()),
            timestamp_local_iso: None,
            version: None,
            variant: if self.epoch == TWITTER_EPOCH {
                Some("Twitter".to_string())
            } else if self.epoch == DISCORD_EPOCH {
                Some("Discord".to_string())
            } else {
                Some("Custom".to_string())
            },
            random_bits: None, // Snowflake doesn't have random bits
            components: Some(components),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: String::new(),
                base58: String::new(),
                base64: encode_base64(&bytes),
                int: Some(self.id.to_string()),
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        // Basic validation: check timestamp is reasonable
        let ts = self.timestamp_ms();
        let now = chrono::Utc::now().timestamp_millis() as u64;

        if ts > now + 86400000 {
            // More than 1 day in the future
            ValidationResult::invalid("Timestamp is in the future")
        } else {
            ValidationResult::valid("snowflake")
        }
    }

    fn encode(&self, format: EncodingFormat) -> String {
        let bytes = self.as_bytes();
        match format {
            EncodingFormat::Canonical => self.canonical(),
            EncodingFormat::Hex => encode_hex(&bytes),
            EncodingFormat::Base64 => encode_base64(&bytes),
            EncodingFormat::Bits => encode_bits(&bytes),
            EncodingFormat::Int => self.id.to_string(),
            _ => self.canonical(),
        }
    }
}

/// Check if a string looks like a Snowflake ID
pub fn is_snowflake(input: &str) -> bool {
    let input = input.trim();
    if input.is_empty() {
        return false;
    }
    if !input.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let len = input.len();
    (15..=19).contains(&len) && input.parse::<u64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let generator = SnowflakeGenerator::new();
        let id = generator.generate().unwrap();
        assert!(is_snowflake(&id));
    }

    #[test]
    fn test_generate_twitter() {
        let generator = SnowflakeGenerator::twitter().with_machine_id(1);
        let id = generator.generate().unwrap();
        let parsed = ParsedSnowflake::parse_twitter(&id).unwrap();
        assert_eq!(parsed.machine_id(), 1);
    }

    #[test]
    fn test_parse_components() {
        // Example Twitter Snowflake ID
        let id = "1234567890123456789";
        let parsed = ParsedSnowflake::parse(id).unwrap();

        assert!(parsed.timestamp_ms() > 0);
        assert!(parsed.datacenter_id() < 32);
        assert!(parsed.machine_id() < 32);
        assert!(parsed.sequence() < 4096);
    }

    #[test]
    fn test_uniqueness() {
        let generator = SnowflakeGenerator::new();
        let ids: Vec<String> = (0..100).map(|_| generator.generate().unwrap()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }
}

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

/// Instagram epoch in milliseconds
pub const INSTAGRAM_EPOCH: u64 = 1314220021721;

/// Sonyflake epoch (Aug 01, 2014 00:00:00 UTC) in milliseconds
pub const SONYFLAKE_EPOCH: u64 = 1409529600000;

/// Default epoch (Unix epoch)
pub const DEFAULT_EPOCH: u64 = 0;

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// Timestamp resolution unit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampUnit {
    Millis,
    TenMillis,
    Seconds,
}

/// A single field in a Snowflake layout
#[derive(Debug, Clone)]
pub struct SnowflakeField {
    pub name: &'static str,
    pub bits: u8,
}

/// Describes the complete bit layout, epoch, and timestamp resolution
#[derive(Debug, Clone)]
pub struct SnowflakeLayout {
    pub name: &'static str,
    pub epoch: u64,
    pub timestamp_unit: TimestampUnit,
    pub fields: &'static [SnowflakeField],
}

// Static field arrays for built-in presets
static TWITTER_FIELDS: &[SnowflakeField] = &[
    SnowflakeField {
        name: "timestamp",
        bits: 41,
    },
    SnowflakeField {
        name: "datacenter_id",
        bits: 5,
    },
    SnowflakeField {
        name: "machine_id",
        bits: 5,
    },
    SnowflakeField {
        name: "sequence",
        bits: 12,
    },
];

static INSTAGRAM_FIELDS: &[SnowflakeField] = &[
    SnowflakeField {
        name: "timestamp",
        bits: 41,
    },
    SnowflakeField {
        name: "shard_id",
        bits: 13,
    },
    SnowflakeField {
        name: "sequence",
        bits: 10,
    },
];

static SONYFLAKE_FIELDS: &[SnowflakeField] = &[
    SnowflakeField {
        name: "timestamp",
        bits: 39,
    },
    SnowflakeField {
        name: "sequence",
        bits: 8,
    },
    SnowflakeField {
        name: "machine_id",
        bits: 16,
    },
];

static MASTODON_FIELDS: &[SnowflakeField] = &[
    SnowflakeField {
        name: "timestamp",
        bits: 48,
    },
    SnowflakeField {
        name: "sequence",
        bits: 16,
    },
];

impl SnowflakeLayout {
    pub fn twitter() -> Self {
        Self {
            name: "twitter",
            epoch: TWITTER_EPOCH,
            timestamp_unit: TimestampUnit::Millis,
            fields: TWITTER_FIELDS,
        }
    }

    pub fn discord() -> Self {
        Self {
            name: "discord",
            epoch: DISCORD_EPOCH,
            timestamp_unit: TimestampUnit::Millis,
            fields: TWITTER_FIELDS,
        }
    }

    pub fn instagram() -> Self {
        Self {
            name: "instagram",
            epoch: INSTAGRAM_EPOCH,
            timestamp_unit: TimestampUnit::Millis,
            fields: INSTAGRAM_FIELDS,
        }
    }

    pub fn sonyflake() -> Self {
        Self {
            name: "sonyflake",
            epoch: SONYFLAKE_EPOCH,
            timestamp_unit: TimestampUnit::TenMillis,
            fields: SONYFLAKE_FIELDS,
        }
    }

    pub fn mastodon() -> Self {
        Self {
            name: "mastodon",
            epoch: DEFAULT_EPOCH,
            timestamp_unit: TimestampUnit::Millis,
            fields: MASTODON_FIELDS,
        }
    }

    /// Default layout: Unix epoch, Twitter bit layout
    pub fn default_layout() -> Self {
        Self {
            name: "custom",
            epoch: DEFAULT_EPOCH,
            timestamp_unit: TimestampUnit::Millis,
            fields: TWITTER_FIELDS,
        }
    }

    /// Look up a preset by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "twitter" => Some(Self::twitter()),
            "discord" => Some(Self::discord()),
            "instagram" => Some(Self::instagram()),
            "sonyflake" => Some(Self::sonyflake()),
            "mastodon" => Some(Self::mastodon()),
            _ => None,
        }
    }

    /// Resolve layout from --preset and --epoch flags
    pub fn resolve(preset: Option<&str>, epoch: Option<&str>) -> Result<Self> {
        match (preset, epoch) {
            (Some(_), Some(_)) => Err(IdtError::InvalidArgument(
                "Cannot use both --preset and --epoch".to_string(),
            )),
            (Some(name), None) => Self::by_name(name).ok_or_else(|| {
                IdtError::InvalidArgument(format!(
                    "Unknown preset '{}'. Available: twitter, discord, instagram, sonyflake, mastodon",
                    name
                ))
            }),
            (None, Some(e)) => {
                // backward compat: "twitter"/"discord" as epoch-only override
                let mut layout = Self::default_layout();
                match e.to_lowercase().as_str() {
                    "twitter" => {
                        layout.epoch = TWITTER_EPOCH;
                        layout.name = "twitter";
                    }
                    "discord" => {
                        layout.epoch = DISCORD_EPOCH;
                        layout.name = "discord";
                    }
                    _ => {
                        layout.epoch = e.parse::<u64>().map_err(|_| {
                            IdtError::InvalidArgument(format!(
                                "Invalid epoch '{}': use 'discord', 'twitter', or milliseconds since Unix epoch",
                                e
                            ))
                        })?;
                    }
                }
                Ok(layout)
            }
            (None, None) => Ok(Self::default_layout()),
        }
    }

    /// Get the bit width for a named field
    pub fn field_bits(&self, name: &str) -> Option<u8> {
        self.fields.iter().find(|f| f.name == name).map(|f| f.bits)
    }

    /// Compute the bit offset (from LSB) for a named field
    fn field_offset(&self, name: &str) -> Option<u8> {
        let mut offset = 0u8;
        for f in self.fields.iter().rev() {
            if f.name == name {
                return Some(offset);
            }
            offset += f.bits;
        }
        None
    }

    /// Extract a field value from an ID
    pub fn extract_field(&self, id: u64, name: &str) -> Option<u64> {
        let bits = self.field_bits(name)?;
        let offset = self.field_offset(name)?;
        let mask = (1u64 << bits) - 1;
        Some((id >> offset) & mask)
    }

    /// Check if the layout has a field with the given name
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f.name == name)
    }
}

/// Snowflake generator
pub struct SnowflakeGenerator {
    pub layout: SnowflakeLayout,
    pub field_values: std::collections::HashMap<String, u64>,
}

impl Default for SnowflakeGenerator {
    fn default() -> Self {
        Self {
            layout: SnowflakeLayout::default_layout(),
            field_values: std::collections::HashMap::new(),
        }
    }
}

impl SnowflakeGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn twitter() -> Self {
        Self {
            layout: SnowflakeLayout::twitter(),
            field_values: std::collections::HashMap::new(),
        }
    }

    pub fn discord() -> Self {
        Self {
            layout: SnowflakeLayout::discord(),
            field_values: std::collections::HashMap::new(),
        }
    }

    pub fn with_layout(mut self, layout: SnowflakeLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.layout.epoch = epoch;
        self
    }

    pub fn with_machine_id(mut self, machine_id: u16) -> Self {
        self.field_values
            .insert("machine_id".to_string(), machine_id as u64);
        self
    }

    pub fn with_datacenter_id(mut self, datacenter_id: u16) -> Self {
        self.field_values
            .insert("datacenter_id".to_string(), datacenter_id as u64);
        self
    }

    pub fn with_field(mut self, name: &str, value: u64) -> Self {
        self.field_values.insert(name.to_string(), value);
        self
    }

    fn current_timestamp(&self) -> u64 {
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let elapsed_ms = now_ms - self.layout.epoch;
        match self.layout.timestamp_unit {
            TimestampUnit::Millis => elapsed_ms,
            TimestampUnit::TenMillis => elapsed_ms / 10,
            TimestampUnit::Seconds => elapsed_ms / 1000,
        }
    }

    fn next_sequence(&self, timestamp: u64, seq_bits: u8) -> u64 {
        let seq_mask = (1u64 << seq_bits) - 1;
        let last = LAST_TIMESTAMP.swap(timestamp, Ordering::SeqCst);
        if timestamp == last {
            SEQUENCE.fetch_add(1, Ordering::SeqCst) & seq_mask
        } else {
            SEQUENCE.store(1, Ordering::SeqCst);
            0
        }
    }
}

impl IdGenerator for SnowflakeGenerator {
    fn generate(&self) -> Result<String> {
        let timestamp = self.current_timestamp();

        let seq_bits = self.layout.field_bits("sequence")
            .expect("Snowflake layout must have a sequence field");
        let sequence = self.next_sequence(timestamp, seq_bits);

        // Build ID by iterating fields MSB→LSB
        let mut id: u64 = 0;
        let total_bits: u8 = self.layout.fields.iter().map(|f| f.bits).sum();
        let mut shift = total_bits;

        for field in self.layout.fields {
            shift -= field.bits;
            let mask = (1u64 << field.bits) - 1;
            let value = if field.name == "timestamp" {
                timestamp & mask
            } else if field.name == "sequence" {
                sequence & mask
            } else {
                self.field_values.get(field.name).copied().unwrap_or(0) & mask
            };
            id |= value << shift;
        }

        Ok(id.to_string())
    }
}

/// Parsed Snowflake ID
pub struct ParsedSnowflake {
    id: u64,
    layout: SnowflakeLayout,
    input: String,
}

impl ParsedSnowflake {
    pub fn parse(input: &str) -> Result<Self> {
        Self::parse_with_layout(input, SnowflakeLayout::default_layout())
    }

    pub fn parse_twitter(input: &str) -> Result<Self> {
        Self::parse_with_layout(input, SnowflakeLayout::twitter())
    }

    pub fn parse_discord(input: &str) -> Result<Self> {
        Self::parse_with_layout(input, SnowflakeLayout::discord())
    }

    pub fn parse_with_epoch(input: &str, epoch: u64) -> Result<Self> {
        let mut layout = SnowflakeLayout::default_layout();
        layout.epoch = epoch;
        if epoch == TWITTER_EPOCH {
            layout.name = "twitter";
        } else if epoch == DISCORD_EPOCH {
            layout.name = "discord";
        }
        Self::parse_with_layout(input, layout)
    }

    pub fn parse_with_layout(input: &str, layout: SnowflakeLayout) -> Result<Self> {
        let input_trimmed = input.trim();

        let id = input_trimmed
            .parse::<u64>()
            .map_err(|e| IdtError::ParseError(format!("Invalid Snowflake ID: {}", e)))?;

        Ok(Self {
            id,
            layout,
            input: input_trimmed.to_string(),
        })
    }

    pub fn timestamp_raw(&self) -> u64 {
        self.layout
            .extract_field(self.id, "timestamp")
            .expect("Snowflake layout must have a timestamp field")
    }

    pub fn timestamp_ms(&self) -> u64 {
        let raw = self.timestamp_raw();
        let ms = match self.layout.timestamp_unit {
            TimestampUnit::Millis => raw,
            TimestampUnit::TenMillis => raw * 10,
            TimestampUnit::Seconds => raw * 1000,
        };
        ms + self.layout.epoch
    }

    pub fn datacenter_id(&self) -> u16 {
        self.layout
            .extract_field(self.id, "datacenter_id")
            .unwrap_or(0) as u16
    }

    pub fn machine_id(&self) -> u16 {
        self.layout
            .extract_field(self.id, "machine_id")
            .unwrap_or(0) as u16
    }

    pub fn sequence(&self) -> u64 {
        self.layout.extract_field(self.id, "sequence").unwrap_or(0)
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
        let timestamp = self.timestamp().expect("Snowflake always has a timestamp");

        // Build components dynamically from layout fields
        let mut components = serde_json::Map::new();
        components.insert("timestamp_ms".to_string(), json!(self.timestamp_ms()));
        components.insert("epoch".to_string(), json!(self.layout.epoch));

        for field in self.layout.fields {
            if field.name == "timestamp" {
                continue; // already added as timestamp_ms
            }
            if let Some(val) = self.layout.extract_field(self.id, field.name) {
                components.insert(field.name.to_string(), json!(val));
            }
        }

        let variant_name = match self.layout.name {
            "twitter" => "Twitter",
            "discord" => "Discord",
            "instagram" => "Instagram",
            "sonyflake" => "Sonyflake",
            "mastodon" => "Mastodon",
            _ => "Custom",
        };

        InspectionResult {
            id_type: "snowflake".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: Some(timestamp),
            timestamp_iso: Some(timestamp.to_iso8601()),
            timestamp_local_iso: None,
            version: None,
            variant: Some(variant_name.to_string()),
            random_bits: None,
            components: Some(json!(components)),
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
        let ts = self.timestamp_ms();
        let now = chrono::Utc::now().timestamp_millis() as u64;

        if ts > now + 86400000 {
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
        let id = "1234567890123456789";
        let parsed = ParsedSnowflake::parse(id).unwrap();

        assert!(parsed.timestamp_ms() > 0);
        assert!(parsed.datacenter_id() < 32);
        assert!(parsed.machine_id() < 32);
        assert!(parsed.sequence() < 4096);
    }

    #[test]
    fn test_layout_field_bits_sum() {
        // All layouts must have fields summing to at most 64 bits
        let layouts = [
            SnowflakeLayout::twitter(),
            SnowflakeLayout::discord(),
            SnowflakeLayout::instagram(),
            SnowflakeLayout::sonyflake(),
            SnowflakeLayout::mastodon(),
        ];
        for layout in &layouts {
            let total: u8 = layout.fields.iter().map(|f| f.bits).sum();
            assert!(
                total <= 64,
                "Layout '{}' bits sum to {} (exceeds 64)",
                layout.name,
                total
            );
            assert!(
                total >= 63,
                "Layout '{}' bits sum to {} (less than 63)",
                layout.name,
                total
            );
        }
    }

    #[test]
    fn test_preset_by_name() {
        assert!(SnowflakeLayout::by_name("twitter").is_some());
        assert!(SnowflakeLayout::by_name("discord").is_some());
        assert!(SnowflakeLayout::by_name("instagram").is_some());
        assert!(SnowflakeLayout::by_name("sonyflake").is_some());
        assert!(SnowflakeLayout::by_name("mastodon").is_some());
        assert!(SnowflakeLayout::by_name("unknown").is_none());
    }

    #[test]
    fn test_resolve_preset_and_epoch_conflict() {
        let result = SnowflakeLayout::resolve(Some("twitter"), Some("0"));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_backward_compat_epoch() {
        let layout = SnowflakeLayout::resolve(None, Some("twitter")).unwrap();
        assert_eq!(layout.epoch, TWITTER_EPOCH);

        let layout = SnowflakeLayout::resolve(None, Some("1288834974657")).unwrap();
        assert_eq!(layout.epoch, 1288834974657);
    }

    #[test]
    fn test_instagram_layout() {
        let generator = SnowflakeGenerator::new()
            .with_layout(SnowflakeLayout::instagram())
            .with_field("shard_id", 42);
        let id = generator.generate().unwrap();
        let parsed = ParsedSnowflake::parse_with_layout(&id, SnowflakeLayout::instagram()).unwrap();
        assert_eq!(parsed.layout.extract_field(parsed.id, "shard_id"), Some(42));
    }

    #[test]
    fn test_sonyflake_layout() {
        let generator = SnowflakeGenerator::new()
            .with_layout(SnowflakeLayout::sonyflake())
            .with_machine_id(100);
        let id = generator.generate().unwrap();
        let parsed = ParsedSnowflake::parse_with_layout(&id, SnowflakeLayout::sonyflake()).unwrap();
        assert_eq!(parsed.machine_id(), 100);
    }

    #[test]
    fn test_mastodon_layout() {
        let generator = SnowflakeGenerator::new().with_layout(SnowflakeLayout::mastodon());
        let id = generator.generate().unwrap();
        let parsed = ParsedSnowflake::parse_with_layout(&id, SnowflakeLayout::mastodon()).unwrap();
        // Mastodon uses Unix epoch so timestamp_ms should be close to current time
        let now = chrono::Utc::now().timestamp_millis() as u64;
        assert!(parsed.timestamp_ms() <= now + 1000);
        assert!(parsed.timestamp_ms() > now - 5000);
    }

    #[test]
    fn test_dynamic_inspect_components() {
        let generator = SnowflakeGenerator::new()
            .with_layout(SnowflakeLayout::instagram())
            .with_field("shard_id", 7);
        let id = generator.generate().unwrap();
        let parsed = ParsedSnowflake::parse_with_layout(&id, SnowflakeLayout::instagram()).unwrap();
        let inspection = parsed.inspect();
        assert_eq!(inspection.variant, Some("Instagram".to_string()));
        let comps = inspection.components.unwrap();
        assert_eq!(comps["shard_id"], 7);
    }
}

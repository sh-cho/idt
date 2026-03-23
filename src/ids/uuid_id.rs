use crate::core::encoding::{
    EncodingFormat, encode_base32, encode_base58, encode_base64, encode_base64_url, encode_bits,
    encode_bytes_spaced, encode_hex, encode_hex_upper,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult,
};
use serde_json::json;
use uuid::Uuid;

/// UUID generator with configurable version
pub struct UuidGenerator {
    pub version: u8,
    pub namespace: Option<Uuid>,
    pub name: Option<String>,
}

impl Default for UuidGenerator {
    fn default() -> Self {
        Self {
            version: 4,
            namespace: None,
            name: None,
        }
    }
}

impl UuidGenerator {
    pub fn new(version: u8) -> Self {
        Self {
            version,
            namespace: None,
            name: None,
        }
    }

    pub fn v1() -> Self {
        Self::new(1)
    }

    pub fn v4() -> Self {
        Self::new(4)
    }

    pub fn v6() -> Self {
        Self::new(6)
    }

    pub fn v7() -> Self {
        Self::new(7)
    }

    pub fn nil() -> Self {
        Self::new(0)
    }

    pub fn max() -> Self {
        Self::new(255)
    }

    pub fn with_namespace(mut self, namespace: Uuid) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

impl IdGenerator for UuidGenerator {
    fn generate(&self) -> Result<String> {
        let uuid = match self.version {
            0 => Uuid::nil(),
            255 => Uuid::max(),
            1 => Uuid::now_v1(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
            3 => {
                let ns = self.namespace.unwrap_or(Uuid::NAMESPACE_DNS);
                let name = self.name.as_deref().unwrap_or("example.com");
                Uuid::new_v3(&ns, name.as_bytes())
            }
            4 => Uuid::new_v4(),
            5 => {
                let ns = self.namespace.unwrap_or(Uuid::NAMESPACE_DNS);
                let name = self.name.as_deref().unwrap_or("example.com");
                Uuid::new_v5(&ns, name.as_bytes())
            }
            6 => Uuid::now_v6(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
            7 => Uuid::now_v7(),
            _ => {
                return Err(IdtError::InvalidArgument(format!(
                    "Unsupported UUID version: {}",
                    self.version
                )));
            }
        };
        Ok(uuid.to_string())
    }
}

/// Parsed UUID value
pub struct ParsedUuid {
    uuid: Uuid,
    input: String,
}

impl ParsedUuid {
    pub fn parse(input: &str) -> Result<Self> {
        let input_trimmed = input.trim();

        // Try parsing with dashes
        if let Ok(uuid) = Uuid::parse_str(input_trimmed) {
            return Ok(Self {
                uuid,
                input: input_trimmed.to_string(),
            });
        }

        // Try parsing without dashes
        let normalized = input_trimmed.replace('-', "");
        if normalized.len() == 32
            && let Ok(uuid) = Uuid::parse_str(&normalized)
        {
            return Ok(Self {
                uuid,
                input: input_trimmed.to_string(),
            });
        }

        Err(IdtError::ParseError(format!("Invalid UUID: {}", input)))
    }

    fn get_version(&self) -> Option<u8> {
        if self.uuid.is_nil() {
            return Some(0);
        }
        if self.uuid.is_max() {
            return Some(255);
        }
        Some(self.uuid.get_version_num() as u8)
    }

    fn get_variant(&self) -> &'static str {
        match self.uuid.get_variant() {
            uuid::Variant::NCS => "NCS",
            uuid::Variant::RFC4122 => "RFC4122",
            uuid::Variant::Microsoft => "Microsoft",
            uuid::Variant::Future => "Future",
            _ => "Unknown",
        }
    }

    fn version_to_kind(version: u8) -> IdKind {
        match version {
            0 => IdKind::UuidNil,
            1 => IdKind::UuidV1,
            3 => IdKind::UuidV3,
            4 => IdKind::UuidV4,
            5 => IdKind::UuidV5,
            6 => IdKind::UuidV6,
            7 => IdKind::UuidV7,
            255 => IdKind::UuidMax,
            _ => IdKind::Uuid,
        }
    }
}

impl ParsedId for ParsedUuid {
    fn kind(&self) -> IdKind {
        self.get_version()
            .map(Self::version_to_kind)
            .unwrap_or(IdKind::Uuid)
    }

    fn canonical(&self) -> String {
        self.uuid.to_string()
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.uuid.as_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        let version = self.get_version()?;
        match version {
            1 | 6 => {
                // UUID v1 and v6 use 100-nanosecond intervals since Oct 15, 1582
                let ts = self.uuid.get_timestamp()?;
                let (secs, nanos) = ts.to_unix();
                let millis = secs * 1000 + nanos as u64 / 1_000_000;
                Some(Timestamp::new(millis))
            }
            7 => {
                // UUID v7 uses milliseconds since Unix epoch
                let ts = self.uuid.get_timestamp()?;
                let (secs, nanos) = ts.to_unix();
                let millis = secs * 1000 + nanos as u64 / 1_000_000;
                Some(Timestamp::new(millis))
            }
            _ => None,
        }
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let version = self.get_version();
        let timestamp = self.timestamp();

        let mut components = json!({
            "version": version,
            "variant": self.get_variant(),
        });

        if let Some(ts) = &timestamp {
            components["timestamp_ms"] = json!(ts.millis);
        }

        // Add random bits info based on version
        let random_bits = match version {
            Some(4) => Some(122),          // 128 - 4 (version) - 2 (variant)
            Some(7) => Some(62),           // Random portion of v7
            Some(1) | Some(6) => Some(14), // Clock sequence
            _ => None,
        };

        InspectionResult {
            id_type: self.kind().to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp,
            timestamp_iso: timestamp.as_ref().map(|ts| ts.to_iso8601()),
            timestamp_local_iso: None,
            version: version.map(|v| format!("{}", v)),
            variant: Some(self.get_variant().to_string()),
            random_bits,
            components: Some(components),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: encode_base32(&bytes),
                base58: encode_base58(&bytes),
                base64: encode_base64(&bytes),
                int: Some(
                    u128::from_be_bytes(bytes.try_into().expect("UUID is 128-bit (16 bytes)"))
                        .to_string(),
                ),
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        ValidationResult::valid(self.kind().name())
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
                u128::from_be_bytes(bytes.try_into().expect("UUID is 128-bit (16 bytes)"))
                    .to_string()
            }
            EncodingFormat::Bytes => encode_bytes_spaced(&bytes),
        }
    }
}

/// Check if a string can be parsed as UUID
pub fn is_uuid(input: &str) -> bool {
    ParsedUuid::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_v4() {
        let generator = UuidGenerator::v4();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 36);
        assert!(is_uuid(&id));
    }

    #[test]
    fn test_generate_v7() {
        let generator = UuidGenerator::v7();
        let id = generator.generate().unwrap();
        let parsed = ParsedUuid::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::UuidV7);
        assert!(parsed.timestamp().is_some());
    }

    #[test]
    fn test_parse_uuid() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        assert_eq!(parsed.kind(), IdKind::UuidV4);
    }

    #[test]
    fn test_nil_uuid() {
        let generator = UuidGenerator::nil();
        let id = generator.generate().unwrap();
        assert_eq!(id, "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_max_uuid() {
        let generator = UuidGenerator::max();
        let id = generator.generate().unwrap();
        assert_eq!(id, "ffffffff-ffff-ffff-ffff-ffffffffffff");
    }

    #[test]
    fn test_generate_v1() {
        let generator = UuidGenerator::v1();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 36);
        let parsed = ParsedUuid::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::UuidV1);
    }

    #[test]
    fn test_generate_v6() {
        let generator = UuidGenerator::v6();
        let id = generator.generate().unwrap();
        assert_eq!(id.len(), 36);
        let parsed = ParsedUuid::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::UuidV6);
    }

    #[test]
    fn test_is_uuid_invalid() {
        assert!(!is_uuid("not-a-uuid"));
        assert!(!is_uuid(""));
        assert!(!is_uuid("12345"));
    }

    #[test]
    fn test_uuid_canonical() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        assert_eq!(parsed.canonical(), input);
    }

    #[test]
    fn test_uuid_as_bytes() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let bytes = parsed.as_bytes();
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_uuid_inspect() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let inspection = parsed.inspect();
        assert!(inspection.valid);
        assert_eq!(inspection.canonical, input);
        assert!(inspection.version.is_some());
    }

    #[test]
    fn test_uuid_validate() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let result = parsed.validate();
        assert!(result.valid);
    }

    #[test]
    fn test_uuid_encode_hex() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let hex = parsed.encode(EncodingFormat::Hex);
        assert_eq!(hex, "550e8400e29b41d4a716446655440000");
    }

    #[test]
    fn test_uuid_encode_base64() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let b64 = parsed.encode(EncodingFormat::Base64);
        assert!(!b64.is_empty());
    }

    #[test]
    fn test_uuid_encode_base58() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let b58 = parsed.encode(EncodingFormat::Base58);
        assert!(!b58.is_empty());
    }

    #[test]
    fn test_uuid_encode_bits() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let bits = parsed.encode(EncodingFormat::Bits);
        assert_eq!(bits.len(), 128);
    }

    #[test]
    fn test_uuid_encode_int() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = ParsedUuid::parse(input).unwrap();
        let int_str = parsed.encode(EncodingFormat::Int);
        assert!(!int_str.is_empty());
        assert!(int_str.parse::<u128>().is_ok());
    }

    #[test]
    fn test_uuid_generate_many() {
        let generator = UuidGenerator::v4();
        let ids = generator.generate_many(5).unwrap();
        assert_eq!(ids.len(), 5);
        for id in &ids {
            assert!(is_uuid(id));
        }
    }

    #[test]
    fn test_uuid_v7_timestamp() {
        let generator = UuidGenerator::v7();
        let id = generator.generate().unwrap();
        let parsed = ParsedUuid::parse(&id).unwrap();
        let ts = parsed.timestamp().unwrap();
        assert!(ts.millis > 1_000_000_000_000);
    }
}

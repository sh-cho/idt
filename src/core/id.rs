use crate::core::encoding::EncodingFormat;
use crate::core::error::Result;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Timestamp wrapper for ID timestamps
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Timestamp {
    pub millis: u64,
}

impl Timestamp {
    pub fn new(millis: u64) -> Self {
        Self { millis }
    }

    pub fn from_secs(secs: u64) -> Self {
        Self {
            millis: secs * 1000,
        }
    }

    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_millis(self.millis as i64)
    }

    pub fn to_iso8601(&self) -> String {
        self.to_datetime()
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
            .unwrap_or_else(|| "invalid".to_string())
    }

    pub fn to_local_iso8601(&self) -> String {
        self.to_datetime()
            .map(|dt| {
                let local: DateTime<Local> = dt.with_timezone(&Local);
                local.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
            })
            .unwrap_or_else(|| "invalid".to_string())
    }

    pub fn local_timezone_abbr(&self) -> String {
        self.to_datetime()
            .map(|dt| {
                let local: DateTime<Local> = dt.with_timezone(&Local);
                local.format("%Z").to_string()
            })
            .unwrap_or_else(|| "Local".to_string())
    }
}

/// Result of inspecting an ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    pub id_type: String,
    pub input: String,
    pub canonical: String,
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_iso: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_local_iso: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub random_bits: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<serde_json::Value>,
    pub encodings: IdEncodings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdEncodings {
    pub hex: String,
    pub base32: String,
    pub base58: String,
    pub base64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int: Option<String>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub id_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl ValidationResult {
    pub fn valid(id_type: &str) -> Self {
        Self {
            valid: true,
            id_type: Some(id_type.to_string()),
            error: None,
            hint: None,
        }
    }

    pub fn invalid(error: &str) -> Self {
        Self {
            valid: false,
            id_type: None,
            error: Some(error.to_string()),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_string());
        self
    }
}

/// Supported ID types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum IdKind {
    #[value(name = "uuid")]
    Uuid,
    #[value(name = "uuidv1", alias = "uuid-v1", alias = "uuid1")]
    UuidV1,
    #[value(name = "uuidv3", alias = "uuid-v3", alias = "uuid3")]
    UuidV3,
    #[value(name = "uuidv4", alias = "uuid-v4", alias = "uuid4")]
    UuidV4,
    #[value(name = "uuidv5", alias = "uuid-v5", alias = "uuid5")]
    UuidV5,
    #[value(name = "uuidv6", alias = "uuid-v6", alias = "uuid6")]
    UuidV6,
    #[value(name = "uuidv7", alias = "uuid-v7", alias = "uuid7")]
    UuidV7,
    #[value(name = "uuid-nil", alias = "uuidnil", alias = "nil")]
    UuidNil,
    #[value(name = "uuid-max", alias = "uuidmax", alias = "max")]
    UuidMax,
    #[value(name = "ulid")]
    Ulid,
    #[value(name = "nanoid", alias = "nano")]
    NanoId,
    #[value(name = "ksuid")]
    Ksuid,
    #[value(name = "snowflake", alias = "snow")]
    Snowflake,
    #[value(name = "objectid", alias = "oid", alias = "mongoid")]
    ObjectId,
    #[value(name = "typeid")]
    TypeId,
    #[value(name = "xid")]
    Xid,
    #[value(name = "cuid")]
    Cuid,
    #[value(name = "cuid2")]
    Cuid2,
    #[value(name = "tsid")]
    Tsid,
}

impl IdKind {
    pub fn name(&self) -> &'static str {
        match self {
            IdKind::Uuid => "uuid",
            IdKind::UuidV1 => "uuidv1",
            IdKind::UuidV3 => "uuidv3",
            IdKind::UuidV4 => "uuidv4",
            IdKind::UuidV5 => "uuidv5",
            IdKind::UuidV6 => "uuidv6",
            IdKind::UuidV7 => "uuidv7",
            IdKind::UuidNil => "uuid-nil",
            IdKind::UuidMax => "uuid-max",
            IdKind::Ulid => "ulid",
            IdKind::NanoId => "nanoid",
            IdKind::Ksuid => "ksuid",
            IdKind::Snowflake => "snowflake",
            IdKind::ObjectId => "objectid",
            IdKind::TypeId => "typeid",
            IdKind::Xid => "xid",
            IdKind::Cuid => "cuid",
            IdKind::Cuid2 => "cuid2",
            IdKind::Tsid => "tsid",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            IdKind::Uuid => "UUID (any version)",
            IdKind::UuidV1 => "UUID v1 (timestamp + MAC address)",
            IdKind::UuidV3 => "UUID v3 (MD5 namespace hash)",
            IdKind::UuidV4 => "UUID v4 (random)",
            IdKind::UuidV5 => "UUID v5 (SHA-1 namespace hash)",
            IdKind::UuidV6 => "UUID v6 (reordered timestamp)",
            IdKind::UuidV7 => "UUID v7 (Unix timestamp + random)",
            IdKind::UuidNil => "Nil UUID (all zeros)",
            IdKind::UuidMax => "Max UUID (all ones)",
            IdKind::Ulid => "ULID (Universally Unique Lexicographically Sortable Identifier)",
            IdKind::NanoId => "NanoID (compact URL-friendly unique ID)",
            IdKind::Ksuid => "KSUID (K-Sortable Unique Identifier)",
            IdKind::Snowflake => "Snowflake ID (Twitter-style distributed ID)",
            IdKind::ObjectId => "MongoDB ObjectId",
            IdKind::TypeId => "TypeID (type-prefixed, sortable ID)",
            IdKind::Xid => "Xid (globally unique, sortable ID)",
            IdKind::Cuid => "CUID (collision-resistant unique identifier)",
            IdKind::Cuid2 => "CUID2 (secure collision-resistant ID)",
            IdKind::Tsid => "TSID (time-sorted unique identifier)",
        }
    }

    pub fn has_timestamp(&self) -> bool {
        matches!(
            self,
            IdKind::UuidV1
                | IdKind::UuidV6
                | IdKind::UuidV7
                | IdKind::Ulid
                | IdKind::Ksuid
                | IdKind::Snowflake
                | IdKind::ObjectId
                | IdKind::TypeId
                | IdKind::Xid
                | IdKind::Cuid
                | IdKind::Tsid
        )
    }

    pub fn is_sortable(&self) -> bool {
        matches!(
            self,
            IdKind::UuidV6
                | IdKind::UuidV7
                | IdKind::Ulid
                | IdKind::Ksuid
                | IdKind::Snowflake
                | IdKind::TypeId
                | IdKind::Xid
                | IdKind::Tsid
        )
    }

    pub fn bit_length(&self) -> usize {
        match self {
            IdKind::Uuid
            | IdKind::UuidV1
            | IdKind::UuidV3
            | IdKind::UuidV4
            | IdKind::UuidV5
            | IdKind::UuidV6
            | IdKind::UuidV7
            | IdKind::UuidNil
            | IdKind::UuidMax => 128,
            IdKind::Ulid => 128,
            IdKind::NanoId => 126, // 21 chars * 6 bits (approximate)
            IdKind::Ksuid => 160,
            IdKind::Snowflake => 64,
            IdKind::ObjectId => 96,
            IdKind::TypeId => 128, // UUID portion
            IdKind::Xid => 96,
            IdKind::Cuid => 128,
            IdKind::Cuid2 => 128,
            IdKind::Tsid => 64,
        }
    }

    pub fn all() -> &'static [IdKind] {
        &[
            IdKind::Uuid,
            IdKind::UuidV1,
            IdKind::UuidV3,
            IdKind::UuidV4,
            IdKind::UuidV5,
            IdKind::UuidV6,
            IdKind::UuidV7,
            IdKind::UuidNil,
            IdKind::UuidMax,
            IdKind::Ulid,
            IdKind::NanoId,
            IdKind::Ksuid,
            IdKind::Snowflake,
            IdKind::ObjectId,
            IdKind::TypeId,
            IdKind::Xid,
            IdKind::Cuid,
            IdKind::Cuid2,
            IdKind::Tsid,
        ]
    }

    pub fn generatable() -> &'static [IdKind] {
        &[
            IdKind::Uuid,
            IdKind::UuidV1,
            IdKind::UuidV4,
            IdKind::UuidV6,
            IdKind::UuidV7,
            IdKind::UuidNil,
            IdKind::UuidMax,
            IdKind::Ulid,
            IdKind::NanoId,
            IdKind::Snowflake,
            IdKind::ObjectId,
            IdKind::Ksuid,
            IdKind::Xid,
            IdKind::Tsid,
            IdKind::Cuid,
            IdKind::Cuid2,
            IdKind::TypeId,
        ]
    }
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for IdKind {
    type Err = crate::core::error::IdtError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "uuid" => Ok(IdKind::Uuid),
            "uuidv1" | "uuid-v1" | "uuid1" => Ok(IdKind::UuidV1),
            "uuidv3" | "uuid-v3" | "uuid3" => Ok(IdKind::UuidV3),
            "uuidv4" | "uuid-v4" | "uuid4" => Ok(IdKind::UuidV4),
            "uuidv5" | "uuid-v5" | "uuid5" => Ok(IdKind::UuidV5),
            "uuidv6" | "uuid-v6" | "uuid6" => Ok(IdKind::UuidV6),
            "uuidv7" | "uuid-v7" | "uuid7" => Ok(IdKind::UuidV7),
            "uuid-nil" | "uuidnil" | "nil" => Ok(IdKind::UuidNil),
            "uuid-max" | "uuidmax" | "max" => Ok(IdKind::UuidMax),
            "ulid" => Ok(IdKind::Ulid),
            "nanoid" | "nano" => Ok(IdKind::NanoId),
            "ksuid" => Ok(IdKind::Ksuid),
            "snowflake" | "snow" => Ok(IdKind::Snowflake),
            "objectid" | "oid" | "mongoid" => Ok(IdKind::ObjectId),
            "typeid" => Ok(IdKind::TypeId),
            "xid" => Ok(IdKind::Xid),
            "cuid" => Ok(IdKind::Cuid),
            "cuid2" => Ok(IdKind::Cuid2),
            "tsid" => Ok(IdKind::Tsid),
            _ => Err(crate::core::error::IdtError::UnknownType(s.to_string())),
        }
    }
}

/// Trait for ID types that can be generated
pub trait IdGenerator {
    fn generate(&self) -> Result<String>;
    fn generate_many(&self, count: usize) -> Result<Vec<String>> {
        (0..count).map(|_| self.generate()).collect()
    }
}

/// Trait for ID types that can be parsed and inspected
pub trait IdParser {
    fn parse(&self, input: &str) -> Result<Box<dyn ParsedId>>;
    fn can_parse(&self, input: &str) -> bool;
}

/// Trait for parsed ID values
pub trait ParsedId: Send + Sync {
    fn kind(&self) -> IdKind;
    fn canonical(&self) -> String;
    fn as_bytes(&self) -> Vec<u8>;
    fn timestamp(&self) -> Option<Timestamp>;
    fn inspect(&self) -> InspectionResult;
    fn validate(&self) -> ValidationResult;
    fn encode(&self, format: EncodingFormat) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_new() {
        let ts = Timestamp::new(1000);
        assert_eq!(ts.millis, 1000);
    }

    #[test]
    fn test_timestamp_from_secs() {
        let ts = Timestamp::from_secs(5);
        assert_eq!(ts.millis, 5000);
    }

    #[test]
    fn test_timestamp_to_datetime() {
        let ts = Timestamp::new(1_700_000_000_000); // 2023-11-14
        let dt = ts.to_datetime().unwrap();
        assert_eq!(dt.timestamp_millis(), 1_700_000_000_000);
    }

    #[test]
    fn test_timestamp_to_iso8601() {
        let ts = Timestamp::new(0);
        let iso = ts.to_iso8601();
        assert!(iso.contains("1970-01-01"));
    }

    #[test]
    fn test_timestamp_to_local_iso8601() {
        let ts = Timestamp::new(1_700_000_000_000);
        let local = ts.to_local_iso8601();
        assert!(local.contains("2023"));
    }

    #[test]
    fn test_timestamp_local_timezone_abbr() {
        let ts = Timestamp::new(1_700_000_000_000);
        let abbr = ts.local_timezone_abbr();
        assert!(!abbr.is_empty());
    }

    #[test]
    fn test_validation_result_valid() {
        let r = ValidationResult::valid("uuid");
        assert!(r.valid);
        assert_eq!(r.id_type, Some("uuid".to_string()));
        assert!(r.error.is_none());
        assert!(r.hint.is_none());
    }

    #[test]
    fn test_validation_result_invalid() {
        let r = ValidationResult::invalid("bad format");
        assert!(!r.valid);
        assert!(r.id_type.is_none());
        assert_eq!(r.error, Some("bad format".to_string()));
    }

    #[test]
    fn test_validation_result_with_hint() {
        let r = ValidationResult::invalid("error").with_hint("try this");
        assert_eq!(r.hint, Some("try this".to_string()));
    }

    #[test]
    fn test_id_kind_name() {
        assert_eq!(IdKind::Uuid.name(), "uuid");
        assert_eq!(IdKind::UuidV7.name(), "uuidv7");
        assert_eq!(IdKind::Ulid.name(), "ulid");
        assert_eq!(IdKind::Snowflake.name(), "snowflake");
        assert_eq!(IdKind::ObjectId.name(), "objectid");
        assert_eq!(IdKind::TypeId.name(), "typeid");
        assert_eq!(IdKind::Xid.name(), "xid");
        assert_eq!(IdKind::Cuid.name(), "cuid");
        assert_eq!(IdKind::Cuid2.name(), "cuid2");
        assert_eq!(IdKind::Tsid.name(), "tsid");
        assert_eq!(IdKind::NanoId.name(), "nanoid");
        assert_eq!(IdKind::Ksuid.name(), "ksuid");
    }

    #[test]
    fn test_id_kind_description() {
        let desc = IdKind::UuidV4.description();
        assert!(desc.contains("random"));
    }

    #[test]
    fn test_id_kind_has_timestamp() {
        assert!(IdKind::UuidV7.has_timestamp());
        assert!(IdKind::Ulid.has_timestamp());
        assert!(IdKind::Snowflake.has_timestamp());
        assert!(!IdKind::UuidV4.has_timestamp());
        assert!(!IdKind::NanoId.has_timestamp());
        assert!(!IdKind::Cuid2.has_timestamp());
    }

    #[test]
    fn test_id_kind_is_sortable() {
        assert!(IdKind::UuidV7.is_sortable());
        assert!(IdKind::Ulid.is_sortable());
        assert!(!IdKind::UuidV4.is_sortable());
        assert!(!IdKind::NanoId.is_sortable());
    }

    #[test]
    fn test_id_kind_bit_length() {
        assert_eq!(IdKind::UuidV4.bit_length(), 128);
        assert_eq!(IdKind::Snowflake.bit_length(), 64);
        assert_eq!(IdKind::ObjectId.bit_length(), 96);
        assert_eq!(IdKind::Ksuid.bit_length(), 160);
        assert_eq!(IdKind::Tsid.bit_length(), 64);
    }

    #[test]
    fn test_id_kind_all() {
        let all = IdKind::all();
        assert!(all.len() >= 19);
        assert!(all.contains(&IdKind::Uuid));
        assert!(all.contains(&IdKind::Tsid));
    }

    #[test]
    fn test_id_kind_generatable() {
        let generatable = IdKind::generatable();
        assert!(generatable.contains(&IdKind::UuidV4));
        assert!(generatable.contains(&IdKind::Ulid));
        // v3 and v5 need namespace/name so not in generatable
        assert!(!generatable.contains(&IdKind::UuidV3));
        assert!(!generatable.contains(&IdKind::UuidV5));
    }

    #[test]
    fn test_id_kind_display() {
        assert_eq!(format!("{}", IdKind::UuidV4), "uuidv4");
        assert_eq!(format!("{}", IdKind::Ulid), "ulid");
    }

    #[test]
    fn test_id_kind_from_str() {
        assert_eq!("uuid".parse::<IdKind>().unwrap(), IdKind::Uuid);
        assert_eq!("uuidv1".parse::<IdKind>().unwrap(), IdKind::UuidV1);
        assert_eq!("uuid-v1".parse::<IdKind>().unwrap(), IdKind::UuidV1);
        assert_eq!("uuid1".parse::<IdKind>().unwrap(), IdKind::UuidV1);
        assert_eq!("ulid".parse::<IdKind>().unwrap(), IdKind::Ulid);
        assert_eq!("snow".parse::<IdKind>().unwrap(), IdKind::Snowflake);
        assert_eq!("oid".parse::<IdKind>().unwrap(), IdKind::ObjectId);
        assert_eq!("mongoid".parse::<IdKind>().unwrap(), IdKind::ObjectId);
        assert_eq!("nano".parse::<IdKind>().unwrap(), IdKind::NanoId);
        assert_eq!("nil".parse::<IdKind>().unwrap(), IdKind::UuidNil);
        assert_eq!("max".parse::<IdKind>().unwrap(), IdKind::UuidMax);
        assert!("unknown_type".parse::<IdKind>().is_err());
    }
}

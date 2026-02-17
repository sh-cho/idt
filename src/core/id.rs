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
        Self { millis: secs * 1000 }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdKind {
    Uuid,
    UuidV1,
    UuidV3,
    UuidV4,
    UuidV5,
    UuidV6,
    UuidV7,
    UuidNil,
    UuidMax,
    Ulid,
    NanoId,
    Ksuid,
    Snowflake,
    ObjectId,
    TypeId,
    Xid,
    Cuid,
    Cuid2,
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

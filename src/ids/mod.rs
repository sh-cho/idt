pub mod cuid2_id;
pub mod cuid_id;
pub mod ksuid_id;
pub mod nanoid_id;
pub mod objectid_id;
pub mod snowflake_id;
pub mod tsid_id;
pub mod typeid_id;
pub mod ulid_id;
pub mod uuid_id;
pub mod xid_id;

pub use cuid_id::{CuidGenerator, ParsedCuid, is_cuid};
pub use cuid2_id::{Cuid2Generator, ParsedCuid2, is_cuid2};
pub use ksuid_id::{KsuidGenerator, ParsedKsuid, is_ksuid};
pub use nanoid_id::{NanoIdGenerator, ParsedNanoId, is_nanoid};
pub use objectid_id::{ObjectIdGenerator, ParsedObjectId, is_objectid};
pub use snowflake_id::{
    DISCORD_EPOCH, ParsedSnowflake, SnowflakeGenerator, TWITTER_EPOCH, is_snowflake,
};
pub use tsid_id::{ParsedTsid, TsidGenerator, is_tsid};
pub use typeid_id::{ParsedTypeId, TypeIdGenerator, is_typeid};
pub use ulid_id::{ParsedUlid, UlidGenerator, is_ulid};
pub use uuid_id::{ParsedUuid, UuidGenerator, is_uuid};
pub use xid_id::{ParsedXid, XidGenerator, is_xid};

use crate::core::error::{IdtError, Result};
use crate::core::id::{IdGenerator, IdKind, ParsedId};

/// Create a generator for the given ID kind
pub fn create_generator(kind: IdKind) -> Result<Box<dyn IdGenerator>> {
    match kind {
        IdKind::Uuid | IdKind::UuidV4 => Ok(Box::new(UuidGenerator::v4())),
        IdKind::UuidV1 => Ok(Box::new(UuidGenerator::v1())),
        IdKind::UuidV6 => Ok(Box::new(UuidGenerator::v6())),
        IdKind::UuidV7 => Ok(Box::new(UuidGenerator::v7())),
        IdKind::UuidNil => Ok(Box::new(UuidGenerator::nil())),
        IdKind::UuidMax => Ok(Box::new(UuidGenerator::max())),
        IdKind::Ulid => Ok(Box::new(UlidGenerator::new())),
        IdKind::NanoId => Ok(Box::new(NanoIdGenerator::new())),
        IdKind::Snowflake => Ok(Box::new(SnowflakeGenerator::new())),
        IdKind::ObjectId => Ok(Box::new(ObjectIdGenerator::new())),
        IdKind::Ksuid => Ok(Box::new(KsuidGenerator::new())),
        IdKind::Xid => Ok(Box::new(XidGenerator::new())),
        IdKind::Tsid => Ok(Box::new(TsidGenerator::new())),
        IdKind::Cuid => Ok(Box::new(CuidGenerator::new())),
        IdKind::Cuid2 => Ok(Box::new(Cuid2Generator::new())),
        IdKind::TypeId => Ok(Box::new(TypeIdGenerator::new(""))),
        _ => Err(IdtError::GenerationError(format!(
            "Generation not supported for: {}",
            kind.name()
        ))),
    }
}

/// Parse an ID string into a ParsedId, optionally with a type hint
pub fn parse_id(input: &str, type_hint: Option<IdKind>) -> Result<Box<dyn ParsedId>> {
    let input = input.trim();

    if let Some(kind) = type_hint {
        return parse_as_type(input, kind);
    }

    // Auto-detect
    let detections = crate::core::detect_id_type(input)?;

    for detection in detections {
        if let Ok(parsed) = parse_as_type(input, detection.kind) {
            return Ok(parsed);
        }
    }

    Err(IdtError::DetectionFailed)
}

/// Parse input as a specific ID type
fn parse_as_type(input: &str, kind: IdKind) -> Result<Box<dyn ParsedId>> {
    match kind {
        IdKind::Uuid
        | IdKind::UuidV1
        | IdKind::UuidV3
        | IdKind::UuidV4
        | IdKind::UuidV5
        | IdKind::UuidV6
        | IdKind::UuidV7
        | IdKind::UuidNil
        | IdKind::UuidMax => Ok(Box::new(ParsedUuid::parse(input)?)),
        IdKind::Ulid => Ok(Box::new(ParsedUlid::parse(input)?)),
        IdKind::NanoId => Ok(Box::new(ParsedNanoId::parse(input)?)),
        IdKind::Snowflake => Ok(Box::new(ParsedSnowflake::parse(input)?)),
        IdKind::ObjectId => Ok(Box::new(ParsedObjectId::parse(input)?)),
        IdKind::Ksuid => Ok(Box::new(ParsedKsuid::parse(input)?)),
        IdKind::Xid => Ok(Box::new(ParsedXid::parse(input)?)),
        IdKind::Tsid => Ok(Box::new(ParsedTsid::parse(input)?)),
        IdKind::Cuid => Ok(Box::new(ParsedCuid::parse(input)?)),
        IdKind::Cuid2 => Ok(Box::new(ParsedCuid2::parse(input)?)),
        IdKind::TypeId => Ok(Box::new(ParsedTypeId::parse(input)?)),
    }
}

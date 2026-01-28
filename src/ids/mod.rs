pub mod nanoid_id;
pub mod snowflake_id;
pub mod ulid_id;
pub mod uuid_id;

pub use nanoid_id::{is_nanoid, NanoIdGenerator, ParsedNanoId};
pub use snowflake_id::{is_snowflake, ParsedSnowflake, SnowflakeGenerator, DISCORD_EPOCH, TWITTER_EPOCH};
pub use ulid_id::{is_ulid, ParsedUlid, UlidGenerator};
pub use uuid_id::{is_uuid, ParsedUuid, UuidGenerator};

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
        _ => Err(IdtError::ParseError(format!(
            "Parsing not implemented for: {}",
            kind.name()
        ))),
    }
}

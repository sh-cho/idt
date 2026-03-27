pub mod asin_id;
pub mod cuid2_id;
pub mod cuid_id;
pub mod ean13_id;
pub mod ean8_id;
pub mod gtin14_id;
pub mod isbn10_id;
pub mod isbn13_id;
pub mod isin_id;
pub mod ismn_id;
pub mod isni_id;
pub mod issn_id;
pub mod ksuid_id;
pub mod nanoid_id;
pub mod objectid_id;
pub mod snowflake_id;
pub mod tsid_id;
pub mod typeid_id;
pub mod ulid_id;
pub mod upca_id;
pub mod uuid_id;
pub mod xid_id;

pub use asin_id::{ParsedAsin, is_asin};
pub use cuid_id::{CuidGenerator, ParsedCuid, is_cuid};
pub use cuid2_id::{Cuid2Generator, ParsedCuid2, is_cuid2};
pub use ean8_id::{ParsedEan8, is_ean8};
pub use ean13_id::{ParsedEan13, is_ean13};
pub use gtin14_id::{ParsedGtin14, is_gtin14};
pub use isbn10_id::{ParsedIsbn10, is_isbn10};
pub use isbn13_id::{ParsedIsbn13, is_isbn13};
pub use isin_id::{ParsedIsin, is_isin};
pub use ismn_id::{ParsedIsmn, is_ismn};
pub use isni_id::{ParsedIsni, is_isni};
pub use issn_id::{ParsedIssn, is_issn};
pub use ksuid_id::{KsuidGenerator, ParsedKsuid, is_ksuid};
pub use nanoid_id::{NanoIdGenerator, ParsedNanoId, is_nanoid};
pub use objectid_id::{ObjectIdGenerator, ParsedObjectId, is_objectid};
pub use snowflake_id::{
    DISCORD_EPOCH, INSTAGRAM_EPOCH, ParsedSnowflake, SONYFLAKE_EPOCH, SnowflakeField,
    SnowflakeGenerator, SnowflakeLayout, TWITTER_EPOCH, TimestampUnit, is_snowflake,
};
pub use tsid_id::{ParsedTsid, TsidGenerator, is_tsid};
pub use typeid_id::{ParsedTypeId, TypeIdGenerator, is_typeid};
pub use ulid_id::{ParsedUlid, UlidGenerator, is_ulid};
pub use upca_id::{ParsedUpcA, is_upca};
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
        IdKind::Ean13 => Ok(Box::new(ParsedEan13::parse(input)?)),
        IdKind::Isbn13 => Ok(Box::new(ParsedIsbn13::parse(input)?)),
        IdKind::Isbn10 => Ok(Box::new(ParsedIsbn10::parse(input)?)),
        IdKind::Isin => Ok(Box::new(ParsedIsin::parse(input)?)),
        IdKind::Ean8 => Ok(Box::new(ParsedEan8::parse(input)?)),
        IdKind::UpcA => Ok(Box::new(ParsedUpcA::parse(input)?)),
        IdKind::Issn => Ok(Box::new(ParsedIssn::parse(input)?)),
        IdKind::Ismn => Ok(Box::new(ParsedIsmn::parse(input)?)),
        IdKind::Isni => Ok(Box::new(ParsedIsni::parse(input)?)),
        IdKind::Gtin14 => Ok(Box::new(ParsedGtin14::parse(input)?)),
        IdKind::Asin => Ok(Box::new(ParsedAsin::parse(input)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_generator_all_generatable() {
        for kind in IdKind::generatable() {
            let generator = create_generator(*kind);
            assert!(generator.is_ok(), "create_generator failed for {:?}", kind);
            let id = generator.unwrap().generate();
            assert!(id.is_ok(), "generate failed for {:?}", kind);
        }
    }

    #[test]
    fn test_create_generator_unsupported() {
        assert!(create_generator(IdKind::UuidV3).is_err());
        assert!(create_generator(IdKind::UuidV5).is_err());
    }

    #[test]
    fn test_parse_id_uuid() {
        let parsed = parse_id("550e8400-e29b-41d4-a716-446655440000", Some(IdKind::Uuid));
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_parse_id_ulid() {
        let parsed = parse_id("01ARZ3NDEKTSV4RRFFQ69G5FAV", Some(IdKind::Ulid));
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_parse_id_auto_detect_uuid() {
        let parsed = parse_id("550e8400-e29b-41d4-a716-446655440000", None);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_parse_id_invalid() {
        let parsed = parse_id("!!!", None);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_id_snowflake() {
        let parsed = parse_id("1234567890123456789", Some(IdKind::Snowflake));
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_parse_id_with_hint_each_type() {
        let uuid = create_generator(IdKind::UuidV4)
            .unwrap()
            .generate()
            .unwrap();
        assert!(parse_id(&uuid, Some(IdKind::Uuid)).is_ok());

        let ulid = create_generator(IdKind::Ulid).unwrap().generate().unwrap();
        assert!(parse_id(&ulid, Some(IdKind::Ulid)).is_ok());

        let snow = create_generator(IdKind::Snowflake)
            .unwrap()
            .generate()
            .unwrap();
        assert!(parse_id(&snow, Some(IdKind::Snowflake)).is_ok());

        let ksuid = create_generator(IdKind::Ksuid).unwrap().generate().unwrap();
        assert!(parse_id(&ksuid, Some(IdKind::Ksuid)).is_ok());

        let oid = create_generator(IdKind::ObjectId)
            .unwrap()
            .generate()
            .unwrap();
        assert!(parse_id(&oid, Some(IdKind::ObjectId)).is_ok());

        let xid = create_generator(IdKind::Xid).unwrap().generate().unwrap();
        assert!(parse_id(&xid, Some(IdKind::Xid)).is_ok());

        let tsid = create_generator(IdKind::Tsid).unwrap().generate().unwrap();
        assert!(parse_id(&tsid, Some(IdKind::Tsid)).is_ok());

        let typeid = create_generator(IdKind::TypeId)
            .unwrap()
            .generate()
            .unwrap();
        assert!(parse_id(&typeid, Some(IdKind::TypeId)).is_ok());
    }
}

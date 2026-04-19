use crate::core::encoding::{
    EncodingFormat, SHORTUUID_ALPHABET, decode_shortuuid, encode_base32, encode_base58,
    encode_base64, encode_base64_url, encode_bits, encode_bytes_spaced, encode_hex,
    encode_hex_upper, encode_shortuuid,
};
use crate::core::error::{IdtError, Result};
use crate::core::id::{
    IdEncodings, IdGenerator, IdKind, InspectionResult, ParsedId, SizeUnit, StructureSegment,
    Timestamp, ValidationResult,
};
use serde_json::json;
use uuid::Uuid;

/// ShortUUID generator — always v4 UUID encoded in base57.
#[derive(Default)]
pub struct ShortUuidGenerator;

impl ShortUuidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for ShortUuidGenerator {
    fn generate(&self) -> Result<String> {
        let uuid = Uuid::new_v4();
        Ok(encode_shortuuid(uuid.as_bytes()))
    }
}

/// Parsed ShortUUID value — stores the decoded UUID.
pub struct ParsedShortUuid {
    uuid: Uuid,
    input: String,
}

impl ParsedShortUuid {
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        if trimmed.len() != 22 {
            return Err(IdtError::ParseError(format!(
                "Invalid shortuuid length {} (expected 22)",
                trimmed.len()
            )));
        }

        let bytes = decode_shortuuid(trimmed)
            .map_err(|e| IdtError::ParseError(format!("Invalid shortuuid: {}", e)))?;

        let uuid = Uuid::from_slice(&bytes)
            .map_err(|e| IdtError::ParseError(format!("Invalid UUID bytes: {}", e)))?;

        Ok(Self {
            uuid,
            input: trimmed.to_string(),
        })
    }
}

impl ParsedId for ParsedShortUuid {
    fn kind(&self) -> IdKind {
        IdKind::ShortUuid
    }

    fn canonical(&self) -> String {
        encode_shortuuid(self.uuid.as_bytes())
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.uuid.as_bytes().to_vec()
    }

    fn timestamp(&self) -> Option<Timestamp> {
        None
    }

    fn inspect(&self) -> InspectionResult {
        let bytes = self.as_bytes();
        let version = self.uuid.get_version_num();
        let variant = format!("{:?}", self.uuid.get_variant());

        let components = json!({
            "uuid": self.uuid.to_string(),
            "uuid_version": version,
            "uuid_variant": variant,
            "alphabet": "base57 (shortuuid)",
        });

        InspectionResult {
            id_type: "shortuuid".to_string(),
            input: self.input.clone(),
            canonical: self.canonical(),
            valid: true,
            timestamp: None,
            timestamp_iso: None,
            timestamp_local_iso: None,
            version: Some(version.to_string()),
            variant: Some(variant.clone()),
            random_bits: None,
            components: Some(components),
            structure: Some(vec![
                StructureSegment {
                    name: "Encoded".to_string(),
                    size: 22,
                    unit: SizeUnit::Chars,
                    value: Some(self.canonical()),
                    description: format!(
                        "Base57 encoding using alphabet '{}'",
                        std::str::from_utf8(SHORTUUID_ALPHABET).expect("alphabet is ASCII")
                    ),
                },
                StructureSegment {
                    name: "UUID".to_string(),
                    size: 128,
                    unit: SizeUnit::Bits,
                    value: Some(self.uuid.to_string()),
                    description: format!("Decoded UUID (v{}, {})", version, variant),
                },
            ]),
            encodings: IdEncodings {
                hex: encode_hex(&bytes),
                base32: encode_base32(&bytes),
                base58: encode_base58(&bytes),
                base64: encode_base64(&bytes),
                int: Some(
                    u128::from_be_bytes(bytes.try_into().expect("UUID is 16 bytes")).to_string(),
                ),
            },
        }
    }

    fn validate(&self) -> ValidationResult {
        ValidationResult::valid("shortuuid")
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
                u128::from_be_bytes(bytes.try_into().expect("UUID is 16 bytes")).to_string()
            }
            EncodingFormat::Bytes => encode_bytes_spaced(&bytes),
        }
    }
}

/// Check if a string can be parsed as a ShortUUID.
pub fn is_shortuuid(input: &str) -> bool {
    ParsedShortUuid::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_length_and_alphabet() {
        let generator = ShortUuidGenerator::new();
        for _ in 0..8 {
            let id = generator.generate().unwrap();
            assert_eq!(id.len(), 22);
            assert!(id.bytes().all(|c| SHORTUUID_ALPHABET.contains(&c)));
        }
    }

    #[test]
    fn test_nil_uuid_known_vector() {
        let nil = Uuid::nil();
        let encoded = encode_shortuuid(nil.as_bytes());
        assert_eq!(encoded, "2222222222222222222222");
        let parsed = ParsedShortUuid::parse(&encoded).unwrap();
        assert_eq!(parsed.uuid, nil);
    }

    #[test]
    fn test_roundtrip_via_uuid() {
        let uuid = Uuid::new_v4();
        let encoded = encode_shortuuid(uuid.as_bytes());
        let parsed = ParsedShortUuid::parse(&encoded).unwrap();
        assert_eq!(parsed.uuid, uuid);
        assert_eq!(parsed.canonical(), encoded);
        assert_eq!(parsed.as_bytes(), uuid.as_bytes().to_vec());
    }

    #[test]
    fn test_parse_rejects_invalid_length() {
        assert!(ParsedShortUuid::parse("22").is_err());
        assert!(ParsedShortUuid::parse("2222222222222222222222X").is_err());
    }

    #[test]
    fn test_parse_rejects_invalid_alphabet() {
        // '!' is in neither base57 nor base58 alphabet
        assert!(ParsedShortUuid::parse("!!!!!!!!!!!!!!!!!!!!!!").is_err());
    }

    #[test]
    fn test_encode_preserves_leading_zeros() {
        let bytes = [0u8; 16];
        let encoded = encode_shortuuid(&bytes);
        assert_eq!(encoded.len(), 22);
        assert!(encoded.chars().all(|c| c == '2'));
    }

    #[test]
    fn test_kind_and_validate() {
        let id = ShortUuidGenerator::new().generate().unwrap();
        let parsed = ParsedShortUuid::parse(&id).unwrap();
        assert_eq!(parsed.kind(), IdKind::ShortUuid);
        assert!(parsed.validate().valid);
    }

    #[test]
    fn test_is_shortuuid() {
        let id = ShortUuidGenerator::new().generate().unwrap();
        assert!(is_shortuuid(&id));
        assert!(!is_shortuuid("not-a-shortuuid"));
    }

    #[test]
    fn test_timestamp_is_none() {
        let parsed = ParsedShortUuid::parse("2222222222222222222222").unwrap();
        assert!(parsed.timestamp().is_none());
    }

    #[test]
    fn test_inspect_nil_uuid() {
        let parsed = ParsedShortUuid::parse("2222222222222222222222").unwrap();
        let result = parsed.inspect();

        assert_eq!(result.id_type, "shortuuid");
        assert_eq!(result.canonical, "2222222222222222222222");
        assert!(result.valid);
        assert!(result.timestamp.is_none());
        assert!(result.timestamp_iso.is_none());
        assert!(result.random_bits.is_none());
        assert_eq!(result.version.as_deref(), Some("0"));
        assert_eq!(result.variant.as_deref(), Some("NCS"));

        let structure = result.structure.expect("structure present");
        assert_eq!(structure.len(), 2);
        assert_eq!(structure[0].name, "Encoded");
        assert!(matches!(structure[0].unit, SizeUnit::Chars));
        assert_eq!(structure[0].size, 22);
        assert_eq!(structure[1].name, "UUID");
        assert!(matches!(structure[1].unit, SizeUnit::Bits));
        assert_eq!(structure[1].size, 128);

        let components = result.components.expect("components present");
        assert_eq!(components["uuid"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(components["uuid_version"], 0);
        assert_eq!(components["alphabet"], "base57 (shortuuid)");

        assert_eq!(result.encodings.hex, "00000000000000000000000000000000");
        assert_eq!(result.encodings.int.as_deref(), Some("0"));
    }

    #[test]
    fn test_inspect_known_uuid() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let encoded = encode_shortuuid(uuid.as_bytes());
        let parsed = ParsedShortUuid::parse(&encoded).unwrap();
        let result = parsed.inspect();

        let components = result.components.expect("components present");
        assert_eq!(components["uuid"], "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(components["uuid_version"], 4);
        assert_eq!(result.version.as_deref(), Some("4"));
        assert_eq!(result.encodings.hex, "550e8400e29b41d4a716446655440000");
    }

    #[test]
    fn test_encode_all_formats() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = uuid.as_bytes().to_vec();
        let parsed = ParsedShortUuid::parse(&encode_shortuuid(&bytes)).unwrap();

        assert_eq!(parsed.encode(EncodingFormat::Canonical), parsed.canonical());
        assert_eq!(
            parsed.encode(EncodingFormat::Hex),
            "550e8400e29b41d4a716446655440000"
        );
        assert_eq!(
            parsed.encode(EncodingFormat::HexUpper),
            "550E8400E29B41D4A716446655440000"
        );
        assert_eq!(parsed.encode(EncodingFormat::Base32).len(), 26);
        assert_eq!(parsed.encode(EncodingFormat::Base32Hex).len(), 26);
        assert!(!parsed.encode(EncodingFormat::Base58).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base64).is_empty());
        assert!(!parsed.encode(EncodingFormat::Base64Url).is_empty());
        assert_eq!(parsed.encode(EncodingFormat::Bits).len(), 128);
        assert!(
            parsed
                .encode(EncodingFormat::Bits)
                .chars()
                .all(|c| c == '0' || c == '1')
        );
        assert_eq!(
            parsed.encode(EncodingFormat::Int),
            "113059749145936325402354257176981405696"
        );
        let bytes_fmt = parsed.encode(EncodingFormat::Bytes);
        assert!(bytes_fmt.starts_with("55 0e 84 00"));
        // Binary is raw bytes interpreted as UTF-8 — non-empty but may be lossy.
        assert!(!parsed.encode(EncodingFormat::Binary).is_empty());
    }
}

use crate::core::error::{IdtError, Result};
use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    Canonical,
    Hex,
    HexUpper,
    Base32,
    Base32Hex,
    Base58,
    Base64,
    Base64Url,
    Binary,
    Bits,
    Int,
    Bytes,
}

impl fmt::Display for EncodingFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodingFormat::Canonical => write!(f, "canonical"),
            EncodingFormat::Hex => write!(f, "hex"),
            EncodingFormat::HexUpper => write!(f, "HEX"),
            EncodingFormat::Base32 => write!(f, "base32"),
            EncodingFormat::Base32Hex => write!(f, "base32hex"),
            EncodingFormat::Base58 => write!(f, "base58"),
            EncodingFormat::Base64 => write!(f, "base64"),
            EncodingFormat::Base64Url => write!(f, "base64url"),
            EncodingFormat::Binary => write!(f, "binary"),
            EncodingFormat::Bits => write!(f, "bits"),
            EncodingFormat::Int => write!(f, "int"),
            EncodingFormat::Bytes => write!(f, "bytes"),
        }
    }
}

impl FromStr for EncodingFormat {
    type Err = IdtError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "canonical" => Ok(EncodingFormat::Canonical),
            "hex" => Ok(EncodingFormat::Hex),
            "hexupper" | "hex-upper" | "HEX" => Ok(EncodingFormat::HexUpper),
            "base32" => Ok(EncodingFormat::Base32),
            "base32hex" | "base32-hex" => Ok(EncodingFormat::Base32Hex),
            "base58" => Ok(EncodingFormat::Base58),
            "base64" => Ok(EncodingFormat::Base64),
            "base64url" | "base64-url" => Ok(EncodingFormat::Base64Url),
            "binary" | "bin" => Ok(EncodingFormat::Binary),
            "bits" => Ok(EncodingFormat::Bits),
            "int" | "integer" => Ok(EncodingFormat::Int),
            "bytes" => Ok(EncodingFormat::Bytes),
            _ => Err(IdtError::InvalidArgument(format!(
                "Unknown encoding format: {}",
                s
            ))),
        }
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

pub fn encode_hex_upper(bytes: &[u8]) -> String {
    hex::encode_upper(bytes)
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
    hex::decode(s).map_err(|e| IdtError::EncodingError(e.to_string()))
}

pub fn encode_base32(bytes: &[u8]) -> String {
    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, bytes)
}

pub fn decode_base32(s: &str) -> Result<Vec<u8>> {
    base32::decode(base32::Alphabet::Rfc4648 { padding: false }, s).ok_or_else(|| {
        IdtError::EncodingError(format!("Invalid base32 input '{}' (length {})", s, s.len()))
    })
}

pub fn encode_base58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

pub fn decode_base58(s: &str) -> Result<Vec<u8>> {
    bs58::decode(s)
        .into_vec()
        .map_err(|e| IdtError::EncodingError(e.to_string()))
}

pub fn encode_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

pub fn decode_base64(s: &str) -> Result<Vec<u8>> {
    general_purpose::STANDARD
        .decode(s)
        .map_err(|e| IdtError::EncodingError(e.to_string()))
}

pub fn encode_base64_url(bytes: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn decode_base64_url(s: &str) -> Result<Vec<u8>> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|e| IdtError::EncodingError(e.to_string()))
}

pub fn encode_bits(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:08b}", b))
        .collect::<Vec<_>>()
        .join("")
}

pub fn encode_bytes_spaced(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Alphabet used by Python's `shortuuid` library — 57 lookalike-free chars
/// (no 0, 1, I, O, l). Encoding produces a fixed 22-char string for 128-bit inputs,
/// left-padded with the first alphabet char ('2') to preserve leading zeros.
pub const SHORTUUID_ALPHABET: &[u8; 57] =
    b"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

const SHORTUUID_ENCODED_LEN: usize = 22;

pub fn encode_shortuuid(bytes: &[u8]) -> String {
    let Some(n) = bytes_to_u128(bytes) else {
        return format!("overflow ({} bytes, max 16)", bytes.len());
    };
    let base = SHORTUUID_ALPHABET.len() as u128;
    let mut chars: Vec<u8> = Vec::with_capacity(SHORTUUID_ENCODED_LEN);
    let mut v = n;
    while v > 0 {
        let rem = (v % base) as usize;
        chars.push(SHORTUUID_ALPHABET[rem]);
        v /= base;
    }
    while chars.len() < SHORTUUID_ENCODED_LEN {
        chars.push(SHORTUUID_ALPHABET[0]);
    }
    chars.reverse();
    String::from_utf8(chars).expect("alphabet is ASCII")
}

pub fn decode_shortuuid(s: &str) -> Result<Vec<u8>> {
    if s.len() != SHORTUUID_ENCODED_LEN {
        return Err(IdtError::EncodingError(format!(
            "Invalid shortuuid length {} (expected {})",
            s.len(),
            SHORTUUID_ENCODED_LEN
        )));
    }
    let base = SHORTUUID_ALPHABET.len() as u128;
    let mut n: u128 = 0;
    for c in s.chars() {
        let Some(idx) = SHORTUUID_ALPHABET.iter().position(|&a| a == c as u8) else {
            return Err(IdtError::EncodingError(format!(
                "Invalid shortuuid character '{}'",
                c
            )));
        };
        n = n
            .checked_mul(base)
            .and_then(|v| v.checked_add(idx as u128))
            .ok_or_else(|| IdtError::EncodingError("shortuuid overflow u128".to_string()))?;
    }
    Ok(n.to_be_bytes().to_vec())
}

pub fn bytes_to_u128(bytes: &[u8]) -> Option<u128> {
    if bytes.len() > 16 {
        return None;
    }
    let mut arr = [0u8; 16];
    let start = 16 - bytes.len();
    arr[start..].copy_from_slice(bytes);
    Some(u128::from_be_bytes(arr))
}

pub fn encode_bytes(bytes: &[u8], format: EncodingFormat) -> String {
    match format {
        EncodingFormat::Canonical => encode_hex(bytes), // Default fallback
        EncodingFormat::Hex => encode_hex(bytes),
        EncodingFormat::HexUpper => encode_hex_upper(bytes),
        EncodingFormat::Base32 => encode_base32(bytes),
        EncodingFormat::Base32Hex => encode_base32(bytes), // Simplified
        EncodingFormat::Base58 => encode_base58(bytes),
        EncodingFormat::Base64 => encode_base64(bytes),
        EncodingFormat::Base64Url => encode_base64_url(bytes),
        EncodingFormat::Binary => String::from_utf8_lossy(bytes).to_string(),
        EncodingFormat::Bits => encode_bits(bytes),
        EncodingFormat::Int => bytes_to_u128(bytes)
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("overflow ({} bytes, max 16)", bytes.len())),
        EncodingFormat::Bytes => encode_bytes_spaced(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_format_from_str() {
        assert_eq!(
            EncodingFormat::from_str("canonical").unwrap(),
            EncodingFormat::Canonical
        );
        assert_eq!(
            EncodingFormat::from_str("hex").unwrap(),
            EncodingFormat::Hex
        );
        assert_eq!(
            EncodingFormat::from_str("hexupper").unwrap(),
            EncodingFormat::HexUpper
        );
        assert_eq!(
            EncodingFormat::from_str("hex-upper").unwrap(),
            EncodingFormat::HexUpper
        );
        assert_eq!(
            EncodingFormat::from_str("base32").unwrap(),
            EncodingFormat::Base32
        );
        assert_eq!(
            EncodingFormat::from_str("base32hex").unwrap(),
            EncodingFormat::Base32Hex
        );
        assert_eq!(
            EncodingFormat::from_str("base58").unwrap(),
            EncodingFormat::Base58
        );
        assert_eq!(
            EncodingFormat::from_str("base64").unwrap(),
            EncodingFormat::Base64
        );
        assert_eq!(
            EncodingFormat::from_str("base64url").unwrap(),
            EncodingFormat::Base64Url
        );
        assert_eq!(
            EncodingFormat::from_str("binary").unwrap(),
            EncodingFormat::Binary
        );
        assert_eq!(
            EncodingFormat::from_str("bin").unwrap(),
            EncodingFormat::Binary
        );
        assert_eq!(
            EncodingFormat::from_str("bits").unwrap(),
            EncodingFormat::Bits
        );
        assert_eq!(
            EncodingFormat::from_str("int").unwrap(),
            EncodingFormat::Int
        );
        assert_eq!(
            EncodingFormat::from_str("integer").unwrap(),
            EncodingFormat::Int
        );
        assert_eq!(
            EncodingFormat::from_str("bytes").unwrap(),
            EncodingFormat::Bytes
        );
        assert!(EncodingFormat::from_str("unknown").is_err());
    }

    #[test]
    fn test_encoding_format_display() {
        assert_eq!(EncodingFormat::Canonical.to_string(), "canonical");
        assert_eq!(EncodingFormat::Hex.to_string(), "hex");
        assert_eq!(EncodingFormat::HexUpper.to_string(), "HEX");
        assert_eq!(EncodingFormat::Base32.to_string(), "base32");
        assert_eq!(EncodingFormat::Base32Hex.to_string(), "base32hex");
        assert_eq!(EncodingFormat::Base58.to_string(), "base58");
        assert_eq!(EncodingFormat::Base64.to_string(), "base64");
        assert_eq!(EncodingFormat::Base64Url.to_string(), "base64url");
        assert_eq!(EncodingFormat::Binary.to_string(), "binary");
        assert_eq!(EncodingFormat::Bits.to_string(), "bits");
        assert_eq!(EncodingFormat::Int.to_string(), "int");
        assert_eq!(EncodingFormat::Bytes.to_string(), "bytes");
    }

    #[test]
    fn test_encode_shortuuid_nil() {
        let bytes = [0u8; 16];
        assert_eq!(encode_shortuuid(&bytes), "2222222222222222222222");
    }

    #[test]
    fn test_encode_shortuuid_length() {
        let bytes = [0xffu8; 16];
        assert_eq!(encode_shortuuid(&bytes).len(), 22);
    }

    #[test]
    fn test_shortuuid_roundtrip() {
        let bytes: [u8; 16] = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];
        let encoded = encode_shortuuid(&bytes);
        assert_eq!(encoded.len(), 22);
        let decoded = decode_shortuuid(&encoded).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_decode_shortuuid_invalid_char() {
        // '0' and '1' are outside the base57 alphabet
        assert!(decode_shortuuid("0000000000000000000000").is_err());
        assert!(decode_shortuuid("1111111111111111111111").is_err());
    }

    #[test]
    fn test_decode_shortuuid_invalid_length() {
        assert!(decode_shortuuid("222222222").is_err());
    }

    #[test]
    fn test_encode_decode_hex() {
        let data = b"hello";
        let encoded = encode_hex(data);
        assert_eq!(encoded, "68656c6c6f");
        let decoded = decode_hex(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_hex_upper() {
        assert_eq!(encode_hex_upper(b"hello"), "68656C6C6F");
    }

    #[test]
    fn test_decode_hex_invalid() {
        assert!(decode_hex("xyz").is_err());
    }

    #[test]
    fn test_encode_decode_base32() {
        let data = b"hello";
        let encoded = encode_base32(data);
        let decoded = decode_base32(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_base32_invalid() {
        assert!(decode_base32("!!!").is_err());
    }

    #[test]
    fn test_encode_decode_base58() {
        let data = b"hello";
        let encoded = encode_base58(data);
        let decoded = decode_base58(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_base58_invalid() {
        assert!(decode_base58("0OIl").is_err());
    }

    #[test]
    fn test_encode_decode_base64() {
        let data = b"hello";
        let encoded = encode_base64(data);
        assert_eq!(encoded, "aGVsbG8=");
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_base64_invalid() {
        assert!(decode_base64("!!!").is_err());
    }

    #[test]
    fn test_encode_decode_base64_url() {
        let data = b"\xff\xfe\xfd";
        let encoded = encode_base64_url(data);
        let decoded = decode_base64_url(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_base64_url_invalid() {
        assert!(decode_base64_url("===").is_err());
    }

    #[test]
    fn test_encode_bits() {
        assert_eq!(encode_bits(&[0xff, 0x00]), "1111111100000000");
        assert_eq!(encode_bits(&[0xab]), "10101011");
    }

    #[test]
    fn test_encode_bytes_spaced() {
        assert_eq!(encode_bytes_spaced(&[0xab, 0xcd, 0xef]), "ab cd ef");
    }

    #[test]
    fn test_bytes_to_u128() {
        assert_eq!(bytes_to_u128(&[0x01]), Some(1));
        assert_eq!(bytes_to_u128(&[0x00; 16]), Some(0));
        assert_eq!(bytes_to_u128(&[0x00; 17]), None); // too long
    }

    #[test]
    fn test_encode_bytes_all_formats() {
        let data = &[0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
        assert_eq!(encode_bytes(data, EncodingFormat::Hex), encode_hex(data));
        assert_eq!(
            encode_bytes(data, EncodingFormat::HexUpper),
            encode_hex_upper(data)
        );
        assert_eq!(
            encode_bytes(data, EncodingFormat::Base32),
            encode_base32(data)
        );
        assert_eq!(
            encode_bytes(data, EncodingFormat::Base58),
            encode_base58(data)
        );
        assert_eq!(
            encode_bytes(data, EncodingFormat::Base64),
            encode_base64(data)
        );
        assert_eq!(
            encode_bytes(data, EncodingFormat::Base64Url),
            encode_base64_url(data)
        );
        assert_eq!(
            encode_bytes(data, EncodingFormat::Binary),
            "Hello".to_string()
        );
        assert_eq!(encode_bytes(data, EncodingFormat::Bits), encode_bits(data));
        assert_eq!(
            encode_bytes(data, EncodingFormat::Bytes),
            encode_bytes_spaced(data)
        );
        // Int format
        let int_result = encode_bytes(data, EncodingFormat::Int);
        assert!(!int_result.is_empty());
    }

    #[test]
    fn test_encode_bytes_int_overflow() {
        let data = &[0xff; 17]; // > 16 bytes
        assert_eq!(
            encode_bytes(data, EncodingFormat::Int),
            "overflow (17 bytes, max 16)"
        );
    }
}

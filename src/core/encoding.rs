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
    base32::decode(base32::Alphabet::Rfc4648 { padding: false }, s)
        .ok_or_else(|| IdtError::EncodingError("Invalid base32".to_string()))
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
            .unwrap_or_else(|| "overflow".to_string()),
        EncodingFormat::Bytes => encode_bytes_spaced(bytes),
    }
}

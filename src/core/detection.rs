use crate::core::error::Result;
use crate::core::id::IdKind;

/// Detection result with confidence score
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub kind: IdKind,
    pub confidence: f32,
}

impl DetectionResult {
    pub fn new(kind: IdKind, confidence: f32) -> Self {
        Self { kind, confidence }
    }
}

/// Detect the ID type from a string
pub fn detect_id_type(input: &str) -> Result<Vec<DetectionResult>> {
    let input = input.trim();
    let mut results = Vec::new();

    // Check UUID format (with dashes)
    if is_uuid_format(input) {
        if let Some(version) = detect_uuid_version(input) {
            results.push(DetectionResult::new(version, 1.0));
        } else {
            results.push(DetectionResult::new(IdKind::Uuid, 0.9));
        }
    }

    // Check UUID format (without dashes - 32 hex chars)
    if input.len() == 32 && input.chars().all(|c| c.is_ascii_hexdigit()) {
        results.push(DetectionResult::new(IdKind::Uuid, 0.7));
    }

    // Check ULID format (26 chars, Crockford Base32)
    if is_ulid_format(input) {
        results.push(DetectionResult::new(IdKind::Ulid, 0.95));
    }

    // Check Snowflake (numeric, 15-19 digits)
    if is_snowflake_format(input) {
        results.push(DetectionResult::new(IdKind::Snowflake, 0.8));
    }

    // Check NanoID (21 chars by default, URL-safe alphabet)
    if is_nanoid_format(input) {
        results.push(DetectionResult::new(IdKind::NanoId, 0.6));
    }

    // Sort by confidence descending
    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    if results.is_empty() {
        Err(crate::core::error::IdtError::DetectionFailed)
    } else {
        Ok(results)
    }
}

/// Check if input matches UUID format with dashes
fn is_uuid_format(input: &str) -> bool {
    if input.len() != 36 {
        return false;
    }

    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() != 5 {
        return false;
    }

    let expected_lens = [8, 4, 4, 4, 12];
    for (part, &expected_len) in parts.iter().zip(&expected_lens) {
        if part.len() != expected_len || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }

    true
}

/// Detect UUID version from formatted UUID string
fn detect_uuid_version(input: &str) -> Option<IdKind> {
    let input = input.replace('-', "");
    if input.len() != 32 {
        return None;
    }

    // Check for nil UUID
    if input.chars().all(|c| c == '0') {
        return Some(IdKind::UuidNil);
    }

    // Check for max UUID
    if input.to_lowercase().chars().all(|c| c == 'f') {
        return Some(IdKind::UuidMax);
    }

    // Version is in the 13th character (index 12)
    let version_char = input.chars().nth(12)?;
    let version = version_char.to_digit(16)?;

    // Variant is in the 17th character (index 16)
    let variant_char = input.chars().nth(16)?;
    let variant = variant_char.to_digit(16)?;

    // Check variant is RFC 4122 (8, 9, a, b)
    if !(8..=11).contains(&variant) {
        return Some(IdKind::Uuid); // Valid UUID but unknown variant
    }

    match version {
        1 => Some(IdKind::UuidV1),
        3 => Some(IdKind::UuidV3),
        4 => Some(IdKind::UuidV4),
        5 => Some(IdKind::UuidV5),
        6 => Some(IdKind::UuidV6),
        7 => Some(IdKind::UuidV7),
        _ => Some(IdKind::Uuid),
    }
}

/// Check if input matches ULID format
fn is_ulid_format(input: &str) -> bool {
    if input.len() != 26 {
        return false;
    }

    // ULID uses Crockford Base32 alphabet (case-insensitive)
    // Excludes I, L, O, U
    let input_upper = input.to_uppercase();

    // First character must be 0-7 (timestamp constraint)
    let first = input_upper.chars().next().unwrap();
    if !('0'..='7').contains(&first) {
        return false;
    }

    // All characters must be valid Crockford Base32
    const CROCKFORD: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    input_upper.chars().all(|c| CROCKFORD.contains(c))
}

/// Check if input matches Snowflake format (numeric, 15-19 digits)
fn is_snowflake_format(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    // Must be all digits
    if !input.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Snowflake IDs are typically 15-19 digits
    // (depends on epoch and timestamp)
    let len = input.len();
    (15..=19).contains(&len)
}

/// Check if input matches NanoID format
fn is_nanoid_format(input: &str) -> bool {
    // Default NanoID is 21 characters
    // Uses URL-safe alphabet: A-Za-z0-9_-
    if input.len() != 21 {
        return false;
    }

    input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_uuidv4() {
        let results = detect_id_type("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].kind, IdKind::UuidV4);
    }

    #[test]
    fn test_detect_ulid() {
        let results = detect_id_type("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].kind, IdKind::Ulid);
    }

    #[test]
    fn test_detect_snowflake() {
        let results = detect_id_type("1234567890123456789").unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].kind, IdKind::Snowflake);
    }
}

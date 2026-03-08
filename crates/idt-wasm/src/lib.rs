use wasm_bindgen::prelude::*;

use idt::core::detection::detect_id_type;
use idt::core::id::IdKind;
use idt::ids::{create_generator, parse_id};

/// Generate one or more IDs of the given type.
/// Returns a JSON string: array of ID strings.
#[wasm_bindgen]
pub fn generate(id_type: &str, count: u32) -> Result<String, JsValue> {
    let kind: IdKind = id_type
        .parse()
        .map_err(|e: idt::core::error::IdtError| JsValue::from_str(&e.to_string()))?;

    let generator =
        create_generator(kind).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let ids: Vec<String> = (0..count)
        .map(|_| generator.generate())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_string(&ids).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Inspect an ID value. Returns a JSON string with inspection details.
#[wasm_bindgen]
pub fn inspect(id_value: &str, type_hint: Option<String>) -> Result<String, JsValue> {
    let hint = type_hint
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<IdKind>())
        .transpose()
        .map_err(|e: idt::core::error::IdtError| JsValue::from_str(&e.to_string()))?;

    let parsed = parse_id(id_value, hint).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = parsed.inspect();

    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Validate an ID value. Returns a JSON string with validation result.
#[wasm_bindgen]
pub fn validate(id_value: &str, type_hint: Option<String>) -> Result<String, JsValue> {
    let hint = type_hint
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<IdKind>())
        .transpose()
        .map_err(|e: idt::core::error::IdtError| JsValue::from_str(&e.to_string()))?;

    let parsed = parse_id(id_value, hint).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = parsed.validate();

    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Detect the type of an ID value. Returns a JSON string: array of {kind, confidence}.
#[wasm_bindgen]
pub fn detect(id_value: &str) -> Result<String, JsValue> {
    let results = detect_id_type(id_value).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let detections: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "kind": r.kind.name(),
                "confidence": r.confidence,
            })
        })
        .collect();

    serde_json::to_string(&detections).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// List all generatable ID types. Returns a JSON string: array of {name, description}.
#[wasm_bindgen]
pub fn list_types() -> String {
    let types: Vec<serde_json::Value> = IdKind::generatable()
        .iter()
        .map(|kind| {
            serde_json::json!({
                "name": kind.name(),
                "description": kind.description(),
            })
        })
        .collect();

    serde_json::to_string(&types).unwrap()
}

use crate::cli::app::OutputFormat;
use crate::core::error::Result;
use serde::Serialize;

pub fn format_output<T: Serialize>(
    value: &T,
    format: OutputFormat,
    pretty: bool,
) -> Result<String> {
    match format {
        OutputFormat::Json => {
            if pretty {
                Ok(serde_json::to_string_pretty(value)?)
            } else {
                Ok(serde_json::to_string(value)?)
            }
        }
        OutputFormat::Yaml => Ok(serde_yaml_ng::to_string(value)?),
        OutputFormat::Toml => {
            let toml_value = toml::Value::try_from(value)
                .map_err(|e| crate::core::error::IdtError::SerializationError(e.to_string()))?;
            Ok(toml::to_string_pretty(&toml_value)
                .map_err(|e| crate::core::error::IdtError::SerializationError(e.to_string()))?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_compact() {
        let value = json!({"key": "value"});
        let result = format_output(&value, OutputFormat::Json, false).unwrap();
        assert_eq!(result, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_json_pretty() {
        let value = json!({"key": "value"});
        let result = format_output(&value, OutputFormat::Json, true).unwrap();
        assert!(result.contains('\n'));
        assert!(result.contains("key"));
    }

    #[test]
    fn test_yaml() {
        let value = json!({"key": "value"});
        let result = format_output(&value, OutputFormat::Yaml, false).unwrap();
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_toml() {
        let value = json!({"key": "value"});
        let result = format_output(&value, OutputFormat::Toml, false).unwrap();
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }
}

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

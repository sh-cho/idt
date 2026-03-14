use crate::cli::app::GenArgs;
use crate::core::EncodingFormat;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdGenerator, IdKind};
use crate::ids::snowflake_id::SnowflakeLayout;
use crate::ids::{NanoIdGenerator, SnowflakeGenerator, TypeIdGenerator, UuidGenerator};
use std::io::{self, Write};

pub fn execute(args: &GenArgs, json_output: bool, pretty: bool) -> Result<()> {
    if json_output && args.template.is_some() {
        return Err(IdtError::InvalidArgument(
            "--template cannot be used with --json".into(),
        ));
    }

    if let Some(ref tpl) = args.template
        && !tpl.contains("{}")
    {
        eprintln!(
            "warning: template does not contain '{{}}' placeholder; IDs will not appear in output"
        );
    }

    let kind = args.id_type;
    let ids = generate_ids(args, kind)?;

    let mut writer: Box<dyn Write> = Box::new(io::stdout());

    // Apply format conversion if specified
    let format: Option<EncodingFormat> = args.format.as_ref().map(|f| f.parse()).transpose()?;

    let formatted_ids: Vec<String> = if let Some(fmt) = format {
        ids.iter()
            .map(|id| format_id(id, &kind, fmt))
            .collect::<Result<Vec<_>>>()?
    } else {
        ids
    };

    // Apply template if specified
    let final_ids = if let Some(ref tpl) = args.template {
        formatted_ids
            .iter()
            .map(|id| tpl.replace("{}", id))
            .collect()
    } else {
        formatted_ids
    };

    // Output
    if json_output {
        output_json(&mut writer, &final_ids, pretty)?;
    } else {
        output_plain(&mut writer, &final_ids, args.no_newline && args.count == 1)?;
    }

    Ok(())
}

fn generate_ids(args: &GenArgs, kind: IdKind) -> Result<Vec<String>> {
    let mut ids = Vec::with_capacity(args.count);

    match kind {
        IdKind::Uuid | IdKind::UuidV4 => {
            let version = args.uuid_version.unwrap_or(4);
            let generator = match version {
                1 => UuidGenerator::v1(),
                4 => UuidGenerator::v4(),
                6 => UuidGenerator::v6(),
                7 => UuidGenerator::v7(),
                _ => {
                    return Err(IdtError::InvalidArgument(format!(
                        "UUID version {} not supported for generation. Use 1, 4, 6, or 7.",
                        version
                    )));
                }
            };
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        IdKind::UuidV1 => {
            let generator = UuidGenerator::v1();
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        IdKind::UuidV6 => {
            let generator = UuidGenerator::v6();
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        IdKind::UuidV7 => {
            let generator = UuidGenerator::v7();
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        IdKind::UuidNil => {
            let generator = UuidGenerator::nil();
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        IdKind::UuidMax => {
            let generator = UuidGenerator::max();
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        IdKind::Ulid => {
            let generator = crate::ids::UlidGenerator::new();
            for _ in 0..args.count {
                ids.push(crate::core::id::IdGenerator::generate(&generator)?);
            }
        }
        IdKind::NanoId => {
            let mut generator = NanoIdGenerator::new();
            if let Some(ref alphabet) = args.alphabet {
                generator = generator.with_alphabet(alphabet);
            }
            if let Some(length) = args.length {
                generator = generator.with_length(length);
            }
            for _ in 0..args.count {
                ids.push(crate::core::id::IdGenerator::generate(&generator)?);
            }
        }
        IdKind::Snowflake => {
            let layout = SnowflakeLayout::resolve(args.preset.as_deref(), args.epoch.as_deref())?;

            let mut generator = SnowflakeGenerator::new().with_layout(layout);

            if let Some(machine_id) = args.machine_id {
                if !generator.layout.has_field("machine_id") {
                    return Err(IdtError::InvalidArgument(format!(
                        "Preset '{}' does not have a machine_id field",
                        generator.layout.name
                    )));
                }
                generator = generator.with_machine_id(machine_id);
            }
            if let Some(datacenter_id) = args.datacenter_id {
                if !generator.layout.has_field("datacenter_id") {
                    return Err(IdtError::InvalidArgument(format!(
                        "Preset '{}' does not have a datacenter_id field",
                        generator.layout.name
                    )));
                }
                generator = generator.with_datacenter_id(datacenter_id);
            }

            // Handle --field key=value pairs
            for field_arg in &args.field {
                let (name, value) = field_arg.split_once('=').ok_or_else(|| {
                    IdtError::InvalidArgument(format!(
                        "Invalid --field format '{}': expected NAME=VALUE",
                        field_arg
                    ))
                })?;
                if !generator.layout.has_field(name) {
                    return Err(IdtError::InvalidArgument(format!(
                        "Preset '{}' does not have a '{}' field. Available: {}",
                        generator.layout.name,
                        name,
                        generator
                            .layout
                            .fields
                            .iter()
                            .filter(|f| f.name != "timestamp" && f.name != "sequence")
                            .map(|f| f.name)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }
                let val: u64 = value.parse().map_err(|_| {
                    IdtError::InvalidArgument(format!(
                        "Invalid value '{}' for field '{}': expected integer",
                        value, name
                    ))
                })?;
                generator = generator.with_field(name, val);
            }

            for _ in 0..args.count {
                ids.push(crate::core::id::IdGenerator::generate(&generator)?);
            }
        }
        IdKind::ObjectId
        | IdKind::Ksuid
        | IdKind::Xid
        | IdKind::Tsid
        | IdKind::Cuid
        | IdKind::Cuid2 => {
            let generator = crate::ids::create_generator(kind)?;
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        IdKind::TypeId => {
            let prefix = args.prefix.as_deref().unwrap_or("");
            let generator = TypeIdGenerator::new(prefix);
            for _ in 0..args.count {
                ids.push(generator.generate()?);
            }
        }
        _ => {
            return Err(IdtError::GenerationError(format!(
                "Generation not supported for: {}",
                kind.name()
            )));
        }
    }

    Ok(ids)
}

fn format_id(id: &str, kind: &IdKind, format: EncodingFormat) -> Result<String> {
    // Parse and re-encode
    let parsed = crate::ids::parse_id(id, Some(*kind))?;
    Ok(parsed.encode(format))
}

fn output_json(writer: &mut dyn Write, ids: &[String], pretty: bool) -> Result<()> {
    let output = if ids.len() == 1 {
        serde_json::json!({ "id": ids[0] })
    } else {
        serde_json::json!(ids)
    };

    if pretty {
        writeln!(writer, "{}", serde_json::to_string_pretty(&output)?)?;
    } else {
        writeln!(writer, "{}", serde_json::to_string(&output)?)?;
    }
    Ok(())
}

fn output_plain(writer: &mut dyn Write, ids: &[String], no_newline: bool) -> Result<()> {
    for (i, id) in ids.iter().enumerate() {
        if i == ids.len() - 1 && no_newline {
            write!(writer, "{}", id)?;
        } else {
            writeln!(writer, "{}", id)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_template_basic() {
        let ids = ["abc123".to_string()];
        let tpl = "id={}";
        let result: Vec<String> = ids.iter().map(|id| tpl.replace("{}", id)).collect();
        assert_eq!(result, ["id=abc123"]);
    }

    #[test]
    fn test_template_sql() {
        let ids = ["abc123".to_string()];
        let tpl = "INSERT INTO users (id) VALUES ('{}');";
        let result: Vec<String> = ids.iter().map(|id| tpl.replace("{}", id)).collect();
        assert_eq!(result, ["INSERT INTO users (id) VALUES ('abc123');"]);
    }

    #[test]
    fn test_template_multiple_placeholders() {
        let ids = ["abc".to_string()];
        let tpl = "{}-{}";
        let result: Vec<String> = ids.iter().map(|id| tpl.replace("{}", id)).collect();
        assert_eq!(result, ["abc-abc"]);
    }

    #[test]
    fn test_template_no_placeholder() {
        let tpl = "no placeholder here";
        assert!(!tpl.contains("{}"));
    }
}

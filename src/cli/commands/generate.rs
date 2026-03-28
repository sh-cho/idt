use crate::cli::app::{GenArgs, OutputFormat};
use crate::cli::output::format_output;
use crate::core::EncodingFormat;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdGenerator, IdKind};
use crate::ids::snowflake_id::SnowflakeLayout;
use crate::ids::{NanoIdGenerator, SnowflakeGenerator, TypeIdGenerator, UuidGenerator};
use std::io::{self, Write};

pub fn execute(args: &GenArgs, output_format: Option<OutputFormat>, pretty: bool) -> Result<()> {
    if output_format.is_some() && args.template.is_some() {
        return Err(IdtError::InvalidArgument(
            "--template cannot be used with structured output formats".into(),
        ));
    }

    if let Some(ref tpl) = args.template
        && !tpl.contains("{}")
    {
        eprintln!(
            "warning: template does not contain '{{}}' placeholder; IDs will not appear in output"
        );
    }

    let kind: IdKind = args.id_type.into();
    let ids = generate_ids(args, kind)?;

    let mut writer: Box<dyn Write> = Box::new(io::stdout());

    // Apply encoding format conversion if specified
    let encoding: Option<EncodingFormat> = args.format.as_ref().map(|f| f.parse()).transpose()?;

    let formatted_ids: Vec<String> = if let Some(enc) = encoding {
        ids.iter()
            .map(|id| format_id(id, &kind, enc))
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
    if let Some(fmt) = output_format {
        let output = if final_ids.len() == 1 {
            let wrapper = serde_json::json!({ "id": final_ids[0] });
            format_output(&wrapper, fmt, pretty)?
        } else {
            format_output(&final_ids, fmt, pretty)?
        };
        writeln!(writer, "{}", output)?;
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
        _ => unreachable!("GenIdKind only contains generable types"),
    }

    Ok(ids)
}

fn format_id(id: &str, kind: &IdKind, format: EncodingFormat) -> Result<String> {
    // Parse and re-encode
    let parsed = crate::ids::parse_id(id, Some(*kind))?;
    Ok(parsed.encode(format))
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

    use super::*;
    use crate::cli::app::GenArgs;
    use crate::core::id::{GenIdKind, IdKind};

    fn make_gen_args(kind: GenIdKind) -> GenArgs {
        GenArgs {
            id_type: kind,
            count: 1,
            format: None,
            no_newline: false,
            template: None,
            uuid_version: None,
            alphabet: None,
            length: None,
            epoch: None,
            preset: None,
            field: vec![],
            machine_id: None,
            datacenter_id: None,
            prefix: None,
        }
    }

    #[test]
    fn test_generate_uuid_v4() {
        let args = make_gen_args(GenIdKind::UuidV4);
        let ids = generate_ids(&args, IdKind::UuidV4).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 36);
    }

    #[test]
    fn test_generate_uuid_v7() {
        let args = make_gen_args(GenIdKind::UuidV7);
        let ids = generate_ids(&args, IdKind::UuidV7).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_generate_uuid_v1() {
        let args = make_gen_args(GenIdKind::UuidV1);
        let ids = generate_ids(&args, IdKind::UuidV1).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_generate_uuid_v6() {
        let args = make_gen_args(GenIdKind::UuidV6);
        let ids = generate_ids(&args, IdKind::UuidV6).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_generate_uuid_nil() {
        let args = make_gen_args(GenIdKind::UuidNil);
        let ids = generate_ids(&args, IdKind::UuidNil).unwrap();
        assert_eq!(ids[0], "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_generate_uuid_max() {
        let args = make_gen_args(GenIdKind::UuidMax);
        let ids = generate_ids(&args, IdKind::UuidMax).unwrap();
        assert_eq!(ids[0], "ffffffff-ffff-ffff-ffff-ffffffffffff");
    }

    #[test]
    fn test_generate_uuid_with_version() {
        let mut args = make_gen_args(GenIdKind::Uuid);
        args.uuid_version = Some(7);
        let ids = generate_ids(&args, IdKind::Uuid).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_generate_uuid_unsupported_version() {
        let mut args = make_gen_args(GenIdKind::Uuid);
        args.uuid_version = Some(99);
        assert!(generate_ids(&args, IdKind::Uuid).is_err());
    }

    #[test]
    fn test_generate_ulid() {
        let args = make_gen_args(GenIdKind::Ulid);
        let ids = generate_ids(&args, IdKind::Ulid).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 26);
    }

    #[test]
    fn test_generate_nanoid() {
        let args = make_gen_args(GenIdKind::NanoId);
        let ids = generate_ids(&args, IdKind::NanoId).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 21);
    }

    #[test]
    fn test_generate_nanoid_custom() {
        let mut args = make_gen_args(GenIdKind::NanoId);
        args.alphabet = Some("abc".to_string());
        args.length = Some(10);
        let ids = generate_ids(&args, IdKind::NanoId).unwrap();
        assert_eq!(ids[0].len(), 10);
        assert!(ids[0].chars().all(|c| "abc".contains(c)));
    }

    #[test]
    fn test_generate_snowflake() {
        let args = make_gen_args(GenIdKind::Snowflake);
        let ids = generate_ids(&args, IdKind::Snowflake).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_generate_objectid() {
        let args = make_gen_args(GenIdKind::ObjectId);
        let ids = generate_ids(&args, IdKind::ObjectId).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 24);
    }

    #[test]
    fn test_generate_ksuid() {
        let args = make_gen_args(GenIdKind::Ksuid);
        let ids = generate_ids(&args, IdKind::Ksuid).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 27);
    }

    #[test]
    fn test_generate_xid() {
        let args = make_gen_args(GenIdKind::Xid);
        let ids = generate_ids(&args, IdKind::Xid).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 20);
    }

    #[test]
    fn test_generate_tsid() {
        let args = make_gen_args(GenIdKind::Tsid);
        let ids = generate_ids(&args, IdKind::Tsid).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 13);
    }

    #[test]
    fn test_generate_cuid() {
        let args = make_gen_args(GenIdKind::Cuid);
        let ids = generate_ids(&args, IdKind::Cuid).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 25);
    }

    #[test]
    fn test_generate_cuid2() {
        let args = make_gen_args(GenIdKind::Cuid2);
        let ids = generate_ids(&args, IdKind::Cuid2).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 24);
    }

    #[test]
    fn test_generate_typeid() {
        let mut args = make_gen_args(GenIdKind::TypeId);
        args.prefix = Some("user".to_string());
        let ids = generate_ids(&args, IdKind::TypeId).unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids[0].starts_with("user_"));
    }

    #[test]
    fn test_generate_multiple() {
        let mut args = make_gen_args(GenIdKind::UuidV4);
        args.count = 5;
        let ids = generate_ids(&args, IdKind::UuidV4).unwrap();
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn test_format_id_hex() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let result = format_id(id, &IdKind::UuidV4, EncodingFormat::Hex).unwrap();
        assert!(!result.is_empty());
        assert!(!result.contains('-'));
    }

    #[test]
    fn test_output_plain_single() {
        let mut buf = Vec::new();
        let ids = vec!["test-id".to_string()];
        output_plain(&mut buf, &ids, false).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "test-id\n");
    }

    #[test]
    fn test_output_plain_no_newline() {
        let mut buf = Vec::new();
        let ids = vec!["test-id".to_string()];
        output_plain(&mut buf, &ids, true).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "test-id");
    }

    #[test]
    fn test_output_plain_multiple() {
        let mut buf = Vec::new();
        let ids = vec!["id1".to_string(), "id2".to_string()];
        output_plain(&mut buf, &ids, false).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "id1\nid2\n");
    }

    #[test]
    fn test_execute_template_with_format_error() {
        let mut args = make_gen_args(GenIdKind::UuidV4);
        args.template = Some("{}".to_string());
        let result = execute(&args, Some(OutputFormat::Json), false);
        assert!(result.is_err());
    }

    // --- Group A: execute() function coverage ---

    #[test]
    fn test_execute_plain_output() {
        let args = make_gen_args(GenIdKind::UuidV4);
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_encoding_format() {
        let mut args = make_gen_args(GenIdKind::UuidV4);
        args.format = Some("hex".to_string());
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_template() {
        let mut args = make_gen_args(GenIdKind::UuidV4);
        args.template = Some("id={}".to_string());
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_template_no_placeholder_warning() {
        let mut args = make_gen_args(GenIdKind::UuidV4);
        args.template = Some("no placeholder here".to_string());
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_json_single() {
        let args = make_gen_args(GenIdKind::UuidV4);
        let result = execute(&args, Some(OutputFormat::Json), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_json_multiple() {
        let mut args = make_gen_args(GenIdKind::UuidV4);
        args.count = 3;
        let result = execute(&args, Some(OutputFormat::Json), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_yaml_output() {
        let args = make_gen_args(GenIdKind::UuidV4);
        let result = execute(&args, Some(OutputFormat::Yaml), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_toml_output() {
        let args = make_gen_args(GenIdKind::UuidV4);
        let result = execute(&args, Some(OutputFormat::Toml), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_json_pretty() {
        let args = make_gen_args(GenIdKind::UuidV4);
        let result = execute(&args, Some(OutputFormat::Json), true);
        assert!(result.is_ok());
    }

    // --- Group B: UUID version branches via Uuid kind ---

    #[test]
    fn test_generate_uuid_kind_version_1() {
        let mut args = make_gen_args(GenIdKind::Uuid);
        args.uuid_version = Some(1);
        let ids = generate_ids(&args, IdKind::Uuid).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 36);
    }

    #[test]
    fn test_generate_uuid_kind_version_6() {
        let mut args = make_gen_args(GenIdKind::Uuid);
        args.uuid_version = Some(6);
        let ids = generate_ids(&args, IdKind::Uuid).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].len(), 36);
    }

    // --- Group C: Snowflake validation & field parsing ---

    #[test]
    fn test_snowflake_with_machine_id() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("twitter".to_string());
        args.machine_id = Some(1);
        let ids = generate_ids(&args, IdKind::Snowflake).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_snowflake_machine_id_rejected() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("instagram".to_string());
        args.machine_id = Some(1);
        let err = generate_ids(&args, IdKind::Snowflake).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("does not have a machine_id field"));
    }

    #[test]
    fn test_snowflake_with_datacenter_id() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("twitter".to_string());
        args.datacenter_id = Some(1);
        let ids = generate_ids(&args, IdKind::Snowflake).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_snowflake_datacenter_id_rejected() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("sonyflake".to_string());
        args.datacenter_id = Some(1);
        let err = generate_ids(&args, IdKind::Snowflake).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("does not have a datacenter_id field"));
    }

    #[test]
    fn test_snowflake_field_valid() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("twitter".to_string());
        args.field = vec!["machine_id=5".to_string()];
        let ids = generate_ids(&args, IdKind::Snowflake).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_snowflake_field_missing_equals() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("twitter".to_string());
        args.field = vec!["badformat".to_string()];
        let err = generate_ids(&args, IdKind::Snowflake).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("expected NAME=VALUE"));
    }

    #[test]
    fn test_snowflake_field_unknown_name() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("twitter".to_string());
        args.field = vec!["nonexistent=1".to_string()];
        let err = generate_ids(&args, IdKind::Snowflake).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("does not have a 'nonexistent' field"));
    }

    #[test]
    fn test_snowflake_field_invalid_value() {
        let mut args = make_gen_args(GenIdKind::Snowflake);
        args.preset = Some("twitter".to_string());
        args.field = vec!["machine_id=abc".to_string()];
        let err = generate_ids(&args, IdKind::Snowflake).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("expected integer"));
    }
}

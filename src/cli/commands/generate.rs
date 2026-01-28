use crate::cli::app::GenArgs;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdGenerator, IdKind};
use crate::core::EncodingFormat;
use crate::ids::{NanoIdGenerator, SnowflakeGenerator, UuidGenerator};
use crate::ids::{DISCORD_EPOCH, TWITTER_EPOCH};
use std::fs::File;
use std::io::{self, Write};

pub fn execute(args: &GenArgs, json_output: bool, pretty: bool) -> Result<()> {
    let kind: IdKind = args.id_type.parse()?;
    let ids = generate_ids(args, kind)?;

    // Determine output destination
    let mut writer: Box<dyn Write> = if let Some(ref path) = args.output {
        Box::new(File::create(path)?)
    } else {
        Box::new(io::stdout())
    };

    // Apply format conversion if specified
    let format: Option<EncodingFormat> = args
        .format
        .as_ref()
        .map(|f| f.parse())
        .transpose()?;

    let formatted_ids: Vec<String> = if let Some(fmt) = format {
        ids.iter()
            .map(|id| format_id(id, &kind, fmt))
            .collect::<Result<Vec<_>>>()?
    } else {
        ids
    };

    // Output
    if json_output {
        output_json(&mut writer, &formatted_ids, pretty)?;
    } else {
        output_plain(&mut writer, &formatted_ids, args.no_newline && args.count == 1)?;
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
                _ => return Err(IdtError::InvalidArgument(format!(
                    "UUID version {} not supported for generation. Use 1, 4, 6, or 7.",
                    version
                ))),
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
            let mut generator = SnowflakeGenerator::new();

            // Handle named epochs
            if let Some(ref epoch_str) = args.epoch.map(|e| e.to_string()).or_else(|| {
                std::env::var("IDT_SNOWFLAKE_EPOCH").ok()
            }) {
                let epoch = match epoch_str.to_lowercase().as_str() {
                    "twitter" => TWITTER_EPOCH,
                    "discord" => DISCORD_EPOCH,
                    _ => epoch_str.parse().map_err(|_| {
                        IdtError::InvalidArgument(format!("Invalid epoch: {}", epoch_str))
                    })?,
                };
                generator = generator.with_epoch(epoch);
            } else if let Some(epoch) = args.epoch {
                generator = generator.with_epoch(epoch);
            }

            if let Some(machine_id) = args.machine_id {
                generator = generator.with_machine_id(machine_id);
            }
            if let Some(datacenter_id) = args.datacenter_id {
                generator = generator.with_datacenter_id(datacenter_id);
            }

            for _ in 0..args.count {
                ids.push(crate::core::id::IdGenerator::generate(&generator)?);
            }
        }
        _ => {
            return Err(IdtError::GenerationError(format!(
                "Generation not supported for: {}. Try: uuid, ulid, nanoid, snowflake",
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

use crate::cli::app::InspectArgs;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdKind, InspectionResult, ParsedId};
use colored::Colorize;
use std::io::{self, BufRead, Write};

pub fn execute(args: &InspectArgs, json_output: bool, pretty: bool, no_color: bool) -> Result<()> {
    let ids = collect_ids(&args.ids)?;

    if ids.is_empty() {
        return Err(IdtError::InvalidArgument(
            "No IDs provided. Pass IDs as arguments or via stdin.".to_string(),
        ));
    }

    let type_hint: Option<IdKind> = args.id_type.as_ref().map(|t| t.parse()).transpose()?;
    let epoch = resolve_epoch(&args.epoch)?;

    let mut results = Vec::new();
    let mut had_errors = false;

    for id in &ids {
        let parse_result: Result<Box<dyn ParsedId>> = if let Some(epoch_ms) = epoch {
            crate::ids::ParsedSnowflake::parse_with_epoch(id, epoch_ms)
                .map(|s| Box::new(s) as Box<dyn ParsedId>)
        } else {
            crate::ids::parse_id(id, type_hint)
        };

        match parse_result {
            Ok(parsed) => {
                let mut inspection = parsed.inspect();
                if let Some(ref ts) = inspection.timestamp {
                    inspection.timestamp_local_iso = Some(ts.to_local_iso8601());
                }
                results.push(inspection);
            }
            Err(e) => {
                had_errors = true;
                if !args.quiet {
                    eprintln!("Error parsing '{}': {}", id, e);
                }
            }
        }
    }

    if args.quiet {
        // In quiet mode, just return success/failure
        if had_errors {
            return Err(crate::core::error::IdtError::ValidationError(
                "One or more IDs failed to parse".into(),
            ));
        }
        return Ok(());
    }

    // Output results
    let mut stdout = io::stdout();

    if json_output {
        output_json(&mut stdout, &results, pretty)?;
    } else {
        output_human(&mut stdout, &results, no_color)?;
    }

    Ok(())
}

fn resolve_epoch(epoch: &Option<String>) -> Result<Option<u64>> {
    match epoch {
        None => Ok(None),
        Some(s) => {
            let ms = match s.to_lowercase().as_str() {
                "discord" => crate::ids::DISCORD_EPOCH,
                "twitter" => crate::ids::TWITTER_EPOCH,
                _ => s.parse::<u64>().map_err(|_| {
                    IdtError::InvalidArgument(format!(
                        "Invalid epoch '{}': use 'discord', 'twitter', or milliseconds since Unix epoch",
                        s
                    ))
                })?,
            };
            Ok(Some(ms))
        }
    }
}

fn collect_ids(args: &[String]) -> Result<Vec<String>> {
    if !args.is_empty() {
        return Ok(args.to_vec());
    }

    // Read from stdin
    let stdin = io::stdin();
    let mut ids = Vec::new();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            ids.push(trimmed.to_string());
        }
    }

    Ok(ids)
}

fn output_json(writer: &mut dyn Write, results: &[InspectionResult], pretty: bool) -> Result<()> {
    let output = if results.len() == 1 {
        serde_json::to_value(&results[0])?
    } else {
        serde_json::to_value(results)?
    };

    if pretty {
        writeln!(writer, "{}", serde_json::to_string_pretty(&output)?)?;
    } else {
        writeln!(writer, "{}", serde_json::to_string(&output)?)?;
    }

    Ok(())
}

fn output_human(
    writer: &mut dyn Write,
    results: &[InspectionResult],
    no_color: bool,
) -> Result<()> {
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }
        print_inspection(writer, result, no_color)?;
    }
    Ok(())
}

fn print_inspection(
    writer: &mut dyn Write,
    result: &InspectionResult,
    no_color: bool,
) -> Result<()> {
    // Compute label width based on longest label present
    let label_width = if let Some(ref ts) = result.timestamp {
        let local_label = format!("Local Time ({})", ts.local_timezone_abbr());
        local_label.len().max(12)
    } else {
        12
    };

    // Helper for coloring
    let label = |s: &str| -> String {
        if no_color {
            format!("{:width$}", s, width = label_width)
        } else {
            format!("{:width$}", s.dimmed(), width = label_width)
        }
    };

    let title = |s: &str| -> String {
        if no_color {
            s.to_uppercase()
        } else {
            s.to_uppercase().bold().to_string()
        }
    };

    let value = |s: &str| -> String {
        if no_color {
            s.to_string()
        } else {
            s.cyan().to_string()
        }
    };

    // Type and canonical ID
    writeln!(writer, "{}", title(&result.id_type))?;
    writeln!(writer, "  {}", value(&result.canonical))?;

    // Time info (if available)
    if result.timestamp.is_some() || result.version.is_some() {
        writeln!(writer)?;

        if let Some(ref iso) = result.timestamp_iso {
            writeln!(writer, "  {} {}", label("Time (UTC)"), iso)?;
        }

        if let Some(ref ts) = result.timestamp {
            let abbr = ts.local_timezone_abbr();
            if let Some(ref local_iso) = result.timestamp_local_iso {
                writeln!(
                    writer,
                    "  {} {}",
                    label(&format!("Local Time ({})", abbr)),
                    local_iso
                )?;
            }
        }

        if let Some(ref version) = result.version {
            writeln!(writer, "  {} {}", label("Version"), version)?;
        }

        if let Some(ref variant) = result.variant {
            writeln!(writer, "  {} {}", label("Variant"), variant)?;
        }

        if let Some(bits) = result.random_bits {
            writeln!(writer, "  {} {} bits", label("Random"), bits)?;
        }
    }

    // Encodings
    writeln!(writer)?;
    writeln!(writer, "  {} {}", label("Hex"), &result.encodings.hex)?;

    if !result.encodings.base64.is_empty() {
        writeln!(writer, "  {} {}", label("Base64"), &result.encodings.base64)?;
    }

    if let Some(ref int_val) = result.encodings.int {
        writeln!(writer, "  {} {}", label("Int"), int_val)?;
    }

    Ok(())
}

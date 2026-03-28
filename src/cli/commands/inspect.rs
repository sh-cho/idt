use crate::cli::app::{InspectArgs, OutputFormat};
use crate::cli::output::format_output;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdKind, InspectionResult, ParsedId};
use crate::ids::snowflake_id::SnowflakeLayout;
use colored::Colorize;
use std::io::{self, BufRead, Write};

pub fn execute(
    args: &InspectArgs,
    format: Option<OutputFormat>,
    pretty: bool,
    no_color: bool,
) -> Result<()> {
    let ids = collect_ids(&args.ids)?;

    if ids.is_empty() {
        return Err(IdtError::InvalidArgument(
            "No IDs provided. Pass IDs as arguments or via stdin.".to_string(),
        ));
    }

    let type_hint: Option<IdKind> = args.id_type;
    let has_snowflake_opts = args.preset.is_some() || args.epoch.is_some();
    let snowflake_layout = if has_snowflake_opts {
        Some(SnowflakeLayout::resolve(
            args.preset.as_deref(),
            args.epoch.as_deref(),
        )?)
    } else {
        None
    };

    let mut results = Vec::new();
    let mut failed_ids = Vec::new();

    for id in &ids {
        let parse_result: Result<Box<dyn ParsedId>> = if let Some(ref layout) = snowflake_layout {
            crate::ids::ParsedSnowflake::parse_with_layout(id, layout.clone())
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
                failed_ids.push(id.clone());
                if !args.quiet {
                    eprintln!("Error parsing '{}': {}", id, e);
                }
            }
        }
    }

    if args.quiet {
        // In quiet mode, just return success/failure
        if !failed_ids.is_empty() {
            return Err(crate::core::error::IdtError::ValidationError(format!(
                "Failed to parse {} of {} IDs: {}",
                failed_ids.len(),
                ids.len(),
                failed_ids.join(", ")
            )));
        }
        return Ok(());
    }

    // Output results
    let mut stdout = io::stdout();

    if let Some(fmt) = format {
        let output = if results.len() == 1 {
            format_output(&results[0], fmt, pretty)?
        } else {
            format_output(&results, fmt, pretty)?
        };
        writeln!(stdout, "{}", output)?;
    } else {
        output_human(&mut stdout, &results, no_color)?;
    }

    Ok(())
}

fn collect_ids(args: &[String]) -> Result<Vec<String>> {
    if !args.is_empty() {
        return Ok(args.to_vec());
    }

    // Don't block on stdin if it's a terminal (no piped input)
    if std::io::IsTerminal::is_terminal(&io::stdin()) {
        return Ok(Vec::new());
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

    // Structure (if available)
    if let Some(ref structure) = result.structure
        && !structure.is_empty()
    {
        writeln!(writer)?;
        let struct_title = if no_color {
            "  Structure".to_string()
        } else {
            format!("  {}", "Structure".dimmed())
        };
        writeln!(writer, "{}", struct_title)?;

        // Compute column widths
        let name_width = structure.iter().map(|s| s.name.len()).max().unwrap_or(0);
        let unit_label = |size: u32, unit: &crate::core::id::SizeUnit| -> &'static str {
            match unit {
                crate::core::id::SizeUnit::Bits => {
                    if size == 1 {
                        "bit"
                    } else {
                        "bits"
                    }
                }
                crate::core::id::SizeUnit::Digits => {
                    if size == 1 {
                        "digit"
                    } else {
                        "digits"
                    }
                }
                crate::core::id::SizeUnit::Chars => {
                    if size == 1 {
                        "char"
                    } else {
                        "chars"
                    }
                }
            }
        };

        let size_width = structure
            .iter()
            .map(|s| format!("{} {}", s.size, unit_label(s.size, &s.unit)).len())
            .max()
            .unwrap_or(0);
        let val_width = structure
            .iter()
            .map(|s| s.value.as_deref().unwrap_or("-").len())
            .max()
            .unwrap_or(0);

        for seg in structure {
            let unit_str = unit_label(seg.size, &seg.unit);
            let size_str = format!("{} {}", seg.size, unit_str);
            let val_str = seg.value.as_deref().unwrap_or("-");

            let name_formatted = if no_color {
                format!("{:<width$}", seg.name, width = name_width)
            } else {
                format!("{:<width$}", seg.name.cyan(), width = name_width)
            };

            let desc_formatted = if no_color {
                seg.description.clone()
            } else {
                seg.description.dimmed().to_string()
            };

            writeln!(
                writer,
                "    {} {:>size_w$}  {:<val_w$} {}",
                name_formatted,
                size_str,
                val_str,
                desc_formatted,
                size_w = size_width,
                val_w = val_width,
            )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::app::OutputFormat;

    fn make_args(ids: Vec<&str>) -> InspectArgs {
        InspectArgs {
            ids: ids.into_iter().map(String::from).collect(),
            id_type: None,
            epoch: None,
            preset: None,
            quiet: false,
        }
    }

    #[test]
    fn test_empty_input() {
        let args = make_args(vec![]);
        let result = execute(&args, None, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_inspect_uuid_v4() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_ulid() {
        let args = make_args(vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV"]);
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_json_output() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, Some(OutputFormat::Json), false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_json_pretty() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, Some(OutputFormat::Json), true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_multiple_ids() {
        let args = make_args(vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        ]);
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_multiple_ids_json() {
        let args = make_args(vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        ]);
        let result = execute(&args, Some(OutputFormat::Json), false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_with_type_hint() {
        let args = InspectArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: Some(IdKind::Uuid),
            epoch: None,
            preset: None,
            quiet: false,
        };
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_quiet_mode() {
        let args = InspectArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: None,
            epoch: None,
            preset: None,
            quiet: true,
        };
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_quiet_mode_invalid() {
        let args = InspectArgs {
            ids: vec!["invalid-id-string".to_string()],
            id_type: None,
            epoch: None,
            preset: None,
            quiet: true,
        };
        let result = execute(&args, None, false, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_inspect_invalid_id_non_quiet() {
        let args = make_args(vec!["invalid-id-string"]);
        // Should succeed (prints error to stderr but doesn't fail unless quiet)
        let _result = execute(&args, None, false, true);
    }

    #[test]
    fn test_inspect_yaml_output() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, Some(OutputFormat::Yaml), false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_with_snowflake_preset() {
        let args = InspectArgs {
            ids: vec!["1234567890123456789".to_string()],
            id_type: None,
            epoch: None,
            preset: Some("twitter".to_string()),
            quiet: false,
        };
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }
}

use crate::cli::app::ConvertArgs;
use crate::core::EncodingFormat;
use crate::core::error::{IdtError, Result};
use crate::core::id::IdKind;
use std::io::{self, BufRead, Write};

pub fn execute(args: &ConvertArgs, json_output: bool, _pretty: bool) -> Result<()> {
    let ids = collect_ids(&args.ids)?;

    if ids.is_empty() {
        return Err(IdtError::InvalidArgument(
            "No IDs provided. Pass IDs as arguments or via stdin.".to_string(),
        ));
    }

    let type_hint: Option<IdKind> = args.id_type.as_ref().map(|t| t.parse()).transpose()?;

    let format: EncodingFormat = args
        .format
        .as_ref()
        .map(|f| f.parse())
        .transpose()?
        .unwrap_or(EncodingFormat::Canonical);

    let mut results = Vec::new();

    for id in &ids {
        match crate::ids::parse_id(id, type_hint) {
            Ok(parsed) => {
                let mut converted = parsed.encode(format);

                // Apply case transformation
                if args.uppercase {
                    converted = converted.to_uppercase();
                } else if args.lowercase {
                    converted = converted.to_lowercase();
                }

                results.push(ConvertResult {
                    input: id.clone(),
                    output: converted,
                    format: format.to_string(),
                });
            }
            Err(e) => {
                eprintln!("Error converting '{}': {}", id, e);
            }
        }
    }

    // Output
    let mut stdout = io::stdout();

    if json_output {
        output_json(&mut stdout, &results)?;
    } else {
        output_plain(&mut stdout, &results)?;
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct ConvertResult {
    input: String,
    output: String,
    format: String,
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

fn output_json(writer: &mut dyn Write, results: &[ConvertResult]) -> Result<()> {
    if results.len() == 1 {
        writeln!(writer, "{}", serde_json::to_string(&results[0].output)?)?;
    } else {
        let outputs: Vec<&str> = results.iter().map(|r| r.output.as_str()).collect();
        writeln!(writer, "{}", serde_json::to_string(&outputs)?)?;
    }
    Ok(())
}

fn output_plain(writer: &mut dyn Write, results: &[ConvertResult]) -> Result<()> {
    for result in results {
        writeln!(writer, "{}", result.output)?;
    }
    Ok(())
}

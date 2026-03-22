use crate::cli::app::{ConvertArgs, OutputFormat};
use crate::cli::output::format_output;
use crate::core::EncodingFormat;
use crate::core::error::{IdtError, Result};
use crate::core::id::IdKind;
use std::io::{self, BufRead, Write};

pub fn execute(
    args: &ConvertArgs,
    output_format: Option<OutputFormat>,
    pretty: bool,
) -> Result<()> {
    let ids = collect_ids(&args.ids)?;

    if ids.is_empty() {
        return Err(IdtError::InvalidArgument(
            "No IDs provided. Pass IDs as arguments or via stdin.".to_string(),
        ));
    }

    let type_hint: Option<IdKind> = args.id_type;

    let encoding: EncodingFormat = args
        .format
        .as_ref()
        .map(|f| f.parse())
        .transpose()?
        .unwrap_or(EncodingFormat::Canonical);

    let mut results = Vec::new();

    for id in &ids {
        match crate::ids::parse_id(id, type_hint) {
            Ok(parsed) => {
                let mut converted = parsed.encode(encoding);

                // Apply case transformation
                if args.uppercase {
                    converted = converted.to_uppercase();
                } else if args.lowercase {
                    converted = converted.to_lowercase();
                }

                results.push(ConvertResult {
                    input: id.clone(),
                    output: converted,
                    format: encoding.to_string(),
                });
            }
            Err(e) => {
                eprintln!("Error converting '{}': {}", id, e);
            }
        }
    }

    // Output
    let mut stdout = io::stdout();

    if let Some(fmt) = output_format {
        let output = if results.len() == 1 {
            format_output(&results[0].output, fmt, pretty)?
        } else {
            let outputs: Vec<&str> = results.iter().map(|r| r.output.as_str()).collect();
            format_output(&outputs, fmt, pretty)?
        };
        writeln!(stdout, "{}", output)?;
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

fn output_plain(writer: &mut dyn Write, results: &[ConvertResult]) -> Result<()> {
    for result in results {
        writeln!(writer, "{}", result.output)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::app::OutputFormat;

    fn make_args(ids: Vec<&str>) -> ConvertArgs {
        ConvertArgs {
            ids: ids.into_iter().map(String::from).collect(),
            id_type: None,
            format: None,
            to: None,
            uppercase: false,
            lowercase: false,
        }
    }

    #[test]
    fn test_empty_input() {
        let args = make_args(vec![]);
        let result = execute(&args, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_uuid_default() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_uuid_to_hex() {
        let args = ConvertArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: None,
            format: Some("hex".to_string()),
            to: None,
            uppercase: false,
            lowercase: false,
        };
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_uuid_to_base64() {
        let args = ConvertArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: None,
            format: Some("base64".to_string()),
            to: None,
            uppercase: false,
            lowercase: false,
        };
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_uppercase() {
        let args = ConvertArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: None,
            format: Some("hex".to_string()),
            to: None,
            uppercase: true,
            lowercase: false,
        };
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_lowercase() {
        let args = ConvertArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: None,
            format: Some("hex".to_string()),
            to: None,
            uppercase: false,
            lowercase: true,
        };
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_json_output() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, Some(OutputFormat::Json), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_multiple_ids_json() {
        let args = make_args(vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        ]);
        let result = execute(&args, Some(OutputFormat::Json), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_with_type_hint() {
        let args = ConvertArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: Some(IdKind::Uuid),
            format: Some("base58".to_string()),
            to: None,
            uppercase: false,
            lowercase: false,
        };
        let result = execute(&args, None, false);
        assert!(result.is_ok());
    }
}

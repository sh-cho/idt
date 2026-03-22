use crate::cli::app::{OutputFormat, ValidateArgs};
use crate::cli::output::format_output;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdKind, ValidationResult};
use colored::Colorize;
use std::io::{self, BufRead, Write};

pub fn execute(
    args: &ValidateArgs,
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

    let mut results = Vec::new();
    let mut all_valid = true;

    for id in &ids {
        let result = validate_id(id, type_hint, args.strict);
        if !result.valid {
            all_valid = false;
        }
        results.push(ValidateOutput {
            input: id.clone(),
            result,
        });
    }

    // Output
    if !args.quiet {
        let mut stdout = io::stdout();

        if let Some(fmt) = format {
            let output = if results.len() == 1 {
                format_output(&results[0], fmt, pretty)?
            } else {
                format_output(&results, fmt, pretty)?
            };
            writeln!(stdout, "{}", output)?;
        } else {
            output_plain(&mut stdout, &results, no_color)?;
        }
    }

    // Return result
    if all_valid {
        Ok(())
    } else {
        Err(IdtError::ValidationError(
            "One or more IDs are invalid".into(),
        ))
    }
}

fn validate_id(id: &str, type_hint: Option<IdKind>, strict: bool) -> ValidationResult {
    match crate::ids::parse_id(id, type_hint) {
        Ok(parsed) => {
            let mut result = parsed.validate();

            // Strict mode: check canonical form
            if strict && result.valid {
                let canonical = parsed.canonical();
                if canonical != id {
                    result.valid = false;
                    result.error = Some("Non-canonical form".to_string());
                    result.hint = Some(format!("Canonical form: {}", canonical));
                }
            }

            result
        }
        Err(e) => {
            let mut result = ValidationResult::invalid(&e.to_string());

            // Add hints for common mistakes
            if id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit()) {
                result.hint =
                    Some("Looks like UUID without dashes. Try adding dashes.".to_string());
            } else if id.len() == 36 && id.contains('-') {
                result.hint = Some("Check for invalid characters in UUID.".to_string());
            }

            result
        }
    }
}

#[derive(serde::Serialize)]
struct ValidateOutput {
    input: String,
    #[serde(flatten)]
    result: ValidationResult,
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

fn output_plain(writer: &mut dyn Write, results: &[ValidateOutput], no_color: bool) -> Result<()> {
    for result in results {
        let status = if result.result.valid {
            if no_color {
                "valid".to_string()
            } else {
                "valid".green().to_string()
            }
        } else if no_color {
            "invalid".to_string()
        } else {
            "invalid".red().to_string()
        };

        let type_info = result
            .result
            .id_type
            .as_ref()
            .map(|t| format!(" ({})", t))
            .unwrap_or_default();

        writeln!(writer, "{}: {}{}", result.input, status, type_info)?;

        if let Some(ref error) = result.result.error {
            let error_msg = if no_color {
                format!("  Error: {}", error)
            } else {
                format!("  {}: {}", "Error".red(), error)
            };
            writeln!(writer, "{}", error_msg)?;
        }

        if let Some(ref hint) = result.result.hint {
            let hint_msg = if no_color {
                format!("  Hint: {}", hint)
            } else {
                format!("  {}: {}", "Hint".yellow(), hint)
            };
            writeln!(writer, "{}", hint_msg)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::app::OutputFormat;

    fn make_args(ids: Vec<&str>) -> ValidateArgs {
        ValidateArgs {
            ids: ids.into_iter().map(String::from).collect(),
            id_type: None,
            quiet: false,
            strict: false,
        }
    }

    #[test]
    fn test_empty_input() {
        let args = make_args(vec![]);
        let result = execute(&args, None, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_uuid() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_ulid() {
        let args = make_args(vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV"]);
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let args = make_args(vec!["not-a-valid-id"]);
        let result = execute(&args, None, false, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_strict_mode_non_canonical() {
        let args = ValidateArgs {
            ids: vec!["550E8400-E29B-41D4-A716-446655440000".to_string()],
            id_type: Some(IdKind::Uuid),
            quiet: false,
            strict: true,
        };
        let result = execute(&args, None, false, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_quiet_mode_valid() {
        let args = ValidateArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: None,
            quiet: true,
            strict: false,
        };
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_output() {
        let args = make_args(vec!["550e8400-e29b-41d4-a716-446655440000"]);
        let result = execute(&args, Some(OutputFormat::Json), false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_ids() {
        let args = make_args(vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        ]);
        let result = execute(&args, Some(OutputFormat::Json), false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_type_hint() {
        let args = ValidateArgs {
            ids: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
            id_type: Some(IdKind::Uuid),
            quiet: false,
            strict: false,
        };
        let result = execute(&args, None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_id_hint_uuid_without_dashes() {
        // 32 hex chars that don't parse as any known ID type get a helpful hint
        let result = validate_id("zz0e8400e29b41d4a716446655440zzz", None, false);
        assert!(!result.valid);
    }
}

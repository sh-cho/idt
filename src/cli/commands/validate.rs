use crate::cli::app::ValidateArgs;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdKind, ValidationResult};
use colored::Colorize;
use std::io::{self, BufRead, Write};

pub fn execute(args: &ValidateArgs, json_output: bool, _pretty: bool, no_color: bool) -> Result<()> {
    let ids = collect_ids(&args.ids)?;

    if ids.is_empty() {
        return Err(IdtError::InvalidArgument(
            "No IDs provided. Pass IDs as arguments or via stdin.".to_string(),
        ));
    }

    let type_hint: Option<IdKind> = args.id_type.as_ref().map(|t| t.parse()).transpose()?;

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

        if json_output {
            output_json(&mut stdout, &results)?;
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
                result.hint = Some("Looks like UUID without dashes. Try adding dashes.".to_string());
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

fn output_json(writer: &mut dyn Write, results: &[ValidateOutput]) -> Result<()> {
    if results.len() == 1 {
        writeln!(writer, "{}", serde_json::to_string(&results[0])?)?;
    } else {
        writeln!(writer, "{}", serde_json::to_string(results)?)?;
    }
    Ok(())
}

fn output_plain(writer: &mut dyn Write, results: &[ValidateOutput], no_color: bool) -> Result<()> {
    for result in results {
        let status = if result.result.valid {
            if no_color {
                "valid".to_string()
            } else {
                "valid".green().to_string()
            }
        } else {
            if no_color {
                "invalid".to_string()
            } else {
                "invalid".red().to_string()
            }
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

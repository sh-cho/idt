use crate::cli::app::{SortArgs, UnsortablePolicy};
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdKind, ParsedId, Timestamp};
use crate::ids::snowflake_id::SnowflakeLayout;
use std::io::{self, BufRead, Write};

struct SortEntry {
    input: String,
    id_type: String,
    timestamp: Option<Timestamp>,
}

pub fn execute(args: &SortArgs, json_output: bool, pretty: bool, _no_color: bool) -> Result<()> {
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

    let mut sortable: Vec<SortEntry> = Vec::new();
    let mut unsortable: Vec<SortEntry> = Vec::new();

    for id in &ids {
        let parse_result: Result<Box<dyn ParsedId>> = if let Some(ref layout) = snowflake_layout {
            crate::ids::ParsedSnowflake::parse_with_layout(id, layout.clone())
                .map(|s| Box::new(s) as Box<dyn ParsedId>)
        } else {
            crate::ids::parse_id(id, type_hint)
        };

        match parse_result {
            Ok(parsed) => {
                let ts = parsed.timestamp();
                let inspection = parsed.inspect();
                if ts.is_some() {
                    sortable.push(SortEntry {
                        input: id.clone(),
                        id_type: inspection.id_type,
                        timestamp: ts,
                    });
                } else {
                    match args.on_unsortable {
                        UnsortablePolicy::Error => {
                            return Err(IdtError::InvalidArgument(format!(
                                "ID '{}' ({}) has no embedded timestamp",
                                id, inspection.id_type
                            )));
                        }
                        UnsortablePolicy::Skip => {
                            eprintln!(
                                "Warning: skipping '{}' ({}) — no embedded timestamp",
                                id, inspection.id_type
                            );
                        }
                        UnsortablePolicy::End => {
                            unsortable.push(SortEntry {
                                input: id.clone(),
                                id_type: inspection.id_type,
                                timestamp: None,
                            });
                        }
                    }
                }
            }
            Err(e) => match args.on_unsortable {
                UnsortablePolicy::Error => {
                    return Err(IdtError::InvalidArgument(format!(
                        "Failed to parse '{}': {}",
                        id, e
                    )));
                }
                UnsortablePolicy::Skip => {
                    eprintln!("Warning: skipping '{}' — failed to parse: {}", id, e);
                }
                UnsortablePolicy::End => {
                    unsortable.push(SortEntry {
                        input: id.clone(),
                        id_type: "unknown".to_string(),
                        timestamp: None,
                    });
                }
            },
        }
    }

    // Stable sort by timestamp millis
    sortable.sort_by_key(|e| e.timestamp.map(|t| t.millis).unwrap_or(0));

    if args.reverse {
        sortable.reverse();
    }

    let mut stdout = io::stdout();

    if json_output {
        output_json(&mut stdout, &sortable, &unsortable, pretty)?;
    } else {
        output_plain(&mut stdout, &sortable, &unsortable, args.show_time)?;
    }

    Ok(())
}

fn collect_ids(args: &[String]) -> Result<Vec<String>> {
    if !args.is_empty() {
        return Ok(args.to_vec());
    }

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

fn output_plain(
    writer: &mut dyn Write,
    sortable: &[SortEntry],
    unsortable: &[SortEntry],
    show_time: bool,
) -> Result<()> {
    for entry in sortable {
        if show_time {
            if let Some(ref ts) = entry.timestamp {
                writeln!(writer, "{}  {}", ts.to_iso8601(), entry.input)?;
            } else {
                writeln!(writer, "{}", entry.input)?;
            }
        } else {
            writeln!(writer, "{}", entry.input)?;
        }
    }

    for entry in unsortable {
        if show_time {
            writeln!(writer, "                         {}", entry.input)?;
        } else {
            writeln!(writer, "{}", entry.input)?;
        }
    }

    Ok(())
}

fn output_json(
    writer: &mut dyn Write,
    sortable: &[SortEntry],
    unsortable: &[SortEntry],
    pretty: bool,
) -> Result<()> {
    let sorted_items: Vec<serde_json::Value> = sortable
        .iter()
        .map(|e| {
            let mut obj = serde_json::json!({
                "id": e.input,
                "id_type": e.id_type,
            });
            if let Some(ref ts) = e.timestamp {
                obj["timestamp_ms"] = serde_json::json!(ts.millis);
                obj["timestamp_iso"] = serde_json::json!(ts.to_iso8601());
            }
            obj
        })
        .collect();

    let unsortable_items: Vec<serde_json::Value> = unsortable
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.input,
                "id_type": e.id_type,
            })
        })
        .collect();

    let output = serde_json::json!({
        "sorted": sorted_items,
        "unsortable": unsortable_items,
        "count": sortable.len() + unsortable.len(),
    });

    if pretty {
        writeln!(writer, "{}", serde_json::to_string_pretty(&output)?)?;
    } else {
        writeln!(writer, "{}", serde_json::to_string(&output)?)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::app::UnsortablePolicy;

    fn make_args(ids: Vec<&str>) -> SortArgs {
        SortArgs {
            ids: ids.into_iter().map(String::from).collect(),
            id_type: None,
            reverse: false,
            show_time: false,
            epoch: None,
            preset: None,
            on_unsortable: UnsortablePolicy::Skip,
        }
    }

    #[test]
    fn test_empty_input() {
        let args = make_args(vec![]);
        let result = execute(&args, false, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_ulid() {
        // Generate a ULID-like ID to sort (single should just return it)
        let args = make_args(vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV"]);
        let result = execute(&args, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_json_output() {
        let args = make_args(vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV"]);
        let result = execute(&args, true, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsortable_error_policy() {
        let args = SortArgs {
            ids: vec!["not-a-real-id-format-xyz".to_string()],
            id_type: None,
            reverse: false,
            show_time: false,
            epoch: None,
            preset: None,
            on_unsortable: UnsortablePolicy::Error,
        };
        let result = execute(&args, false, false, false);
        assert!(result.is_err());
    }
}

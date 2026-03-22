use crate::cli::app::{OutputFormat, SortArgs, UnsortablePolicy};
use crate::cli::output::format_output;
use crate::core::error::{IdtError, Result};
use crate::core::id::{IdKind, ParsedId, Timestamp};
use crate::ids::snowflake_id::SnowflakeLayout;
use std::io::{self, BufRead, Write};

struct SortEntry {
    input: String,
    id_type: String,
    timestamp: Option<Timestamp>,
}

pub fn execute(
    args: &SortArgs,
    format: Option<OutputFormat>,
    pretty: bool,
    _no_color: bool,
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

    if let Some(fmt) = format {
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

        let output_val = serde_json::json!({
            "sorted": sorted_items,
            "unsortable": unsortable_items,
            "count": sortable.len() + unsortable.len(),
        });

        let output = format_output(&output_val, fmt, pretty)?;
        writeln!(stdout, "{}", output)?;
    } else {
        output_plain(&mut stdout, &sortable, &unsortable, args.show_time)?;
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
        let result = execute(&args, None, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_ulid() {
        // Generate a ULID-like ID to sort (single should just return it)
        let args = make_args(vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV"]);
        let result = execute(&args, None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_json_output() {
        let args = make_args(vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV"]);
        let result = execute(&args, Some(OutputFormat::Json), false, false);
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
        let result = execute(&args, None, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_multiple_ulids() {
        let args = make_args(vec![
            "01BX5ZZKBKACTAV9WEVGEMMVRY",
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        ]);
        let result = execute(&args, None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_reverse() {
        let mut args = make_args(vec![
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "01BX5ZZKBKACTAV9WEVGEMMVRY",
        ]);
        args.reverse = true;
        let result = execute(&args, None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_show_time() {
        let mut args = make_args(vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV"]);
        args.show_time = true;
        let result = execute(&args, None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsortable_end_policy() {
        let args = SortArgs {
            ids: vec!["not-a-real-id-format-xyz".to_string()],
            id_type: None,
            reverse: false,
            show_time: false,
            epoch: None,
            preset: None,
            on_unsortable: UnsortablePolicy::End,
        };
        let result = execute(&args, None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsortable_skip_policy() {
        let args = SortArgs {
            ids: vec!["not-a-real-id-format-xyz".to_string()],
            id_type: None,
            reverse: false,
            show_time: false,
            epoch: None,
            preset: None,
            on_unsortable: UnsortablePolicy::Skip,
        };
        let result = execute(&args, None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_plain_with_time() {
        let entries = vec![SortEntry {
            input: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            id_type: "ulid".to_string(),
            timestamp: Some(Timestamp::new(1469918176385)),
        }];
        let mut buf = Vec::new();
        output_plain(&mut buf, &entries, &[], true).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("01ARZ3NDEKTSV4RRFFQ69G5FAV"));
    }

    #[test]
    fn test_output_plain_unsortable_with_time() {
        let unsortable = vec![SortEntry {
            input: "some-id".to_string(),
            id_type: "unknown".to_string(),
            timestamp: None,
        }];
        let mut buf = Vec::new();
        output_plain(&mut buf, &[], &unsortable, true).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("some-id"));
    }

    #[test]
    fn test_output_plain_without_time() {
        let entries = vec![SortEntry {
            input: "test-id".to_string(),
            id_type: "ulid".to_string(),
            timestamp: Some(Timestamp::new(1000)),
        }];
        let mut buf = Vec::new();
        output_plain(&mut buf, &entries, &[], false).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "test-id\n");
    }

    #[test]
    fn test_collect_ids_from_args() {
        let args = vec!["id1".to_string(), "id2".to_string()];
        let ids = collect_ids(&args).unwrap();
        assert_eq!(ids, vec!["id1", "id2"]);
    }

    #[test]
    fn test_sort_json_with_unsortable_end() {
        let args = SortArgs {
            ids: vec![
                "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                "not-a-real-id".to_string(),
            ],
            id_type: None,
            reverse: false,
            show_time: false,
            epoch: None,
            preset: None,
            on_unsortable: UnsortablePolicy::End,
        };
        let result = execute(&args, Some(OutputFormat::Json), true, false);
        assert!(result.is_ok());
    }
}

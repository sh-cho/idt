use crate::cli::app::CompareArgs;
use crate::core::error::Result;
use crate::core::id::IdKind;
use colored::Colorize;
use std::cmp::Ordering;
use std::io::{self, Write};

pub fn execute(args: &CompareArgs, json_output: bool, _pretty: bool, no_color: bool) -> Result<()> {
    let type_hint: Option<IdKind> = args.id_type.as_ref().map(|t| t.parse()).transpose()?;

    let parsed1 = crate::ids::parse_id(&args.id1, type_hint)?;
    let parsed2 = crate::ids::parse_id(&args.id2, type_hint)?;

    let bytes1 = parsed1.as_bytes();
    let bytes2 = parsed2.as_bytes();

    let ts1 = parsed1.timestamp();
    let ts2 = parsed2.timestamp();

    // Binary comparison
    let binary_order = bytes1.cmp(&bytes2);

    // Lexicographic comparison
    let canonical1 = parsed1.canonical();
    let canonical2 = parsed2.canonical();
    let lexicographic_order = canonical1.cmp(&canonical2);

    // Time comparison (if both have timestamps)
    let time_order = match (&ts1, &ts2) {
        (Some(t1), Some(t2)) => Some(t1.millis.cmp(&t2.millis)),
        _ => None,
    };

    let time_diff = match (&ts1, &ts2) {
        (Some(t1), Some(t2)) => Some(t1.millis as i128 - t2.millis as i128),
        _ => None,
    };

    let result = CompareResult {
        id1: args.id1.clone(),
        id2: args.id2.clone(),
        type1: parsed1.kind().to_string(),
        type2: parsed2.kind().to_string(),
        binary_order: ordering_to_string(binary_order),
        lexicographic_order: ordering_to_string(lexicographic_order),
        chronological_order: time_order.map(ordering_to_string),
        time_diff_ms: time_diff,
        timestamp1: ts1.map(|t| t.millis),
        timestamp2: ts2.map(|t| t.millis),
    };

    let mut stdout = io::stdout();

    if json_output {
        writeln!(stdout, "{}", serde_json::to_string_pretty(&result)?)?;
    } else {
        print_human(&mut stdout, &result, no_color)?;
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct CompareResult {
    id1: String,
    id2: String,
    type1: String,
    type2: String,
    binary_order: String,
    lexicographic_order: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chronological_order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_diff_ms: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp1: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp2: Option<u64>,
}

fn ordering_to_string(ord: Ordering) -> String {
    match ord {
        Ordering::Less => "less".to_string(),
        Ordering::Equal => "equal".to_string(),
        Ordering::Greater => "greater".to_string(),
    }
}

fn print_human(writer: &mut dyn Write, result: &CompareResult, no_color: bool) -> Result<()> {
    let label = |s: &str| -> String {
        if no_color {
            format!("{:20}", s)
        } else {
            format!("{:20}", s.dimmed())
        }
    };

    writeln!(writer, "Comparing IDs:")?;
    writeln!(writer, "  {} {}", label("ID 1:"), result.id1)?;
    writeln!(writer, "  {} {}", label("ID 2:"), result.id2)?;
    writeln!(writer)?;

    if result.type1 != result.type2 {
        let warning = if no_color {
            "Warning: Different types!".to_string()
        } else {
            "Warning: Different types!".yellow().to_string()
        };
        writeln!(
            writer,
            "  {} ({} vs {})",
            warning, result.type1, result.type2
        )?;
        writeln!(writer)?;
    }

    writeln!(writer, "Comparison Results:")?;

    // Binary comparison
    let binary_symbol = match result.binary_order.as_str() {
        "less" => "<",
        "greater" => ">",
        _ => "=",
    };
    writeln!(writer, "  {} ID1 {} ID2", label("Binary:"), binary_symbol)?;

    // Lexicographic comparison
    let lex_symbol = match result.lexicographic_order.as_str() {
        "less" => "<",
        "greater" => ">",
        _ => "=",
    };
    writeln!(
        writer,
        "  {} ID1 {} ID2",
        label("Lexicographic:"),
        lex_symbol
    )?;

    // Chronological comparison (if available)
    if let Some(ref chrono) = result.chronological_order {
        let chrono_desc = match chrono.as_str() {
            "less" => "ID1 is older (created before ID2)",
            "greater" => "ID1 is newer (created after ID2)",
            _ => "Same time",
        };
        writeln!(writer, "  {} {}", label("Chronological:"), chrono_desc)?;

        if let Some(diff) = result.time_diff_ms {
            let diff_abs = diff.abs();
            let diff_str = if diff_abs < 1000 {
                format!("{} ms", diff_abs)
            } else if diff_abs < 60000 {
                format!("{:.2} seconds", diff_abs as f64 / 1000.0)
            } else if diff_abs < 3600000 {
                format!("{:.2} minutes", diff_abs as f64 / 60000.0)
            } else if diff_abs < 86400000 {
                format!("{:.2} hours", diff_abs as f64 / 3600000.0)
            } else {
                format!("{:.2} days", diff_abs as f64 / 86400000.0)
            };
            writeln!(writer, "  {} {}", label("Time difference:"), diff_str)?;
        }
    }

    Ok(())
}

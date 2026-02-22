use crate::cli::app::InfoArgs;
use crate::core::error::Result;
use crate::core::id::IdKind;
use colored::Colorize;
use std::io::{self, Write};

pub fn execute(args: &InfoArgs, json_output: bool, _pretty: bool, no_color: bool) -> Result<()> {
    let mut stdout = io::stdout();

    if let Some(kind) = args.id_type {
        // Show detailed info about specific type
        show_type_detail(&mut stdout, kind, json_output, no_color)?;
    } else {
        // List all types
        list_all_types(&mut stdout, json_output, no_color)?;
    }

    Ok(())
}

fn list_all_types(writer: &mut dyn Write, json_output: bool, no_color: bool) -> Result<()> {
    if json_output {
        let types: Vec<TypeInfo> = IdKind::all()
            .iter()
            .map(|k| TypeInfo {
                name: k.name().to_string(),
                description: k.description().to_string(),
                has_timestamp: k.has_timestamp(),
                is_sortable: k.is_sortable(),
                bit_length: k.bit_length(),
            })
            .collect();

        writeln!(writer, "{}", serde_json::to_string_pretty(&types)?)?;
    } else {
        let title = if no_color {
            "Supported ID Types".to_string()
        } else {
            "Supported ID Types".bold().to_string()
        };

        writeln!(writer, "{}", title)?;
        writeln!(writer, "{}", "=".repeat(60))?;
        writeln!(writer)?;

        // Group by category
        writeln!(writer, "{}:", format_category("UUID Family", no_color))?;
        for kind in &[
            IdKind::UuidV1,
            IdKind::UuidV3,
            IdKind::UuidV4,
            IdKind::UuidV5,
            IdKind::UuidV6,
            IdKind::UuidV7,
            IdKind::UuidNil,
            IdKind::UuidMax,
        ] {
            print_type_summary(writer, *kind, no_color)?;
        }

        writeln!(writer)?;
        writeln!(
            writer,
            "{}:",
            format_category("Modern Sortable IDs", no_color)
        )?;
        for kind in &[IdKind::Ulid, IdKind::Snowflake] {
            print_type_summary(writer, *kind, no_color)?;
        }

        writeln!(writer)?;
        writeln!(writer, "{}:", format_category("Sortable IDs", no_color))?;
        for kind in &[IdKind::Ksuid, IdKind::Xid, IdKind::Tsid, IdKind::TypeId] {
            print_type_summary(writer, *kind, no_color)?;
        }

        writeln!(writer)?;
        writeln!(
            writer,
            "{}:",
            format_category("Collision-Resistant IDs", no_color)
        )?;
        for kind in &[IdKind::Cuid, IdKind::Cuid2] {
            print_type_summary(writer, *kind, no_color)?;
        }

        writeln!(writer)?;
        writeln!(writer, "{}:", format_category("Database IDs", no_color))?;
        print_type_summary(writer, IdKind::ObjectId, no_color)?;

        writeln!(writer)?;
        writeln!(writer, "{}:", format_category("Compact IDs", no_color))?;
        print_type_summary(writer, IdKind::NanoId, no_color)?;

        writeln!(writer)?;
        writeln!(writer, "Use 'idt info <TYPE>' for detailed information.")?;
    }

    Ok(())
}

fn show_type_detail(
    writer: &mut dyn Write,
    kind: IdKind,
    json_output: bool,
    no_color: bool,
) -> Result<()> {
    let info = TypeDetail {
        name: kind.name().to_string(),
        description: kind.description().to_string(),
        has_timestamp: kind.has_timestamp(),
        is_sortable: kind.is_sortable(),
        bit_length: kind.bit_length(),
        example: generate_example(kind)?,
        spec_url: get_spec_url(kind),
        notes: get_notes(kind),
    };

    if json_output {
        writeln!(writer, "{}", serde_json::to_string_pretty(&info)?)?;
    } else {
        print_type_detail(writer, &info, no_color)?;
    }

    Ok(())
}

fn print_type_summary(writer: &mut dyn Write, kind: IdKind, no_color: bool) -> Result<()> {
    let name = if no_color {
        format!("{:12}", kind.name())
    } else {
        format!("{:12}", kind.name().cyan())
    };

    let flags = format!(
        "[{}{}]",
        if kind.has_timestamp() { "T" } else { "-" },
        if kind.is_sortable() { "S" } else { "-" }
    );

    let flags_colored = if no_color {
        flags
    } else {
        flags.dimmed().to_string()
    };

    writeln!(
        writer,
        "  {} {} {}",
        name,
        flags_colored,
        kind.description()
    )?;
    Ok(())
}

fn print_type_detail(writer: &mut dyn Write, info: &TypeDetail, no_color: bool) -> Result<()> {
    let title = if no_color {
        info.name.to_uppercase()
    } else {
        info.name.to_uppercase().bold().to_string()
    };

    writeln!(writer, "{}", title)?;
    writeln!(writer, "{}", "=".repeat(60))?;
    writeln!(writer)?;

    writeln!(writer, "{}", info.description)?;
    writeln!(writer)?;

    let label = |s: &str| -> String {
        if no_color {
            format!("{:16}", s)
        } else {
            format!("{:16}", s.dimmed())
        }
    };

    let yes_no = |b: bool| -> String {
        if b {
            if no_color {
                "Yes".to_string()
            } else {
                "Yes".green().to_string()
            }
        } else if no_color {
            "No".to_string()
        } else {
            "No".red().to_string()
        }
    };

    writeln!(
        writer,
        "{} {}",
        label("Has Timestamp:"),
        yes_no(info.has_timestamp)
    )?;
    writeln!(
        writer,
        "{} {}",
        label("Sortable:"),
        yes_no(info.is_sortable)
    )?;
    writeln!(writer, "{} {} bits", label("Bit Length:"), info.bit_length)?;
    writeln!(writer)?;

    writeln!(writer, "{} {}", label("Example:"), info.example)?;
    writeln!(writer)?;

    if let Some(ref url) = info.spec_url {
        writeln!(writer, "{} {}", label("Specification:"), url)?;
    }

    if !info.notes.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "{}:", label("Notes"))?;
        for note in &info.notes {
            writeln!(writer, "  - {}", note)?;
        }
    }

    Ok(())
}

fn format_category(name: &str, no_color: bool) -> String {
    if no_color {
        name.to_string()
    } else {
        name.bold().underline().to_string()
    }
}

fn generate_example(kind: IdKind) -> Result<String> {
    let generator = crate::ids::create_generator(kind)?;
    generator.generate()
}

fn get_spec_url(kind: IdKind) -> Option<String> {
    match kind {
        IdKind::Uuid | IdKind::UuidV1 | IdKind::UuidV3 | IdKind::UuidV4 | IdKind::UuidV5 => {
            Some("https://datatracker.ietf.org/doc/html/rfc4122".to_string())
        }
        IdKind::UuidV6 | IdKind::UuidV7 => {
            Some("https://datatracker.ietf.org/doc/html/rfc9562".to_string())
        }
        IdKind::Ulid => Some("https://github.com/ulid/spec".to_string()),
        IdKind::Snowflake => Some("https://en.wikipedia.org/wiki/Snowflake_ID".to_string()),
        IdKind::NanoId => Some("https://github.com/ai/nanoid".to_string()),
        IdKind::Ksuid => Some("https://github.com/segmentio/ksuid".to_string()),
        IdKind::ObjectId => {
            Some("https://www.mongodb.com/docs/manual/reference/method/ObjectId/".to_string())
        }
        IdKind::TypeId => Some("https://github.com/jetify-com/typeid".to_string()),
        IdKind::Xid => Some("https://github.com/rs/xid".to_string()),
        IdKind::Cuid => Some("https://github.com/paralleldrive/cuid".to_string()),
        IdKind::Cuid2 => Some("https://github.com/paralleldrive/cuid2".to_string()),
        IdKind::Tsid => Some("https://github.com/f4b6a3/tsid-creator".to_string()),
        _ => None,
    }
}

fn get_notes(kind: IdKind) -> Vec<String> {
    match kind {
        IdKind::UuidV4 => vec![
            "Most commonly used UUID version".to_string(),
            "122 bits of randomness".to_string(),
            "Collision probability extremely low".to_string(),
        ],
        IdKind::UuidV7 => vec![
            "Recommended for new applications needing sortable UUIDs".to_string(),
            "Unix timestamp in milliseconds".to_string(),
            "Compatible with UUID infrastructure".to_string(),
        ],
        IdKind::Ulid => vec![
            "Case-insensitive (Crockford Base32)".to_string(),
            "Monotonic within same millisecond".to_string(),
            "Compatible with UUID (128-bit)".to_string(),
        ],
        IdKind::Snowflake => vec![
            "Originally designed by Twitter".to_string(),
            "Requires coordination (machine/datacenter IDs)".to_string(),
            "Epoch can be customized".to_string(),
        ],
        IdKind::NanoId => vec![
            "Customizable alphabet and length".to_string(),
            "URL-safe by default".to_string(),
            "No timestamp component".to_string(),
        ],
        IdKind::Ksuid => vec![
            "K-Sortable: lexicographic order matches time order".to_string(),
            "160-bit: 32-bit timestamp + 128-bit random payload".to_string(),
            "Custom epoch: 2014-05-13T16:53:20Z".to_string(),
        ],
        IdKind::ObjectId => vec![
            "Used natively by MongoDB".to_string(),
            "96-bit: 4-byte timestamp + 5-byte random + 3-byte counter".to_string(),
            "Timestamp has second-level precision".to_string(),
        ],
        IdKind::TypeId => vec![
            "Type-safe: prefix encodes the entity type".to_string(),
            "Based on UUIDv7 (timestamp-sortable)".to_string(),
            "Use --prefix flag to set type prefix".to_string(),
        ],
        IdKind::Xid => vec![
            "Compact: 20-character base32hex encoding".to_string(),
            "96-bit: 4-byte timestamp + 3-byte machine + 2-byte PID + 3-byte counter".to_string(),
            "Globally unique without coordination".to_string(),
        ],
        IdKind::Cuid => vec![
            "CUID v1 is deprecated; consider CUID2".to_string(),
            "25 characters, starts with 'c'".to_string(),
            "Contains timestamp, counter, fingerprint, and random data".to_string(),
        ],
        IdKind::Cuid2 => vec![
            "Successor to CUID v1 with better security".to_string(),
            "Opaque: no extractable components".to_string(),
            "SHA-256 based with multiple entropy sources".to_string(),
        ],
        IdKind::Tsid => vec![
            "64-bit: fits in a database bigint column".to_string(),
            "42-bit timestamp (milliseconds) + 22-bit random".to_string(),
            "Crockford Base32 encoded (13 characters)".to_string(),
        ],
        _ => vec![],
    }
}

#[derive(serde::Serialize)]
struct TypeInfo {
    name: String,
    description: String,
    has_timestamp: bool,
    is_sortable: bool,
    bit_length: usize,
}

#[derive(serde::Serialize)]
struct TypeDetail {
    name: String,
    description: String,
    has_timestamp: bool,
    is_sortable: bool,
    bit_length: usize,
    example: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    spec_url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
}

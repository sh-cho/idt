use crate::core::id::{GenIdKind, IdKind};
use clap::builder::ValueHint;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "idt",
    author,
    version,
    about = "A fast, ergonomic CLI tool for working with various ID formats",
    long_about = "IDT (ID Tool) - Generate, inspect, convert, and validate various identifier formats.\n\n\
                  Supports UUID (all versions), ULID, NanoID, Snowflake, and more.",
    after_help = "Examples:\n  \
                  idt gen uuid              Generate a UUIDv4\n  \
                  idt gen ulid              Generate a ULID\n  \
                  idt inspect <ID>          Analyze and decode an ID\n  \
                  idt convert <ID> -f hex   Convert ID to hexadecimal\n  \
                  idt sort <ID>...          Sort IDs by timestamp"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output in JSON format (shorthand for --format json)
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Output format (json, yaml, toml)
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FORMAT",
        global = true,
        value_enum
    )]
    pub output_format: Option<OutputFormat>,

    /// Pretty print JSON output
    #[arg(short, long, global = true)]
    pub pretty: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate new IDs
    #[command(visible_alias = "g")]
    Gen(GenArgs),

    /// Analyze and decode ID(s)
    #[command(visible_alias = "i")]
    Inspect(InspectArgs),

    /// Convert between formats
    #[command(visible_alias = "c")]
    Convert(ConvertArgs),

    /// Check if input is a valid ID
    #[command(visible_alias = "v")]
    Validate(ValidateArgs),

    /// Compare two or more IDs
    Compare(CompareArgs),

    /// Sort IDs by their embedded timestamps
    #[command(visible_alias = "s")]
    Sort(SortArgs),

    /// Show information about ID types
    Info(InfoArgs),

    /// Generate shell completion scripts
    Completions(CompletionsArgs),

    /// Generate man pages
    Manpage(ManpageArgs),
}

#[derive(Parser)]
pub struct GenArgs {
    /// ID type to generate
    #[arg(value_name = "TYPE", ignore_case = true)]
    pub id_type: GenIdKind,

    /// Number of IDs to generate
    #[arg(short = 'n', long, default_value = "1")]
    pub count: usize,

    /// Output format (canonical, hex, base32, base58, base64)
    #[arg(short, long)]
    pub format: Option<String>,

    /// Don't print trailing newline (single ID only)
    #[arg(long)]
    pub no_newline: bool,

    /// Wrap each generated ID in a template string ({} = placeholder)
    #[arg(short = 'T', long)]
    pub template: Option<String>,

    // UUID-specific options
    /// UUID version (1, 4, 6, 7)
    #[arg(long, value_name = "VERSION")]
    pub uuid_version: Option<u8>,

    // NanoID-specific options
    /// Custom alphabet for NanoID
    #[arg(long)]
    pub alphabet: Option<String>,

    /// Length for NanoID
    #[arg(long)]
    pub length: Option<usize>,

    // Snowflake-specific options
    /// Custom epoch (discord, twitter, or milliseconds since Unix epoch)
    #[arg(long)]
    pub epoch: Option<String>,

    /// Snowflake preset (twitter, discord, instagram, sonyflake, mastodon)
    #[arg(long)]
    pub preset: Option<String>,

    /// Set a Snowflake field value (e.g., --field shard_id=42)
    #[arg(long, value_name = "NAME=VALUE")]
    pub field: Vec<String>,

    /// Machine/worker ID (0-31)
    #[arg(long)]
    pub machine_id: Option<u16>,

    /// Datacenter ID (0-31)
    #[arg(long)]
    pub datacenter_id: Option<u16>,

    // TypeID-specific options
    /// Type prefix for TypeID
    #[arg(long)]
    pub prefix: Option<String>,
}

#[derive(Parser)]
pub struct InspectArgs {
    /// ID(s) to inspect (reads from stdin if omitted)
    #[arg(value_name = "ID", value_hint = ValueHint::Other)]
    pub ids: Vec<String>,

    /// Hint the ID type (skip auto-detection)
    #[arg(short = 't', long, value_name = "TYPE", ignore_case = true)]
    pub id_type: Option<IdKind>,

    /// Epoch for Snowflake IDs (discord, twitter, or milliseconds since Unix epoch)
    #[arg(long, value_hint = ValueHint::Other)]
    pub epoch: Option<String>,

    /// Snowflake preset (twitter, discord, instagram, sonyflake, mastodon)
    #[arg(long)]
    pub preset: Option<String>,

    /// Only show errors (for validation)
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Parser)]
pub struct ConvertArgs {
    /// ID(s) to convert (reads from stdin if omitted)
    #[arg(value_name = "ID", value_hint = ValueHint::Other)]
    pub ids: Vec<String>,

    /// Source ID type (auto-detect if omitted)
    #[arg(short = 't', long, value_name = "TYPE", ignore_case = true)]
    pub id_type: Option<IdKind>,

    /// Target format
    #[arg(short, long, value_name = "FORMAT", value_hint = ValueHint::Other)]
    pub format: Option<String>,

    /// Convert to different ID type (if compatible)
    #[arg(long, value_name = "TYPE", value_hint = ValueHint::Other)]
    pub to: Option<String>,

    /// Uppercase output
    #[arg(short = 'U', long)]
    pub uppercase: bool,

    /// Lowercase output
    #[arg(short = 'L', long)]
    pub lowercase: bool,
}

#[derive(Parser)]
pub struct ValidateArgs {
    /// ID(s) to validate
    #[arg(value_name = "ID", value_hint = ValueHint::Other)]
    pub ids: Vec<String>,

    /// Expected ID type (any valid if omitted)
    #[arg(short = 't', long, value_name = "TYPE", ignore_case = true)]
    pub id_type: Option<IdKind>,

    /// No output, only exit code
    #[arg(short, long)]
    pub quiet: bool,

    /// Strict validation (reject non-canonical forms)
    #[arg(long)]
    pub strict: bool,
}

#[derive(Parser)]
pub struct CompareArgs {
    /// First ID to compare
    #[arg(value_hint = ValueHint::Other)]
    pub id1: String,

    /// Second ID to compare
    #[arg(value_hint = ValueHint::Other)]
    pub id2: String,

    /// ID type (auto-detect if omitted)
    #[arg(short = 't', long, value_name = "TYPE", ignore_case = true)]
    pub id_type: Option<IdKind>,
}

#[derive(Parser)]
pub struct InfoArgs {
    /// ID type to get information about (list all if omitted)
    #[arg(value_name = "TYPE", ignore_case = true)]
    pub id_type: Option<IdKind>,
}

#[derive(Parser)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_name = "SHELL")]
    pub shell: Shell,
}

#[derive(Parser)]
pub struct ManpageArgs {
    /// Directory to write man pages to (prints to stdout if omitted)
    #[arg(value_name = "DIR", value_hint = ValueHint::DirPath)]
    pub dir: Option<PathBuf>,
}

#[derive(Parser)]
pub struct SortArgs {
    /// ID(s) to sort (reads from stdin if omitted)
    #[arg(value_name = "ID", value_hint = ValueHint::Other)]
    pub ids: Vec<String>,

    /// Hint the ID type (skip auto-detection)
    #[arg(short = 't', long, value_name = "TYPE", ignore_case = true)]
    pub id_type: Option<IdKind>,

    /// Sort in descending order (newest first)
    #[arg(short, long)]
    pub reverse: bool,

    /// Display timestamps alongside IDs
    #[arg(long)]
    pub show_time: bool,

    /// Epoch for Snowflake IDs (discord, twitter, or milliseconds since Unix epoch)
    #[arg(long, value_hint = ValueHint::Other)]
    pub epoch: Option<String>,

    /// Snowflake preset (twitter, discord, instagram, sonyflake, mastodon)
    #[arg(long)]
    pub preset: Option<String>,

    /// Policy for IDs without timestamps: skip (default), error, end
    #[arg(long, default_value = "skip")]
    pub on_unsortable: UnsortablePolicy,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum UnsortablePolicy {
    /// Skip unsortable IDs with a warning to stderr
    #[default]
    Skip,
    /// Fail with an error if any ID is unsortable
    Error,
    /// Append unsortable IDs after sorted ones
    End,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
}

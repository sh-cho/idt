use clap::{Parser, Subcommand, ValueEnum};

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
                  idt convert <ID> -f hex   Convert ID to hexadecimal"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format for all commands
    #[arg(short, long, global = true)]
    pub json: bool,

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

    /// Show information about ID types
    Info(InfoArgs),
}

#[derive(Parser)]
pub struct GenArgs {
    /// ID type to generate
    #[arg(value_name = "TYPE")]
    pub id_type: String,

    /// Number of IDs to generate
    #[arg(short = 'n', long, default_value = "1")]
    pub count: usize,

    /// Output format (canonical, hex, base32, base58, base64)
    #[arg(short, long)]
    pub format: Option<String>,

    /// Don't print trailing newline (single ID only)
    #[arg(long)]
    pub no_newline: bool,

    // UUID-specific options
    /// UUID version (1, 3, 4, 5, 6, 7)
    #[arg(long, value_name = "VERSION")]
    pub uuid_version: Option<u8>,

    /// Namespace for UUID v3/v5 (dns, url, oid, x500, or UUID string)
    #[arg(long)]
    pub namespace: Option<String>,

    /// Name for UUID v3/v5
    #[arg(long)]
    pub name: Option<String>,

    // NanoID-specific options
    /// Custom alphabet for NanoID
    #[arg(long)]
    pub alphabet: Option<String>,

    /// Length for NanoID
    #[arg(long)]
    pub length: Option<usize>,

    // Snowflake-specific options
    /// Custom epoch in milliseconds since Unix epoch
    #[arg(long)]
    pub epoch: Option<u64>,

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
    #[arg(value_name = "ID")]
    pub ids: Vec<String>,

    /// Hint the ID type (skip auto-detection)
    #[arg(short = 't', long, value_name = "TYPE")]
    pub id_type: Option<String>,

    /// Only show errors (for validation)
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Parser)]
pub struct ConvertArgs {
    /// ID(s) to convert (reads from stdin if omitted)
    #[arg(value_name = "ID")]
    pub ids: Vec<String>,

    /// Source ID type (auto-detect if omitted)
    #[arg(short = 't', long, value_name = "TYPE")]
    pub id_type: Option<String>,

    /// Target format
    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Convert to different ID type (if compatible)
    #[arg(long, value_name = "TYPE")]
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
    #[arg(value_name = "ID")]
    pub ids: Vec<String>,

    /// Expected ID type (any valid if omitted)
    #[arg(short = 't', long, value_name = "TYPE")]
    pub id_type: Option<String>,

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
    pub id1: String,

    /// Second ID to compare
    pub id2: String,

    /// ID type (auto-detect if omitted)
    #[arg(short = 't', long, value_name = "TYPE")]
    pub id_type: Option<String>,
}

#[derive(Parser)]
pub struct InfoArgs {
    /// ID type to get information about (list all if omitted)
    #[arg(value_name = "TYPE")]
    pub id_type: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Plain,
    Csv,
    Tsv,
}

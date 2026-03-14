use clap::Parser;
use idt::cli::commands;
use idt::cli::{Cli, Commands, OutputFormat};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    // Resolve effective output format: --format takes precedence, then -j/--json
    let format = cli.output_format.or(if cli.json {
        Some(OutputFormat::Json)
    } else {
        None
    });

    let result = match &cli.command {
        Commands::Gen(args) => commands::generate::execute(args, format, cli.pretty),
        Commands::Inspect(args) => {
            commands::inspect::execute(args, format, cli.pretty, cli.no_color)
        }
        Commands::Convert(args) => commands::convert::execute(args, format, cli.pretty),
        Commands::Validate(args) => {
            commands::validate::execute(args, format, cli.pretty, cli.no_color)
        }
        Commands::Compare(args) => {
            commands::compare::execute(args, format, cli.pretty, cli.no_color)
        }
        Commands::Sort(args) => commands::sort::execute(args, format, cli.pretty, cli.no_color),
        Commands::Info(args) => commands::info::execute(args, format, cli.pretty, cli.no_color),
        Commands::Completions(args) => commands::completions::execute(args),
        Commands::Manpage(args) => commands::manpage::execute(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

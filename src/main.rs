use clap::Parser;
use idt::cli::commands;
use idt::cli::{Cli, Commands};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    let result = match &cli.command {
        Commands::Gen(args) => commands::generate::execute(args, cli.json, cli.pretty),
        Commands::Inspect(args) => {
            commands::inspect::execute(args, cli.json, cli.pretty, cli.no_color)
        }
        Commands::Convert(args) => commands::convert::execute(args, cli.json, cli.pretty),
        Commands::Validate(args) => {
            commands::validate::execute(args, cli.json, cli.pretty, cli.no_color)
        }
        Commands::Compare(args) => {
            commands::compare::execute(args, cli.json, cli.pretty, cli.no_color)
        }
        Commands::Info(args) => commands::info::execute(args, cli.json, cli.pretty, cli.no_color),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

use crate::cli::Cli;
use crate::cli::app::CompletionsArgs;
use crate::core::error::Result;
use clap::CommandFactory;
use clap_complete::generate;
use std::io;

pub fn execute(args: &CompletionsArgs) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}

use crate::cli::Cli;
use crate::cli::app::ManpageArgs;
use crate::core::error::Result;
use clap::CommandFactory;
use clap_mangen::Man;
use std::fs;
use std::io;

pub fn execute(args: &ManpageArgs) -> Result<()> {
    let cmd = Cli::command();

    match &args.dir {
        None => {
            let man = Man::new(cmd);
            man.render(&mut io::stdout())?;
        }
        Some(dir) => {
            fs::create_dir_all(dir)?;

            let man = Man::new(cmd.clone());
            let path = dir.join("idt.1");
            let mut file = fs::File::create(path)?;
            man.render(&mut file)?;

            for subcommand in cmd.get_subcommands() {
                let name = format!("idt-{}", subcommand.get_name());
                let man = Man::new(subcommand.clone());
                let path = dir.join(format!("{name}.1"));
                let mut file = fs::File::create(path)?;
                man.render(&mut file)?;
            }
        }
    }

    Ok(())
}

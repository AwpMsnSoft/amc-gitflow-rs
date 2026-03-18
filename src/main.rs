use clap::{Parser, Subcommand};
use anyhow::Result as AnyResult;

use crate::commands::init::{InitArgs, run as init_run};

mod commands;
mod core;
mod utils;

#[derive(Parser)]
#[command(name = "amc-gitflow-rs", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Init(InitArgs),
    Features,
    Bugfix,
    Release,
    Version
}

fn main() -> AnyResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => init_run(args)?,
        Commands::Features => unimplemented!(),
        Commands::Bugfix => unimplemented!(),
        Commands::Release => unimplemented!(),
        Commands::Version => unimplemented!(),
    }

    Ok(())
}
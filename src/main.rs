use clap::{Parser, Subcommand};
use anyhow::Result as AnyResult;

use crate::commands::init::{InitArgs, run as init_run};
use crate::commands::config::{ConfigArgs, run as config_run};
use crate::commands::features::{FeatureArgs, run as feature_run};
use crate::commands::bugfix::{BugfixArgs, run as bugfix_run};
use crate::commands::release::{ReleaseArgs, run as release_run};

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
    Config(ConfigArgs),
    Features(FeatureArgs),
    Bugfix(BugfixArgs),
    Release(ReleaseArgs),
    Version
}

fn main() -> AnyResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => init_run(args)?,
        Commands::Config(args) => config_run(args)?,
        Commands::Features(args) => feature_run(args)?,
        Commands::Bugfix(args) => bugfix_run(args)?,
        Commands::Release(args) => release_run(args)?,
        Commands::Version => unimplemented!(),
    }

    Ok(())
}
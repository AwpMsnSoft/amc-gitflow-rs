use clap::{Parser, Subcommand};

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
    Init,
    Features,
    Bugfix,
    Release,
    Version
}

fn main() {
    let cli = Cli::parse();
}
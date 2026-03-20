use anyhow::{Result, anyhow, bail};
use clap::{Args, Subcommand};

use crate::core::{config::GitflowConfig, git};
use crate::{error, info, item, success};

#[derive(Args, Debug)]
pub struct FeatureArgs {
    #[command(subcommand)]
    pub command: FeatureSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum FeatureSubcommand {
    /// List all feature branches
    List {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Start a new feature branch
    Start {
        /// The name of the feature
        name: String,
        /// The base branch (defaults to develop)
        base: Option<String>,
    },
    /// Finish a feature branch
    Finish {
        /// The name of the feature
        name: Option<String>,
    },
    /// Publish a feature branch to remote
    Publish {
        /// The name of the feature
        name: Option<String>,
    },
    /// Track a feature branch from remote
    Track {
        /// The name of the feature
        name: String,
    },
}

pub fn run(args: FeatureArgs) -> Result<()> {
    let config = GitflowConfig::load().map_err(|_| {
        error!("Not initialized for amc-gitflow. Run 'amc-gitflow-rs init' first.");
        anyhow!("Not initialized for amc-gitflow.")
    })?;

    match args.command {
        FeatureSubcommand::List { verbose } => list_features(&config, verbose),
        FeatureSubcommand::Start { name, base } => start_feature(&config, &name, base),
        FeatureSubcommand::Finish { name } => finish_feature(&config, name),
        FeatureSubcommand::Publish { name } => publish_feature(&config, name),
        FeatureSubcommand::Track { name } => track_feature(&config, &name),
    }
}

fn list_features(config: &GitflowConfig, verbose: bool) -> Result<()> {
    let branches = git::branch::list()?;
    let prefix = &config.feature_prefix;
    let current = git::branch::current()?;

    let feature_branches: Vec<_> = branches
        .into_iter()
        .filter(|b| b.starts_with(prefix))
        .collect();

    if feature_branches.is_empty() {
        info!("No feature branches exist.");
        return Ok(());
    }

    for branch in feature_branches {
        let short_name = &branch[prefix.len()..];
        let mark = if branch == current { "*" } else { " " };
        if verbose {
            item!("{} {} (full: {})", mark, short_name, branch);
        } else {
            item!("{} {}", mark, short_name);
        }
    }

    Ok(())
}

fn start_feature(config: &GitflowConfig, name: &str, base: Option<String>) -> Result<()> {
    let branch_name = format!("{}{}", config.feature_prefix, name);
    let base_branch = base.unwrap_or_else(|| config.develop_branch.clone());

    if git::branch::exists(&branch_name)? {
        bail!("Feature branch '{}' already exists.", branch_name);
    }

    info!(
        "Creating new feature branch '{}' based on '{}'...",
        branch_name, base_branch
    );
    git::branch::create(&branch_name, &base_branch)?;

    success!("Successfully started feature '{}'!", name);
    Ok(())
}

fn finish_feature(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let prefix = &config.feature_prefix;
    let branch_name = if let Some(n) = name {
        format!("{}{}", prefix, n)
    } else {
        let current = git::branch::current()?;
        if !current.starts_with(prefix) {
            bail!(
                "Current branch '{}' is not a feature branch and no name was provided.",
                current
            );
        }
        current
    };

    if !git::branch::exists(&branch_name)? {
        bail!("Feature branch '{}' does not exist.", branch_name);
    }

    info!("Finishing feature '{}'...", branch_name);

    // 1. Checkout develop
    git::checkout::branch(&config.develop_branch)?;

    // 2. Merge feature into develop
    info!(
        "Merging '{}' into '{}'...",
        branch_name, config.develop_branch
    );
    git::merge::no_fast_forward(&branch_name)?;

    // 3. Delete feature branch
    info!("Deleting feature branch '{}'...", branch_name);
    git::branch::delete(&branch_name, false)?;

    success!("Successfully finished feature!");
    Ok(())
}

fn publish_feature(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let prefix = &config.feature_prefix;
    let current = git::branch::current()?;
    let branch_name = if let Some(n) = name {
        format!("{}{}", prefix, n)
    } else {
        if !current.starts_with(prefix) {
            bail!(
                "Current branch '{}' is not a feature branch and no name was provided.",
                current
            );
        }
        current
    };

    info!("Publishing feature branch '{}' to origin...", branch_name);
    git::remote::push("origin", &branch_name)?;

    success!("Successfully published feature branch!");
    Ok(())
}

fn track_feature(config: &GitflowConfig, name: &str) -> Result<()> {
    let branch_name = format!("{}{}", config.feature_prefix, name);

    info!("Tracking feature branch '{}' from origin...", branch_name);
    git::remote::fetch("origin")?;
    git::branch::create(&branch_name, &format!("origin/{}", branch_name))?;

    success!("Successfully tracking feature branch!");
    Ok(())
}

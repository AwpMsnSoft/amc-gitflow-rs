use anyhow::{Result, anyhow, bail};
use clap::{Args, Subcommand};

use crate::core::{config::GitflowConfig, git};
use crate::{error, info, item, success};

#[derive(Args, Debug)]
pub struct BugfixArgs {
    #[command(subcommand)]
    pub command: BugfixSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum BugfixSubcommand {
    /// List all bugfix branches
    List {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Start a new bugfix branch
    Start {
        /// The name of the bugfix
        name: String,
        /// The base branch (defaults to develop)
        base: Option<String>,
    },
    /// Finish a bugfix branch
    Finish {
        /// The name of the bugfix
        name: Option<String>,
    },
    /// Publish a bugfix branch to remote
    Publish {
        /// The name of the bugfix
        name: Option<String>,
    },
    /// Track a bugfix branch from remote
    Track {
        /// The name of the bugfix
        name: String,
    },
}

/// List, start, finish, publish, or track bugfix branches. This command requires amc-gitflow-rs to be initialized first.
pub fn run(args: BugfixArgs) -> Result<()> {
    let config = GitflowConfig::load().map_err(|_| {
        error!("Not initialized for amc-gitflow. Run 'amc-gitflow-rs init' first.");
        anyhow!("Not initialized for amc-gitflow.")
    })?;

    match args.command {
        BugfixSubcommand::List { verbose } => list_bugfixes(&config, verbose),
        BugfixSubcommand::Start { name, base } => start_bugfix(&config, &name, base),
        BugfixSubcommand::Finish { name } => finish_bugfix(&config, name),
        BugfixSubcommand::Publish { name } => publish_bugfix(&config, name),
        BugfixSubcommand::Track { name } => track_bugfix(&config, &name),
    }
}

fn list_bugfixes(config: &GitflowConfig, verbose: bool) -> Result<()> {
    let branches = git::branch::list()?;
    let prefix = &config.bugfix_prefix;
    let current = git::branch::current()?;

    let bugfix_branches: Vec<_> = branches
        .into_iter()
        .filter(|b| b.starts_with(prefix))
        .collect();

    if bugfix_branches.is_empty() {
        info!("No bugfix branches exist.");
        return Ok(());
    }

    for branch in bugfix_branches {
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

fn start_bugfix(config: &GitflowConfig, name: &str, base: Option<String>) -> Result<()> {
    let branch_name = format!("{}{}", config.bugfix_prefix, name);
    let base_branch = base.unwrap_or_else(|| config.develop_branch.clone());

    if git::branch::exists(&branch_name)? {
        bail!("Bugfix branch '{}' already exists.", branch_name);
    }

    info!(
        "Creating new bugfix branch '{}' based on '{}'...",
        branch_name, base_branch
    );
    git::branch::create(&branch_name, &base_branch)?;

    success!("Successfully started bugfix '{}'!", name);
    Ok(())
}

fn finish_bugfix(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let prefix = &config.bugfix_prefix;
    let branch_name = if let Some(n) = name {
        format!("{}{}", prefix, n)
    } else {
        let current = git::branch::current()?;
        if !current.starts_with(prefix) {
            bail!(
                "Current branch '{}' is not a bugfix branch and no name was provided.",
                current
            );
        }
        current
    };

    if !git::branch::exists(&branch_name)? {
        bail!("Bugfix branch '{}' does not exist.", branch_name);
    }

    info!("Finishing bugfix '{}'...", branch_name);

    // 1. Checkout develop
    git::checkout::branch(&config.develop_branch)?;

    // 2. Merge bugfix into develop
    info!(
        "Merging '{}' into '{}'...",
        branch_name, config.develop_branch
    );
    git::merge::no_fast_forward(&branch_name)?;

    // 3. Delete bugfix branch
    info!("Deleting bugfix branch '{}'...", branch_name);
    git::branch::delete(&branch_name, false)?;

    success!("Successfully finished bugfix!");
    Ok(())
}

fn publish_bugfix(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let prefix = &config.bugfix_prefix;
    let current = git::branch::current()?;
    let branch_name = if let Some(n) = name {
        format!("{}{}", prefix, n)
    } else {
        if !current.starts_with(prefix) {
            bail!(
                "Current branch '{}' is not a bugfix branch and no name was provided.",
                current
            );
        }
        current
    };

    info!("Publishing bugfix branch '{}' to origin...", branch_name);
    git::remote::push("origin", &branch_name)?;

    success!("Successfully published bugfix branch!");
    Ok(())
}

fn track_bugfix(config: &GitflowConfig, name: &str) -> Result<()> {
    let branch_name = format!("{}{}", config.bugfix_prefix, name);

    info!("Tracking bugfix branch '{}' from origin...", branch_name);
    git::remote::fetch("origin")?;
    git::branch::create(&branch_name, &format!("origin/{}", branch_name))?;

    success!("Successfully tracking bugfix branch!");
    Ok(())
}

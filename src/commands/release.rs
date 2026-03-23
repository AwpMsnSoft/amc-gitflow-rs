use anyhow::{Result, anyhow, bail};
use clap::{Args, Subcommand};
use colored::Colorize;
use velvetio::ask;

use crate::core::{config::GitflowConfig, git};
use crate::{error, info, item, success};

#[derive(Args, Debug)]
pub struct ReleaseArgs {
    #[command(subcommand)]
    pub command: ReleaseSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ReleaseSubcommand {
    /// List all release branches
    List {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Start a new release branch
    Start {
        /// The release version/name
        name: String,
        /// The base branch (defaults to develop)
        base: Option<String>,
    },
    /// Finish a release branch
    Finish {
        /// The release version/name
        name: Option<String>,
    },
    /// Publish a release branch to remote
    Publish {
        /// The release version/name
        name: Option<String>,
    },
    /// Track a release branch from remote
    Track {
        /// The release version/name
        name: String,
    },
}

pub fn run(args: ReleaseArgs) -> Result<()> {
    let config = GitflowConfig::load().map_err(|_| {
        error!("Not initialized for amc-gitflow. Run 'amc-gitflow-rs init' first.");
        anyhow!("Not initialized for amc-gitflow.")
    })?;

    match args.command {
        ReleaseSubcommand::List { verbose } => list_releases(&config, verbose),
        ReleaseSubcommand::Start { name, base } => start_release(&config, &name, base),
        ReleaseSubcommand::Finish { name } => finish_release(&config, name),
        ReleaseSubcommand::Publish { name } => publish_release(&config, name),
        ReleaseSubcommand::Track { name } => track_release(&config, &name),
    }
}

fn list_releases(config: &GitflowConfig, verbose: bool) -> Result<()> {
    let branches = git::branch::list()?;
    let prefix = &config.release_prefix;
    let current = git::branch::current()?;

    let release_branches: Vec<_> = branches
        .into_iter()
        .filter(|branch| branch.starts_with(prefix))
        .collect();

    if release_branches.is_empty() {
        info!("No release branches exist.");
        return Ok(());
    }

    for branch in release_branches {
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

fn start_release(config: &GitflowConfig, name: &str, base: Option<String>) -> Result<()> {
    let branch_name = format!("{}{}", config.release_prefix, name);
    let base_branch = base.unwrap_or_else(|| config.develop_branch.clone());
    let active_releases = existing_release_branches(config)?;

    if git::branch::exists(&branch_name)? {
        bail!("Release branch '{}' already exists.", branch_name);
    }

    if !active_releases.is_empty() {
        bail!(
            "Another release branch already exists: '{}'. Finish or delete it before starting a new release.",
            active_releases[0]
        );
    }

    info!(
        "Creating new release branch '{}' based on '{}'...",
        branch_name, base_branch
    );
    git::branch::create(&branch_name, &base_branch)?;

    success!("Successfully started release '{}'!", name);
    Ok(())
}

fn finish_release(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let branch_name = resolve_release_branch_name(config, name)?;
    let release_name = branch_name
        .strip_prefix(&config.release_prefix)
        .unwrap_or(&branch_name);

    if !git::branch::exists(&branch_name)? {
        bail!("Release branch '{}' does not exist.", branch_name);
    }

    let tag_name: String;
    loop {
        let input = ask!(&"Enter the tag name".bold().to_string());
        if input.trim().is_empty() {
            error!("Tag name cannot be empty.");
            continue;
        }
        if git::tag::exists(&input)? {
            error!(
                "Tag '{}' already exists. Please enter a different tag name.",
                input
            );
            continue;
        }
        tag_name = input;
        break;
    }

    info!("Finishing release '{}'...", branch_name);

    git::checkout::branch(&config.product_branch)?;
    info!(
        "Merging '{}' into '{}'...",
        branch_name, config.product_branch
    );
    git::merge::no_fast_forward(&branch_name)?;

    info!("Creating release tag '{}'...", tag_name);
    git::tag::create(&tag_name, &format!("Release {}", release_name))?;

    git::checkout::branch(&config.develop_branch)?;
    info!(
        "Back-merging '{}' into '{}'...",
        branch_name, config.develop_branch
    );
    git::merge::no_fast_forward(&branch_name)?;

    info!("Deleting release branch '{}'...", branch_name);
    git::branch::delete(&branch_name, false)?;

    success!("Successfully finished release '{}'!", release_name);
    Ok(())
}

fn publish_release(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let branch_name = resolve_release_branch_name(config, name)?;

    if !git::branch::exists(&branch_name)? {
        bail!("Release branch '{}' does not exist.", branch_name);
    }

    if git::remote::branch_exists("origin", &branch_name)? {
        bail!(
            "Remote release branch 'origin/{}' already exists.",
            branch_name
        );
    }

    info!("Publishing release branch '{}' to origin...", branch_name);
    git::remote::push_upstream("origin", &branch_name)?;

    success!("Successfully published release branch!");
    Ok(())
}

fn track_release(config: &GitflowConfig, name: &str) -> Result<()> {
    let branch_name = format!("{}{}", config.release_prefix, name);

    if git::branch::exists(&branch_name)? {
        bail!("Release branch '{}' already exists locally.", branch_name);
    }

    info!("Fetching release branches from origin...");
    git::remote::fetch("origin")?;

    if !git::remote::branch_exists("origin", &branch_name)? {
        bail!(
            "Remote release branch 'origin/{}' does not exist.",
            branch_name
        );
    }

    info!("Tracking release branch '{}' from origin...", branch_name);
    git::branch::create(&branch_name, &format!("origin/{}", branch_name))?;

    success!("Successfully tracking release branch!");
    Ok(())
}

fn resolve_release_branch_name(config: &GitflowConfig, name: Option<String>) -> Result<String> {
    let prefix = &config.release_prefix;

    if let Some(name) = name {
        return Ok(format!("{}{}", prefix, name));
    }

    let current = git::branch::current()?;
    if !current.starts_with(prefix) {
        bail!(
            "Current branch '{}' is not a release branch and no name was provided.",
            current
        );
    }

    Ok(current)
}

fn existing_release_branches(config: &GitflowConfig) -> Result<Vec<String>> {
    Ok(git::branch::list()?
        .into_iter()
        .filter(|branch| branch.starts_with(&config.release_prefix))
        .collect())
}

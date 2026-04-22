use anyhow::{Result, anyhow, bail};
use clap::{Args, Subcommand};
use velvetio::ask;

use crate::commands::version::{BumpType, bump_version, get_current_version};
use crate::core::{
    config::{
        ConfigKey, GitflowConfig,
        private::{ConfigKey as PrivateConfigKey, *},
    },
    gh, git,
};
use crate::utils::error::IntoAnyResult;
use crate::utils::run::edit_in_editor;
use crate::{bold, error, info, item, success};

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
        /// The name of the release branch
        name: String,
        /// The base branch (defaults to develop)
        base: Option<String>,
    },
    /// Finish a release branch
    Finish {
        /// The name of the release branch
        name: Option<String>,
        /// Skip interactive prompts (use auto-generated tag and release notes)
        #[arg(short, long, default_value_t = false)]
        auto: bool,
    },
    /// Publish a release branch to remote
    Publish {
        /// The name of the release branch
        name: Option<String>,
    },
    /// Track a release branch from remote
    Track {
        /// The name of the release branch
        name: String,
    },
}

pub fn run(args: ReleaseArgs) -> Result<()> {
    let mut config = GitflowConfig::load().map_err(|_| {
        error!("Not initialized for amc-gitflow. Run 'amc-gitflow-rs init' first.");
        anyhow!("Not initialized for amc-gitflow.")
    })?;

    match args.command {
        ReleaseSubcommand::List { verbose } => list_releases(&config, verbose),
        ReleaseSubcommand::Start { name, base } => start_release(&config, &name, base),
        ReleaseSubcommand::Finish { name, auto } => finish_release(&mut config, name, auto),
        ReleaseSubcommand::Publish { name } => publish_release(&config, name),
        ReleaseSubcommand::Track { name } => track_release(&config, &name),
    }
}

fn list_releases(config: &GitflowConfig, verbose: bool) -> Result<()> {
    let branches = git::branch::list()?;
    let prefix = config.get(ConfigKey::Release);
    let current = git::branch::current()?;

    let release_branches: Vec<_> = branches
        .into_iter()
        .filter(|branch| branch.starts_with(&prefix))
        .collect();

    if release_branches.is_empty() {
        info!("No release branches exist.");
        return Ok(());
    }

    for branch in release_branches {
        let short_name = &branch[prefix.len()..];
        let mark = if branch == current { "*" } else { " " };
        if verbose {
            item!("{mark} {short_name} (full: {branch})");
        } else {
            item!("{mark} {short_name}");
        }
    }

    Ok(())
}

fn start_release(config: &GitflowConfig, name: &str, base: Option<String>) -> Result<()> {
    let branch_name = format!("{prefix}{name}", prefix = config.get(ConfigKey::Release));
    let base_branch = base.unwrap_or_else(|| config.get(ConfigKey::Develop));
    let active_releases = existing_release_branches(config)?;

    if git::branch::exists(&branch_name)? {
        bail!("Release branch '{branch_name}' already exists.");
    }

    if !active_releases.is_empty() {
        let first_active = &active_releases[0];
        bail!(
            "Another release branch already exists: '{first_active}'. Finish or delete it before starting a new release."
        );
    }

    info!("Creating new release branch '{branch_name}' based on '{base_branch}'...");
    git::branch::create(&branch_name, &base_branch)?;

    success!("Successfully started release '{name}'!");
    Ok(())
}

fn finish_release(config: &mut GitflowConfig, name: Option<String>, auto: bool) -> Result<()> {
    let branch_name = resolve_release_branch_name(config, name)?;
    let prefix = config.get(ConfigKey::Release);
    let release_name = branch_name.strip_prefix(&prefix).unwrap_or(&branch_name);

    // 1. Require a PR to have been created via `publish`
    let pr_number = get_private(PrivateConfigKey::Release(SubConfigKey::Pr(
        branch_name.clone(),
    )))
    .map_err(|_| {
        anyhow!("No PR found for release branch '{branch_name}'. Did you run 'publish' first?")
    })?;

    // 2. Check the PR has been merged on GitHub
    info!("Checking PR #{pr_number} merge status...");
    if !gh::pr::is_merged(&pr_number)? {
        bail!("PR #{pr_number} has not been merged yet. Merge it on GitHub before finishing.");
    }

    // 3. Sync product branch from remote
    let product_branch = config.get(ConfigKey::Product);
    info!("Syncing '{product_branch}' with remote...");
    git::checkout::branch(&product_branch)?;
    for remote in git::remote::list()? {
        git::remote::pull(&remote, &product_branch)?;
    }

    // 4. Determine tag name and check for tag conflicts
    let current_version = get_current_version()?;
    let tag_name = if auto {
        current_version
    } else {
        ask!(
            &bold!("Enter a tag name (without prefix)"),
            default: current_version
        )
    };
    if git::tag::exists(&tag_name)? {
        bail!("Tag '{tag_name}' already exists. Please choose a different tag name.");
    }

    // 5. Create git tag on product
    info!("Creating release tag '{tag_name}'...");
    git::tag::create(&tag_name, &format!("Release {release_name}"))?;

    // 6. Push tag to remotes
    for remote in git::remote::list()? {
        info!("Pushing tag '{tag_name}' to {remote}...");
        git::remote::push(&remote, &tag_name)?;
    }

    // 7. Create GitHub Release
    info!("Creating GitHub Release for tag '{tag_name}'...");
    let release_title = format!("Release {release_name}");

    if auto {
        // Use auto-generated notes directly
        gh::release::create(&tag_name, &release_title, None)?;
    } else {
        // Auto-generated notes as starting point, then open $EDITOR
        let auto_notes = gh::release::generate_notes(&tag_name, &product_branch, None)?;
        let notes = edit_in_editor(&auto_notes)?;
        gh::release::create(&tag_name, &release_title, Some(notes))?;
    }

    // 8. Back-merge: product → develop
    let develop_branch = config.get(ConfigKey::Develop);

    info!("Syncing '{develop_branch}' with remote...");
    git::checkout::branch(&develop_branch)?;
    for remote in git::remote::list()? {
        git::remote::pull(&remote, &develop_branch)?;
    }

    info!("Back-merging '{product_branch}' into '{develop_branch}'...");
    git::merge::no_fast_forward(&product_branch)?;

    // Push back-merge to remotes
    for remote in git::remote::list()? {
        git::remote::push(&remote, &develop_branch)?;
    }

    // 9. Clean up: delete local and remote release branches
    if git::branch::exists(&branch_name)? {
        info!("Deleting local release branch '{branch_name}'...");
        git::branch::delete(&branch_name, false)?;
    }

    for remote in git::remote::list()? {
        if git::remote::branch_exists(&remote, &branch_name)? {
            info!("Deleting remote release branch '{branch_name}' on '{remote}'...");
            git::branch::delete_remote(&remote, &branch_name)?;
        }
    }

    // 10. Clean up private config
    unset_private(PrivateConfigKey::Release(SubConfigKey::Pr(
        branch_name.clone(),
    )))?;

    // 11. Bump version if auto flag is set
    if auto {
        bump_version(config, BumpType::Patch)?;
    }

    success!("Release '{release_name}' finished! Tag: {tag_name}, GitHub Release created.");
    Ok(())
}

fn publish_release(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let branch_name = resolve_release_branch_name(config, name)?;
    let prefix = config.get(ConfigKey::Release);
    let short_name = branch_name.strip_prefix(&prefix).unwrap_or(&branch_name);
    let product_branch = config.get(ConfigKey::Product);

    if !git::branch::exists(&branch_name)? {
        bail!("Release branch '{branch_name}' does not exist.");
    }

    // Check if already published
    if git::remote::list()?
        .iter()
        .any(|remote| git::remote::branch_exists(remote, &branch_name).unwrap_or(false))
    {
        bail!("Release branch '{branch_name}' is already published to remote.");
    }
    if gh::pr::list("open")?
        .iter()
        .any(|pr| pr.branch == branch_name)
    {
        bail!("A pull request for release branch '{branch_name}' already exists.");
    }

    // Push release branch to all remotes
    for remote in git::remote::list()? {
        info!("Publishing release branch '{branch_name}' to {remote}...");
        git::remote::push_upstream(&remote, &branch_name)?;
    }

    // Create a PR targeting the product branch (not develop!)
    let pr_title = format!("release: {short_name}");
    info!("Creating pull request from '{branch_name}' into '{product_branch}'...");

    let pr_body = format!(
        include_str!("../templates/release_pr.md"),
        short_name = short_name
    );
    gh::pr::create(
        &pr_title,
        &pr_body,
        &product_branch,
        &branch_name,
        Some(&["release"]),
    )?;

    // Persist the PR number so `finish` can look it up
    let pr_number = gh::pr::list("open")?
        .iter()
        .filter_map(|pr| {
            if pr.branch == branch_name {
                Some(pr.number.clone())
            } else {
                None
            }
        })
        .next()
        .into_anyresult()?;
    set_private(
        PrivateConfigKey::Release(SubConfigKey::Pr(branch_name.clone())),
        pr_number.clone(),
    )?;

    success!("Successfully published release branch and created PR: #{pr_number}");
    Ok(())
}

fn track_release(config: &GitflowConfig, name: &str) -> Result<()> {
    let branch_name = format!("{prefix}{name}", prefix = config.get(ConfigKey::Release));

    if git::branch::exists(&branch_name)? {
        bail!("Release branch '{branch_name}' already exists locally.");
    }

    info!("Fetching release branches from origin...");
    git::remote::fetch("origin")?;

    if !git::remote::branch_exists("origin", &branch_name)? {
        bail!("Remote release branch 'origin/{branch_name}' does not exist.");
    }

    info!("Tracking release branch '{branch_name}' from origin...");
    git::branch::create(&branch_name, &format!("origin/{branch_name}"))?;

    success!("Successfully tracking release branch!");
    Ok(())
}

fn resolve_release_branch_name(config: &GitflowConfig, name: Option<String>) -> Result<String> {
    let prefix = config.get(ConfigKey::Release);

    if let Some(name) = name {
        return Ok(format!("{prefix}{name}"));
    }

    let current = git::branch::current()?;
    if !current.starts_with(&prefix) {
        bail!("Current branch '{current}' is not a release branch and no name was provided.");
    }

    Ok(current)
}

fn existing_release_branches(config: &GitflowConfig) -> Result<Vec<String>> {
    Ok(git::branch::list()?
        .into_iter()
        .filter(|branch| branch.starts_with(&config.get(ConfigKey::Release)))
        .collect())
}

use anyhow::{Result, anyhow, bail};
use clap::{Args, Subcommand};
use velvetio::ask;

use crate::core::{
    config::{
        ConfigKey, GitflowConfig,
        private::{ConfigKey as PrivateConfigKey, *},
    },
    gh, git,
};
use crate::utils::error::IntoAnyResult;
use crate::{bold, error, info, item, success};

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
    /// Start a new feature branch from a GitHub issue
    Start {
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
        /// The remote name (defaults to origin)
        #[arg(default_value = "origin")]
        remote: String,
    },
}

pub fn run(args: FeatureArgs) -> Result<()> {
    let config = GitflowConfig::load().map_err(|_| {
        error!("Not initialized for amc-gitflow. Run 'amc-gitflow-rs init' first.");
        anyhow!("Not initialized for amc-gitflow.")
    })?;

    match args.command {
        FeatureSubcommand::List { verbose } => list_features(&config, verbose),
        FeatureSubcommand::Start { base } => start_feature(&config, base),
        FeatureSubcommand::Finish { name } => finish_feature(&config, name),
        FeatureSubcommand::Publish { name } => publish_feature(&config, name),
        FeatureSubcommand::Track { name, remote } => track_feature(&config, &name, &remote),
    }
}

fn list_features(config: &GitflowConfig, verbose: bool) -> Result<()> {
    let branches = git::branch::list()?;
    let prefix = config.get(ConfigKey::Feature);
    let current = git::branch::current()?;

    let feature_branches: Vec<_> = branches
        .into_iter()
        .filter(|b| b.starts_with(&prefix))
        .collect();

    if feature_branches.is_empty() {
        info!("No feature branches exist.");
        return Ok(());
    }

    for branch in feature_branches {
        let short_name = &branch[prefix.len()..];
        let mark = if branch == current { "*" } else { " " };
        let issue_id = get_private(PrivateConfigKey::Feature(SubConfigKey::Issue(
            branch.clone(),
        )))?;

        if verbose {
            item!("{mark} {short_name} <issue #{issue_id}>: {branch}");
        } else {
            item!("{mark} {short_name} <issue #{issue_id}>");
        }
    }

    Ok(())
}

fn start_feature(config: &GitflowConfig, base: Option<String>) -> Result<()> {
    // 1. Fetch open issues
    info!("Fetching open issues from GitHub...");
    let issues = gh::issue::list()?;
    if issues.is_empty() {
        bail!("No open issues found on GitHub. Please create an issue first.");
    }

    // 2. Select an issue
    issues.iter().enumerate().for_each(|(i, issue)| {
        item!(
            "[{i}] issue #{number}: {title} <{tags}>",
            number = issue.number,
            title = issue.title,
            tags = issue.tags
        );
    });

    let selected_index = ask!(
        &bold!("Select issue index [0-{max_idx}]", max_idx = issues.len() - 1) => usize,
        validate: |input| *input < issues.len(),
        error: "Invalid issue index selected."
    );
    let selected_issue = &issues[selected_index];

    // 3. Prompt for branch name (defaulting to issue-linked name)
    let name = ask!(
        &bold!("Enter feature name"),
        validate: |input| !input.trim().is_empty(),
        error: "Feature name cannot be empty."
    );

    let base_branch = if let Some(base) = base {
        base
    } else {
        config.get(ConfigKey::Develop)
    };
    if !git::branch::exists(&base_branch)? {
        bail!("Base branch '{base_branch}' does not exist.");
    }

    let branch_name = format!("{}{}", config.get(ConfigKey::Feature), name);
    if git::branch::exists(&branch_name)? {
        bail!("Feature branch '{branch_name}' already exists.");
    }

    info!(
        "Creating new feature branch '{branch_name}' based on '{base_branch}' for issue #{issue_number}...",
        issue_number = selected_issue.number
    );
    git::branch::create(&branch_name, &base_branch)?;

    // 4. Bind issue to branch using private key
    set_private(
        PrivateConfigKey::Feature(SubConfigKey::Issue(branch_name.clone())),
        selected_issue.number.clone(),
    )?;

    success!(
        "Successfully started feature '{name}' for issue #{issue_number}!",
        issue_number = selected_issue.number
    );
    Ok(())
}

fn finish_feature(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let prefix = config.get(ConfigKey::Feature);
    let branch_name = if let Some(n) = name {
        format!("{prefix}{n}")
    } else {
        let current = git::branch::current()?;
        if !current.starts_with(&prefix) {
            bail!("Current branch '{current}' is not a feature branch and no name was provided.");
        }
        current
    };

    // Look up the stored PR URL/number for this branch
    let pr_number = get_private(PrivateConfigKey::Feature(SubConfigKey::Pr(
        branch_name.clone(),
    )))
    .map_err(|_| {
        anyhow!("No PR found for feature branch '{branch_name}'. Did you run 'publish' first?")
    })?;

    // Check whether the PR has been merged on the remote
    info!("Checking PR #{pr_number} merge status...");
    if !gh::pr::is_merged(&pr_number)? {
        bail!(
            "PR #{pr_number} has not been merged yet. Finish is only allowed after the remote PR is merged."
        );
    }

    let base_branch = config.get(ConfigKey::Develop);

    // Checkout develop and pull the merged changes
    info!("Syncing '{base_branch}' with remote...");
    git::checkout::branch(&base_branch)?;
    for remote in git::remote::list()? {
        git::remote::pull(&remote, &base_branch)?;
    }

    // Delete the local feature branch
    if git::branch::exists(&branch_name)? {
        info!("Deleting local feature branch '{branch_name}'...");
        git::branch::delete(&branch_name, false)?;
    }

    // Delete the remote feature branch
    for remote in git::remote::list()? {
        if git::remote::branch_exists(&remote, &branch_name)? {
            info!("Deleting remote feature branch '{branch_name}' on '{remote}'...");
            git::branch::delete_remote(&remote, &branch_name)?;
        }
    }

    // Clean up the stored private config keys
    unset_private(PrivateConfigKey::Feature(SubConfigKey::Pr(
        branch_name.clone(),
    )))?;
    unset_private(PrivateConfigKey::Feature(SubConfigKey::Issue(
        branch_name.clone(),
    )))?;

    success!("Feature '{branch_name}' finished and cleaned up successfully!");
    Ok(())
}

fn publish_feature(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let prefix = config.get(ConfigKey::Feature);
    let current = git::branch::current()?;
    let branch_name = if let Some(n) = name {
        format!("{prefix}{n}")
    } else {
        if !current.starts_with(&prefix) {
            bail!("Current branch '{current}' is not a feature branch and no name was provided.");
        }
        current
    };

    if !git::branch::exists(&branch_name)? {
        bail!("Feature branch '{branch_name}' does not exist.");
    }

    let base_branch = config.get(ConfigKey::Develop);
    let short_name = &branch_name[prefix.len()..];

    // Check if the branch is already published (has an upstream or already has a open related PR)
    if git::remote::list()?
        .iter()
        .any(|remote| git::remote::branch_exists(&remote, &branch_name).unwrap_or(false))
    {
        bail!("Feature branch '{branch_name}' is already published to remote.");
    }
    if gh::pr::list("open")?
        .iter()
        .any(|pr| pr.branch == branch_name)
    {
        bail!("A pull request for feature branch '{branch_name}' already exists.");
    }

    // Get linked issue number if exists
    let issue_number = get_private(PrivateConfigKey::Feature(SubConfigKey::Issue(
        branch_name.clone(),
    )))?;

    // Push feature branch to remote
    for remote in git::remote::list()? {
        info!("Publishing feature branch '{branch_name}' to {remote}...");
        git::remote::push_upstream(&remote, &branch_name)?;
    }

    // Create a PR targeting the develop branch
    let pr_title = format!("feat: {short_name}");
    info!("Creating pull request from '{branch_name}' into '{base_branch}'...");

    let pr_body = format!(
        r"
### What & Why
- Summary:

### Checklist
- [ ] Single-purpose
- [ ] Lint/format OK
- [ ] Type hints added
- [ ] Tests updated/passing
- [ ] Docs updated (if needed)
- [ ] Fixes # (optional)

### Notes
- Impact / alternatives / rollout:

### Links
- Issue: #{issue_number}
- References: <urls>

### Breaking Changes (if any)
- Description & migration notes:
",
    );
    gh::pr::create(
        &pr_title,
        &pr_body,
        &base_branch,
        &branch_name,
        Some(&["enhancement"]),
    )?;

    // Persist the PR number keyed by branch name so `finish` can look it up
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
        PrivateConfigKey::Feature(SubConfigKey::Pr(branch_name.clone())),
        pr_number.clone(),
    )?;

    success!("Successfully published feature branch and created PR: #{pr_number}");
    Ok(())
}

fn track_feature(config: &GitflowConfig, name: &str, remote: &str) -> Result<()> {
    let branch_name = format!("{}{}", config.get(ConfigKey::Feature), name);

    info!("Tracking feature branch '{branch_name}' from {remote}...");
    git::remote::fetch(remote)?;
    git::branch::create(&branch_name, &format!("{remote}/{branch_name}"))?;

    success!("Successfully tracking feature branch!");
    Ok(())
}

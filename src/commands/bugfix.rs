use anyhow::{Result, anyhow, bail};
use clap::{Args, Subcommand};

use crate::core::{
    config::{ConfigKey, GitflowConfig},
    gh, git,
};
use crate::utils::error::IntoAnyResult;
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
        /// The remote name (defaults to origin)
        #[arg(default_value = "origin")]
        remote: String,
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
        BugfixSubcommand::Track { name, remote } => track_bugfix(&config, &name, &remote),
    }
}

fn list_bugfixes(config: &GitflowConfig, verbose: bool) -> Result<()> {
    let branches = git::branch::list()?;
    let prefix = &config.get(ConfigKey::Bugfix);
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
    let branch_name = format!("{}{}", config.get(ConfigKey::Bugfix), name);
    let base_branch = base.unwrap_or_else(|| config.get(ConfigKey::Develop));

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
    let prefix = &config.get(ConfigKey::Bugfix);
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

    // Look up the stored PR URL/number for this branch
    let private_key = format!(
        "bugfix-pr.{}",
        branch_name.replace('/', ".").replace('_', "-")
    );
    let pr_number = GitflowConfig::get_private(&private_key).map_err(|_| {
        anyhow!(
            "No PR found for bugfix branch '{}'. Did you run 'publish' first?",
            branch_name
        )
    })?;

    // Check whether the PR has been merged on the remote
    info!("Checking PR #{} merge status...", pr_number);
    if !gh::pr::is_merged(&pr_number)? {
        bail!(
            "PR #{} has not been merged yet. Finish is only allowed after the remote PR is merged.",
            pr_number,
        );
    }

    let base_branch = config.get(ConfigKey::Develop);

    // Checkout develop and pull the merged changes
    info!("Syncing '{}' with remote...", base_branch);
    git::checkout::branch(&base_branch)?;
    for remote in git::remote::list()? {
        git::remote::pull(&remote, &base_branch)?;
    }

    // Delete the local bugfix branch
    if git::branch::exists(&branch_name)? {
        info!("Deleting local bugfix branch '{}'...", branch_name);
        git::branch::delete(&branch_name, false)?;
    }

    // Delete the remote bugfix branch
    for remote in git::remote::list()? {
        if git::remote::branch_exists(&remote, &branch_name)? {
            info!(
                "Deleting remote bugfix branch '{}' on '{}'...",
                branch_name, remote
            );
            git::branch::delete_remote(&remote, &branch_name)?;
        }
    }

    // Clean up the stored private config key
    GitflowConfig::unset_private(&private_key)?;

    success!(
        "Bugfix '{}' finished and cleaned up successfully!",
        branch_name
    );
    Ok(())
}

fn publish_bugfix(config: &GitflowConfig, name: Option<String>) -> Result<()> {
    let prefix = &config.get(ConfigKey::Bugfix);
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

    if !git::branch::exists(&branch_name)? {
        bail!("Bugfix branch '{}' does not exist.", branch_name);
    }

    let base_branch = config.get(ConfigKey::Develop);
    let short_name = &branch_name[prefix.len()..];

    // Check if the branch is already published (has an upstream or already has a open related PR)
    if git::remote::list()?
        .iter()
        .any(|remote| git::remote::branch_exists(&remote, &branch_name).unwrap_or(false))
    {
        bail!(
            "Bugfix branch '{}' is already published to remote.",
            branch_name
        );
    }
    if gh::pr::list("open")?
        .iter()
        .any(|pr| pr.branch == branch_name)
    {
        bail!(
            "A pull request for bugfix branch '{}' already exists.",
            branch_name
        );
    }

    // Push bugfix branch to remote
    for remote in git::remote::list()? {
        info!(
            "Publishing bugfix branch '{}' to {}...",
            branch_name, remote
        );
        git::remote::push_upstream(&remote, &branch_name)?;
    }

    // Create a PR targeting the develop branch
    let pr_title = format!("fix: {}", short_name);
    info!(
        "Creating pull request from '{}' into '{}'...",
        branch_name, base_branch
    );
    let pr_body = r"
### Bug Description
- What is the current behavior?
- What is the expected behavior?

### Fix Approach
- Why was this bug happening?
- How was the bug resolved?

### Checklist
- [ ] Reproducible test case added
- [ ] Lint/format OK
- [ ] Type hints added
- [ ] Tests updated/passing
- [ ] Docs updated (if needed)

### Links
- Issues: Fixes #123 (optional)
- References: <urls>
";
    gh::pr::create(&pr_title, &pr_body, &base_branch, &branch_name)?;

    // Persist the PR number keyed by branch name so `finish` can look it up
    let private_key = format!(
        "bugfix-pr.{}",
        branch_name.replace('/', ".").replace('_', "-")
    );
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
    GitflowConfig::set_private(&private_key, pr_number.clone())?;

    success!(
        "Successfully published bugfix branch and created PR: #{}",
        pr_number
    );
    Ok(())
}

fn track_bugfix(config: &GitflowConfig, name: &str, remote: &str) -> Result<()> {
    let branch_name = format!("{}{}", config.get(ConfigKey::Bugfix), name);

    info!(
        "Tracking bugfix branch '{}' from {}...",
        branch_name, remote
    );
    git::remote::fetch(remote)?;
    git::branch::create(&branch_name, &format!("{}/{}", remote, branch_name))?;

    success!("Successfully tracking bugfix branch!");
    Ok(())
}

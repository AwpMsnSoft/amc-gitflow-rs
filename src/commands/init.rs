use anyhow::{Result, bail};
use clap::Args;

use crate::core::{
    config::{ConfigKey, GitflowConfig},
    gh, git,
};
use crate::{error, info, success, warn};

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Force setting of gitflow branches, even if already configured
    #[arg(short, long)]
    pub force: bool,

    /// Use default branch naming conventions
    #[arg(short, long)]
    pub defaults: bool,
}

/// Initialize amc-gitflow-rs in the current repository. This will set up the necessary git branches and configuration for using amc-gitflow. If the repository is not already a git repository, it will be initialized as one. If amc-gitflow is already initialized, this command will do nothing unless the --force flag is used to reinitialize it.
pub fn run(args: InitArgs) -> Result<()> {
    info!("Initializing amc-gitflow...");

    // 1. Check tools
    if !git::is_installed() {
        error!("git is not installed.");
        bail!("git not found");
    }

    if !gh::is_installed() {
        error!("gh is not installed.");
        bail!("gh not found");
    }

    if !gh::auth::is_authenticated() {
        warn!("gh not authenticated. Running 'gh auth login'...");
        gh::auth::login()?;
    }

    // 2. Initializing git if needed
    let is_git_repo = git::status::is_clean().is_ok() || git::branch::current().is_ok();
    if !is_git_repo {
        info!("Initializing new git repository...");
        git::repo::init()?;
    }

    // 3. Gitflow initialization check
    let is_initialized = GitflowConfig::load().is_ok();

    if is_initialized && !args.force {
        warn!("Already initialized for amc-gitflow.");
        info!("To force reinitialization, use: amc-gitflow-rs init -f");
        return Ok(());
    }

    // 4. Determine configuration
    let config = if args.defaults {
        GitflowConfig::default()
    } else {
        GitflowConfig::new()
    };

    // 5. Apply configuration
    config.save()?;

    // 6. Ensure branches exist
    if !git::branch::exists(&config.get(ConfigKey::Product))? {
        // Fresh repo check: if current branch fails, we need an initial commit
        if git::branch::current().is_err() {
            info!("Creating initial commit...");
            git::commit::init()?;
        }
    }

    if !git::branch::exists(&config.get(ConfigKey::Develop))? {
        info!("Creating {} branch...", config.get(ConfigKey::Develop));
        git::branch::create(
            &config.get(ConfigKey::Develop),
            &config.get(ConfigKey::Product),
        )?;
    }

    success!("Successfully initialized amc-gitflow!");
    info!("Production branch: {}", config.get(ConfigKey::Product));
    info!("Development branch: {}", config.get(ConfigKey::Develop));

    Ok(())
}

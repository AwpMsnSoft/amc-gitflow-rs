use anyhow::{Result as AnyResult, bail};
use clap::Args;
use velvetio::{ask, confirm};

use crate::core::{
    config::{ConfigKey, GitflowConfig},
    gh, git,
};
use crate::{bold, error, info, success, warn};

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Force setting of gitflow branches, even if already configured
    #[arg(short, long)]
    pub force: bool,

    /// Use default branch naming conventions
    #[arg(short, long)]
    pub defaults: bool,

    /// Synchronize with a remote repository if specified (default: not synchronized)
    #[arg(short, long, default_value = None)]
    pub remote: Option<String>,
}

/// Initialize amc-gitflow-rs in the current repository. This will set up the necessary git branches and configuration for using amc-gitflow. If the repository is not already a git repository, it will be initialized as one. If amc-gitflow is already initialized, this command will do nothing unless the --force flag is used to reinitialize it.
pub fn run(args: InitArgs) -> AnyResult<()> {
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
    if !git::repo::is_repository() {
        info!("Repository not found. Initializing new git repository...");
        git::repo::init()?;
    }

    // 3. Initializing remote repository if needed
    if let Some(remote) = &args.remote {
        if !git::remote::has_remotes()? {
            info!("No remotes found. Creating a new GitHub repository...");
            let is_org = confirm(&bold!("Is this repository for an organization?"));
            let owner = if is_org {
                ask::<String>(&bold!("Organization name:"))
            } else {
                gh::auth::username()?
            };
            let repo_name = git::repo::name()?;
            let is_public = confirm(&bold!("Make the repository public?"));

            match gh::repo::create(&repo_name, &remote, is_public, &owner) {
                Ok(_) => success!("Successfully created the remote repository."),
                Err(e) => {
                    error!(
                        "Failed to create remote repository. It might already exist or you might not have permissions."
                    );
                    bail!("Failed to create remote repository: {e}");
                }
            }
        } else {
            info!("Remote already configured.");
        }
    }

    // 4. Gitflow initialization check
    let is_initialized = GitflowConfig::load().is_ok();

    if is_initialized && !args.force {
        warn!("Already initialized for amc-gitflow.");
        info!("To force reinitialization, use: amc-gitflow-rs init -f");
        return Ok(());
    }

    // 5. Get amc-gitflow configuration
    let config = if args.defaults {
        GitflowConfig::default()
    } else {
        info!("No existing configuration found. Setting up amc-gitflow configuration...");
        GitflowConfig::new()
    };
    config.save()?;

    // 6. Ensure branches exist
    if !git::branch::exists(&config.get(ConfigKey::Product))? {
        // Fresh repo check: if current branch fails, we need an initial commit
        if git::branch::current().is_err() {
            info!(
                "The repository appears to be new and has no commits. Creating an initial commit..."
            );
            git::commit::init()?;
        }
    }

    if !git::branch::exists(&config.get(ConfigKey::Develop))? {
        let develop_branch = config.get(ConfigKey::Develop);
        info!("Creating {develop_branch} branch...");
        git::branch::create(&develop_branch, &config.get(ConfigKey::Product))?;
    }

    // 7. Push local branches to remote if needed
    if args.remote.is_some() {
        for remote in git::remote::list()? {
            info!("Pushing branches to remote '{remote}'...");
            git::remote::push(&remote, &config.get(ConfigKey::Product))?;
            git::remote::push(&remote, &config.get(ConfigKey::Develop))?;
        }
    }

    success!("Successfully initialized amc-gitflow!");
    info!("Production branch: {}", config.get(ConfigKey::Product));
    info!("Development branch: {}", config.get(ConfigKey::Develop));

    Ok(())
}

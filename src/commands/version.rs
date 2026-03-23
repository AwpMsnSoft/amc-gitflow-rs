use anyhow::{Result, anyhow};
use clap::{Args, Subcommand, ValueEnum};

use crate::core::config::GitflowConfig;
use crate::{error, info, success};

#[derive(Args, Debug)]
pub struct VersionArgs {
    #[command(subcommand)]
    pub command: Option<VersionSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum VersionSubcommand {
    /// Show current project version
    Show,
    /// Bump project version
    Bump {
        /// Type of bump
        target: BumpType,
    },
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum BumpType {
    /// Increments the first digit (X.y.z -> (X+1).0.0)
    Major,
    /// Increments the second digit (x.Y.z -> x.(Y+1).0)
    Minor,
    /// Increments the third digit (x.y.Z -> x.y.(Z+1))
    Patch,
}

/// Handle version related commands.
pub fn run(args: VersionArgs) -> Result<()> {
    let mut config = GitflowConfig::load().map_err(|_| {
        error!("Not initialized for amc-gitflow. Run 'amc-gitflow-rs init' first.");
        anyhow!("Not initialized for amc-gitflow.")
    })?;

    match args.command.unwrap_or(VersionSubcommand::Show) {
        VersionSubcommand::Show => show_version(&config),
        VersionSubcommand::Bump { target } => bump_version(&mut config, target),
    }
}

pub fn show_version(config: &GitflowConfig) -> Result<()> {
    info!("Current version: {}", config.project_version);
    Ok(())
}

fn bump_version(config: &mut GitflowConfig, target: BumpType) -> Result<()> {
    let current = config.project_version.clone();
    let next = match target {
        BumpType::Major => increment_semver(&current, 0)?,
        BumpType::Minor => increment_semver(&current, 1)?,
        BumpType::Patch => increment_semver(&current, 2)?,
    };

    info!("Bumping version from {} to {}...", current, next);
    config.project_version = next.clone();
    config.save()?;
    success!("Successfully bumped version to {}!", next);
    Ok(())
}

pub fn get_current_version() -> Result<String> {
    let config = GitflowConfig::load()?;
    Ok(config.project_version)
}

fn increment_semver(version: &str, index: usize) -> Result<String> {
    let mut parts: Vec<u32> = version
        .split('.')
        .map(|s| s.parse().map_err(|_| anyhow!("Invalid semver: {}", version)))
        .collect::<Result<Vec<u32>>>()?;

    if parts.len() != 3 {
        return Err(anyhow!("Only x.y.z semver format is supported for automatic bumping. Current: {}", version));
    }

    parts[index] += 1;
    for i in (index + 1)..3 {
        parts[i] = 0;
    }

    Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
}

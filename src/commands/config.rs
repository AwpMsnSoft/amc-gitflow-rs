use anyhow::{Result as AnyResult, anyhow};
use clap::{Args, Subcommand};
use velvetio::confirm;

use crate::core::config::{CONFIG_DESCRIPTIONS, ConfigKey, GitflowConfig};
use crate::{bold, error, info, item, success, warn};

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// List all gitflow configuration
    List,
    /// Set a configuration value
    Set {
        /// The configuration key
        key: ConfigKey,
        /// The configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// The configuration key
        key: ConfigKey,
    },
}

/// List, get, or set gitflow configuration values. This command requires amc-gitflow-rs to be initialized first.
pub fn run(args: ConfigArgs) -> AnyResult<()> {
    let mut config = GitflowConfig::load().map_err(|_| {
        error!("Not initialized for amc-gitflow. Run 'amc-gitflow-rs init' first.");
        anyhow!("Not initialized for amc-gitflow.")
    })?;

    match args.command {
        ConfigSubcommand::List => {
            for key in ConfigKey::VARIANTS {
                item!(
                    "{}:\t{}",
                    CONFIG_DESCRIPTIONS.get(key).unwrap(),
                    config.get(key.clone())
                );
            }
        }
        ConfigSubcommand::Get { key } => {
            info!(
                "{}:\t{}",
                CONFIG_DESCRIPTIONS.get(&key).unwrap(),
                config.get(key.clone())
            );
        }
        ConfigSubcommand::Set { key, value } => {
            if let ConfigKey::Version = key {
                warn!(
                    "Project version is typically managed through 'amc-gitflow-rs version' commands and should not be set manually in most cases. Setting it directly may lead to unexpected behavior."
                );
                if !confirm(&bold!("Are you sure you want to set the version manually?")) {
                    return Ok(());
                }
            }
            let old_value = config.get(key.clone());
            config.set(key.clone(), value.clone());
            config.save()?;
            success!(
                "Updated {} from {} to {}",
                CONFIG_DESCRIPTIONS.get(&key).unwrap(),
                old_value,
                value
            );
        }
    }
    Ok(())
}

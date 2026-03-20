use anyhow::{Result as AnyResult, anyhow};
use clap::{Args, Subcommand};

use crate::core::config::{CONFIG_DESCRIPTIONS, ConfigKey, GitflowConfig};
use crate::{error, info, item, success};

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
                    CONFIG_DESCRIPTIONS.get(key.as_str()).unwrap(),
                    config.get(key.as_str())
                );
            }
        }
        ConfigSubcommand::Get { key } => {
            info!(
                "{}:\t{}",
                CONFIG_DESCRIPTIONS.get(key.as_str()).unwrap(),
                config.get(key.as_str())
            );
        }
        ConfigSubcommand::Set { key, value } => {
            config.set(key.as_str(), value.clone());
            config.save()?;
            success!(
                "Updated {} to {}",
                CONFIG_DESCRIPTIONS.get(key.as_str()).unwrap(),
                value
            );
        }
    }
    Ok(())
}

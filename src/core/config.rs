use anyhow::Result as AnyResult;
use colored::Colorize;
use velvetio::prelude::*;

use crate::core::git;

/// Gitflow configuration structure
pub struct GitflowConfig {
    pub master_branch: String,
    pub develop_branch: String,
    pub feature_prefix: String,
    pub release_prefix: String,
    pub hotfix_prefix: String,
    pub support_prefix: String,
    pub version_tag_prefix: String,
}

impl GitflowConfig {
    /// Create a new GitflowConfig with default values
    pub fn default() -> Self {
        Self {
            master_branch: "master".to_string(),
            develop_branch: "develop".to_string(),
            feature_prefix: "feature/".to_string(),
            release_prefix: "release/".to_string(),
            hotfix_prefix: "hotfix/".to_string(),
            support_prefix: "support/".to_string(),
            version_tag_prefix: "".to_string(),
        }
    }

    /// Create a new GitflowConfig with console inputs
    pub fn new() -> Self {
        macro_rules! bold {
            ($args: tt) => {{ $args.bold().to_string() }};
        }

        let master_branch = ask!(
            &bold!("Branch name for production releases"),
            default: "master".to_string()
        );
        let develop_branch = ask!(
            &bold!("Branch name for \"next release\" development"),
            default: "develop".to_string()
        );
        let feature_prefix = ask!(
            &bold!("Prefix for feature branches"),
            default: "feature/".to_string()
        );
        let release_prefix = ask!(
            &bold!("Prefix for release branches"),
            default: "release/".to_string()
        );
        let hotfix_prefix = ask!(
            &bold!("Prefix for hotfix branches"),
            default: "hotfix/".to_string()
        );
        let support_prefix = ask!(
            &bold!("Prefix for support branches"),
            default: "support/".to_string()
        );
        let version_tag_prefix = ask!(
            &bold!("Prefix for version tags"),
            default: "".to_string()
        );

        Self {
            master_branch,
            develop_branch,
            feature_prefix,
            release_prefix,
            hotfix_prefix,
            support_prefix,
            version_tag_prefix,
        }
    }

    /// Load current gitflow configuration from git config
    pub fn load() -> AnyResult<Self> {
        Ok(Self {
            master_branch: git::config::get("amc-gitflow-rs.branch.master")?,
            develop_branch: git::config::get("amc-gitflow-rs.branch.develop")?,
            feature_prefix: git::config::get("amc-gitflow-rs.prefix.feature")?,
            release_prefix: git::config::get("amc-gitflow-rs.prefix.release")?,
            hotfix_prefix: git::config::get("amc-gitflow-rs.prefix.hotfix")?,
            support_prefix: git::config::get("amc-gitflow-rs.prefix.support")?,
            version_tag_prefix: git::config::get("amc-gitflow-rs.prefix.versiontag")?,
        })
    }

    /// Save current configuration to git config
    pub fn save(&self) -> AnyResult<()> {
        git::config::set("amc-gitflow-rs.branch.master", &self.master_branch)?;
        git::config::set("amc-gitflow-rs.branch.develop", &self.develop_branch)?;
        git::config::set("amc-gitflow-rs.prefix.feature", &self.feature_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.release", &self.release_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.hotfix", &self.hotfix_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.support", &self.support_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.versiontag", &self.version_tag_prefix)?;
        Ok(())
    }
}

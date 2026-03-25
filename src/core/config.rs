use anyhow::Result as AnyResult;
use clap::ValueEnum;
use lazy_static::lazy_static;
use std::collections::HashMap;
use velvetio::prelude::*;

use crate::{bold, core::git};

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConfigKey {
    Product,
    Develop,
    Feature,
    Release,
    Bugfix,
    Support,
    Versiontag,
    Version,
}

impl ConfigKey {
    pub const VARIANTS: &'static [ConfigKey] = &[
        ConfigKey::Product,
        ConfigKey::Develop,
        ConfigKey::Feature,
        ConfigKey::Release,
        ConfigKey::Bugfix,
        ConfigKey::Support,
        ConfigKey::Versiontag,
        ConfigKey::Version,
    ];
}

/// Gitflow configuration structure
pub struct GitflowConfig {
    product_branch: String,
    develop_branch: String,
    feature_prefix: String,
    release_prefix: String,
    bugfix_prefix: String,
    support_prefix: String,
    versiontag_prefix: String,
    project_version: String,
}

lazy_static! {
    pub static ref CONFIG_DESCRIPTIONS: HashMap<ConfigKey, &'static str> = {
        let mut m = HashMap::new();
        m.insert(ConfigKey::Product, "branch name for production releases");
        m.insert(
            ConfigKey::Develop,
            "branch name for \"next release\" development",
        );
        m.insert(ConfigKey::Feature, "prefix for feature branches");
        m.insert(ConfigKey::Release, "prefix for release branches");
        m.insert(ConfigKey::Bugfix, "prefix for bugfix branches");
        m.insert(ConfigKey::Support, "prefix for support branches");
        m.insert(ConfigKey::Versiontag, "prefix for version tags");
        m.insert(ConfigKey::Version, "current project version");
        m
    };
}

impl GitflowConfig {
    /// Create a new GitflowConfig with default values
    pub fn default() -> Self {
        Self {
            product_branch: "master".to_string(),
            develop_branch: "develop".to_string(),
            feature_prefix: "feature/".to_string(),
            release_prefix: "release/".to_string(),
            bugfix_prefix: "bugfix/".to_string(),
            support_prefix: "support/".to_string(),
            versiontag_prefix: "".to_string(),
            project_version: "0.1.0".to_string(),
        }
    }

    /// Create a new GitflowConfig with console inputs
    pub fn new() -> Self {
        let product_branch = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get(&ConfigKey::Product).unwrap_or(&"product branch")),
            default: "master".to_string()
        );
        let develop_branch = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get(&ConfigKey::Develop).unwrap_or(&"develop branch")),
            default: "develop".to_string()
        );
        let feature_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get(&ConfigKey::Feature).unwrap_or(&"feature prefix")),
            default: "feature/".to_string()
        );
        let release_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get(&ConfigKey::Release).unwrap_or(&"release prefix")),
            default: "release/".to_string()
        );
        let bugfix_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get(&ConfigKey::Bugfix).unwrap_or(&"bugfix prefix")),
            default: "bugfix/".to_string()
        );
        let support_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get(&ConfigKey::Support).unwrap_or(&"support prefix")),
            default: "support/".to_string()
        );
        let versiontag_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get(&ConfigKey::Versiontag).unwrap_or(&"versiontag prefix")),
            default: "".to_string()
        );

        Self {
            product_branch,
            develop_branch,
            feature_prefix,
            release_prefix,
            bugfix_prefix,
            support_prefix,
            versiontag_prefix,
            project_version: "0.1.0".to_string(),
        }
    }

    /// Load current gitflow configuration from git config
    pub fn load() -> AnyResult<Self> {
        Ok(Self {
            product_branch: git::config::get("amc-gitflow-rs.branch.product")?,
            develop_branch: git::config::get("amc-gitflow-rs.branch.develop")?,
            feature_prefix: git::config::get("amc-gitflow-rs.prefix.feature")?,
            release_prefix: git::config::get("amc-gitflow-rs.prefix.release")?,
            bugfix_prefix: git::config::get("amc-gitflow-rs.prefix.bugfix")?,
            support_prefix: git::config::get("amc-gitflow-rs.prefix.support")?,
            versiontag_prefix: git::config::get("amc-gitflow-rs.prefix.versiontag")?,
            project_version: git::config::get("amc-gitflow-rs.project.version")
                .unwrap_or_else(|_| "0.1.0".to_string()),
        })
    }

    /// Save current configuration to git config
    pub fn save(&self) -> AnyResult<()> {
        git::config::set("amc-gitflow-rs.branch.product", &self.product_branch)?;
        git::config::set("amc-gitflow-rs.branch.develop", &self.develop_branch)?;
        git::config::set("amc-gitflow-rs.prefix.feature", &self.feature_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.release", &self.release_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.bugfix", &self.bugfix_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.support", &self.support_prefix)?;
        git::config::set("amc-gitflow-rs.prefix.versiontag", &self.versiontag_prefix)?;
        git::config::set("amc-gitflow-rs.project.version", &self.project_version)?;
        Ok(())
    }

    /// Get a configuration value by key
    pub fn get(&self, key: ConfigKey) -> String {
        match key {
            ConfigKey::Product => self.product_branch.clone(),
            ConfigKey::Develop => self.develop_branch.clone(),
            ConfigKey::Feature => self.feature_prefix.clone(),
            ConfigKey::Release => self.release_prefix.clone(),
            ConfigKey::Bugfix => self.bugfix_prefix.clone(),
            ConfigKey::Support => self.support_prefix.clone(),
            ConfigKey::Versiontag => self.versiontag_prefix.clone(),
            ConfigKey::Version => self.project_version.clone(),
        }
    }

    /// Set a configuration value by key
    ///
    /// NOTE: This only updates the in-memory struct. You need to call `save()` to persist it to git config.
    pub fn set(&mut self, key: ConfigKey, value: String) {
        match key {
            ConfigKey::Product => self.product_branch = value,
            ConfigKey::Develop => self.develop_branch = value,
            ConfigKey::Feature => self.feature_prefix = value,
            ConfigKey::Release => self.release_prefix = value,
            ConfigKey::Bugfix => self.bugfix_prefix = value,
            ConfigKey::Support => self.support_prefix = value,
            ConfigKey::Versiontag => self.versiontag_prefix = value,
            ConfigKey::Version => self.project_version = value,
        }
    }
}

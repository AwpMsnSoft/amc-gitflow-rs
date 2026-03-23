use anyhow::Result as AnyResult;
use clap::ValueEnum;
use colored::Colorize;
use lazy_static::lazy_static;
use std::collections::HashMap;
use velvetio::prelude::*;

use crate::core::git;

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum ConfigKey {
    Product,
    Develop,
    Feature,
    Release,
    Bugfix,
    Support,
    Versiontag,
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
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigKey::Product => "product",
            ConfigKey::Develop => "develop",
            ConfigKey::Feature => "feature",
            ConfigKey::Release => "release",
            ConfigKey::Bugfix => "bugfix",
            ConfigKey::Support => "support",
            ConfigKey::Versiontag => "versiontag",
        }
    }
}

/// Gitflow configuration structure
pub struct GitflowConfig {
    pub product_branch: String,
    pub develop_branch: String,
    pub feature_prefix: String,
    pub release_prefix: String,
    pub bugfix_prefix: String,
    pub support_prefix: String,
    pub versiontag_prefix: String,
}

lazy_static! {
    pub static ref CONFIG_DESCRIPTIONS: HashMap<String, &'static str> = {
        let mut m = HashMap::new();
        m.insert("product".to_string(), "branch name for production releases");
        m.insert(
            "develop".to_string(),
            "branch name for \"next release\" development",
        );
        m.insert("feature".to_string(), "prefix for feature branches");
        m.insert("release".to_string(), "prefix for release branches");
        m.insert("bugfix".to_string(), "prefix for bugfix branches");
        m.insert("support".to_string(), "prefix for support branches");
        m.insert("versiontag".to_string(), "prefix for version tags");
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
        }
    }

    /// Create a new GitflowConfig with console inputs
    pub fn new() -> Self {
        macro_rules! bold {
            ($args: expr) => {{ $args.bold().to_string() }};
        }

        let product_branch = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get("product").unwrap()),
            default: "master".to_string()
        );
        let develop_branch = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get("develop").unwrap()),
            default: "develop".to_string()
        );
        let feature_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get("feature").unwrap()),
            default: "feature/".to_string()
        );
        let release_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get("release").unwrap()),
            default: "release/".to_string()
        );
        let bugfix_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get("bugfix").unwrap()),
            default: "bugfix/".to_string()
        );
        let support_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get("support").unwrap()),
            default: "support/".to_string()
        );
        let versiontag_prefix = ask!(
            &bold!(CONFIG_DESCRIPTIONS.get("versiontag").unwrap()),
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
        Ok(())
    }

    /// Get a configuration value by key
    pub fn get(&self, key: &str) -> String {
        match key {
            "product" => self.product_branch.clone(),
            "develop" => self.develop_branch.clone(),
            "feature" => self.feature_prefix.clone(),
            "release" => self.release_prefix.clone(),
            "bugfix" => self.bugfix_prefix.clone(),
            "support" => self.support_prefix.clone(),
            "versiontag" => self.versiontag_prefix.clone(),
            _ => "".to_string(),
        }
    }

    /// Set a configuration value by key
    /// 
    /// NOTE: This only updates the in-memory struct. You need to call `save()` to persist it to git config.
    pub fn set(&mut self, key: &str, value: String) {
        match key {
            "product" => self.product_branch = value,
            "develop" => self.develop_branch = value,
            "feature" => self.feature_prefix = value,
            "release" => self.release_prefix = value,
            "bugfix" => self.bugfix_prefix = value,
            "support" => self.support_prefix = value,
            "versiontag" => self.versiontag_prefix = value,
            _ => {}
        }
    }
}

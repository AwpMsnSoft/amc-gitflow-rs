#![allow(unused)]

use anyhow::{Result as AnyResult, bail};
use regex::Regex;

use crate::utils::run::run;

/// Check if gh command is installed
pub fn is_installed() -> bool {
    run("gh", &["--version"]).is_ok()
}

pub mod auth {
    use super::*;

    /// Check if gh is authenticated
    pub fn is_authenticated() -> bool {
        match run("gh", &["auth", "status"]) {
            Ok(output) => output.contains("Logged in to github.com"),
            Err(_) => false,
        }
    }

    /// Prompt user to login with gh
    pub fn login() -> AnyResult<String> {
        run("gh", &["auth", "login"])
    }

    /// Username of the authenticated user
    pub fn username() -> AnyResult<String> {
        let output = run("gh", &["auth", "status"])?;

        // use regex to parse: ...... Logged in to github.com account <$USER> ......
        let re = Regex::new(r"Logged in to github.com account (\S+)").unwrap();
        if let Some(caps) = re.captures(&output) {
            if let Some(user) = caps.get(1) {
                return Ok(user.as_str().to_string());
            }
        }

        bail!(
            "Could not determine authenticated username from gh auth status output. Is gh authenticated?"
        );
    }
}

pub mod pr {
    use super::*;

    /// Create a pull request
    pub fn create(title: &str, body: &str, base: &str, head: &str) -> AnyResult<String> {
        run(
            "gh",
            &[
                "pr", "create", "--title", title, "--body", body, "--base", base, "--head", head,
            ],
        )
    }

    /// List pull requests
    pub fn list(state: &str) -> AnyResult<String> {
        run("gh", &["pr", "list", "--state", state])
    }

    /// View pull request details
    pub fn view(number: &str) -> AnyResult<String> {
        run("gh", &["pr", "view", number])
    }

    /// Merge a pull request
    pub fn merge(number: &str, method: &str) -> AnyResult<String> {
        run("gh", &["pr", "merge", number, "--", method])
    }

    /// Check pull request status
    pub fn status() -> AnyResult<String> {
        run("gh", &["pr", "status"])
    }
}

pub mod repo {
    use super::*;

    /// Repository information
    pub fn view() -> AnyResult<String> {
        run("gh", &["repo", "view"])
    }

    /// Set default repository for current directory
    pub fn set_default(repo: &str) -> AnyResult<String> {
        run("gh", &["repo", "set-default", repo])
    }

    /// Create remote repository
    pub fn create(name: &str, is_public: bool, owner: &str) -> AnyResult<String> {
        let mut args = vec!["repo", "create"];
        let target = format!("{}/{}", owner, name);
        args.push(&target);

        args.push(if is_public { "--public" } else { "--private" });
        args.append(&mut vec!["--source=.", "--remote=origin"]);

        run("gh", &args)
    }
}

pub mod release {
    use super::*;

    /// Create a release
    pub fn create(tag: &str, title: &str, notes: &str) -> AnyResult<String> {
        run(
            "gh",
            &["release", "create", tag, "--title", title, "--notes", notes],
        )
    }
}

use anyhow::Result as AnyResult;

use crate::utils::run::run;

/// Check if gh command is installed
pub fn is_installed() -> bool {
    run("gh", &["--version"]).is_ok()
}

pub mod auth {
    use super::*;

    /// Check if gh is authenticated
    pub fn is_authenticated() -> bool {
        run("gh", &["auth", "status"]).is_ok()
    }

    /// Prompt user to login with gh
    pub fn login() -> AnyResult<String> {
        run("gh", &["auth", "login"])
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

#![allow(unused)]

use anyhow::{Result as AnyResult, bail};
use regex::Regex;
use try_map::FlipResultExt;

use crate::utils::run::{edit_in_editor, run};

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

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Status {
        pub number: String,
        pub title: String,
        pub branch: String,
        pub state: String,
    }

    /// Create a pull request.
    /// If `body` is None, opens $EDITOR for interactive body input.
    pub fn create(title: &str, body: &str, base: &str, head: &str) -> AnyResult<String> {
        let body = edit_in_editor(body)?;
        run(
            "gh",
            &[
                "pr", "create", "--title", title, "--body", &body, "--base", base, "--head", head,
            ],
        )
    }

    /// List pull requests with given state (open, closed, merged, all)
    pub fn list(state: &str) -> AnyResult<Vec<Status>> {
        run("gh", &["pr", "list", "--state", state])?
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts = line.split('\t').collect::<Vec<_>>();
                if parts.len() < 4 {
                    bail!("Unexpected line format from `gh pr list`: '{}'", line);
                }
                Ok(Status {
                    number: parts[0].to_string(),
                    title: parts[1].to_string(),
                    branch: parts[2].to_string(),
                    state: parts[3].to_string(),
                })
            })
            .collect::<Vec<_>>()
            .flip()
    }

    /// View pull request details
    pub fn view(number: &str) -> AnyResult<String> {
        run("gh", &["pr", "view", number])
    }

    /// Check if a pull request (by number) has been merged.
    /// Returns Ok(true) if merged, Ok(false) if not yet merged.
    pub fn is_merged(number: &str) -> AnyResult<bool> {
        let output = run(
            "gh",
            &["pr", "view", number, "--json", "state", "--jq", ".state"],
        )?;
        Ok(output.trim().eq_ignore_ascii_case("merged"))
    }

    /// Get the URL of a pull request by number
    pub fn url(number: &str) -> AnyResult<String> {
        run(
            "gh",
            &["pr", "view", number, "--json", "url", "--jq", ".url"],
        )
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

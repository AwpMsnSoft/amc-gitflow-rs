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
    pub fn create(
        title: &str,
        body: &str,
        base: &str,
        head: &str,
        labels: Option<&[&str]>,
    ) -> AnyResult<String> {
        let body = edit_in_editor(body)?;
        let mut args = vec![
            "pr", "create", "--title", title, "--body", &body, "--base", base, "--head", head,
        ];

        if let Some(labels) = labels {
            for label in labels {
                args.push("--label");
                args.push(label);
            }
        }

        run("gh", &args)
    }

    /// List pull requests with given state (open, closed, merged, all)
    pub fn list(state: &str) -> AnyResult<Vec<Status>> {
        run("gh", &["pr", "list", "--state", state])?
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts = line.split('\t').collect::<Vec<_>>();
                if parts.len() < 4 {
                    bail!("Unexpected line format from `gh pr list`: '{line}'");
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

pub mod issue {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct State {
        pub number: String,
        pub state: String,
        pub title: String,
        pub tags: String,
    }

    /// List open issues
    pub fn list() -> AnyResult<Vec<State>> {
        run("gh", &["issue", "list", "--state", "open"])?
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() < 4 {
                    bail!("Unexpected line format from `gh issue list`: '{line}'");
                }
                Ok(State {
                    number: parts[0].to_string(),
                    state: parts[1].to_string(),
                    title: parts[2].to_string(),
                    tags: parts[3].to_string(),
                })
            })
            .collect::<Vec<_>>()
            .flip()
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
    pub fn create(name: &str, remote: &str, is_public: bool, owner: &str) -> AnyResult<String> {
        let mut args = vec!["repo", "create"];
        let target = format!("{owner}/{name}");
        args.push(&target);

        let remote_arg = format!("--remote={remote}");
        args.push(if is_public { "--public" } else { "--private" });
        args.append(&mut vec!["--source=.", &remote_arg]);

        run("gh", &args)
    }
}

pub mod release {
    use super::*;

    /// Create a release with explicit notes
    pub fn create(tag: &str, title: &str, notes: Option<String>) -> AnyResult<String> {
        let mut args = vec!["release", "create", tag, "--title", title];

        if let Some(notes) = notes {
            args.push("--notes");
            args.push(&notes);
            run("gh", &args)
        } else {
            args.push("--generate-notes");
            run("gh", &args)
        }
    }

    /// Generate release notes draft (without creating the release).
    /// Uses `gh api` to call the generate-release-notes endpoint.
    /// Returns the auto-generated markdown body.
    pub fn generate_notes(
        tag: &str,
        target: &str,
        previous_tag: Option<&str>,
    ) -> AnyResult<String> {
        let mut args = vec!["api", "repos/{owner}/{repo}/releases/generate-notes", "-f"];
        let tag_field = format!("tag_name={tag}");
        args.push(&tag_field);
        args.push("-f");
        let target_field = format!("target_commitish={target}");
        args.push(&target_field);
        if let Some(prev) = previous_tag {
            args.push("-f");
            let prev_field = format!("previous_tag_name={prev}");
            args.push(&prev_field);
            let output = run("gh", &args)?;
            // The API returns JSON: { "name": "...", "body": "..." }
            // Extract the body field
            extract_body_from_json(&output)
        } else {
            let output = run("gh", &args)?;
            extract_body_from_json(&output)
        }
    }

    fn extract_body_from_json(json: &str) -> AnyResult<String> {
        // Minimal JSON extraction for the "body" field
        let re = Regex::new(r#""body"\s*:\s*"((?:[^"\\]|\\.)*)""#).unwrap();
        if let Some(caps) = re.captures(json) {
            if let Some(body) = caps.get(1) {
                let unescaped = body
                    .as_str()
                    .replace("\\n", "\n")
                    .replace("\\r", "")
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\");
                return Ok(unescaped);
            }
        }
        // Fallback: return the raw output
        Ok(json.to_string())
    }
}

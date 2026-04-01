#![allow(unused)]

use anyhow::Result as AnyResult;

use crate::utils::run::run;

/// Check if git command is installed
pub fn is_installed() -> bool {
    run("git", &["--version"]).is_ok()
}

pub mod repo {
    use super::*;

    /// Initialize a new git repository
    pub fn init() -> AnyResult<String> {
        run("git", &["init"])
    }

    /// Get the name of the current git repository
    pub fn name() -> AnyResult<String> {
        let top_level = run("git", &["rev-parse", "--show-toplevel"])?;
        run("basename", &[&top_level])
    }

    /// Check if the current directory is a git repository
    pub fn is_repository() -> bool {
        match run("git", &["status"]) {
            Ok(_) => true,
            Err(err) => !err.to_string().contains("fatal: not a git repository"),
        }
    }
}

pub mod commit {
    use super::*;

    /// Create an initial empty commit
    pub fn init() -> AnyResult<String> {
        run(
            "git",
            &[
                "commit",
                "--allow-empty",
                "-m",
                "init(all): Workspace with initial commit",
            ],
        )
    }
}

pub mod branch {
    use super::*;

    /// Get the current branch name
    pub fn current() -> AnyResult<String> {
        run("git", &["rev-parse", "--abbrev-ref", "HEAD"])
    }

    /// Check if a branch exists
    pub fn exists(name: &str) -> AnyResult<bool> {
        let output = run("git", &["branch", "--list", name])?;
        Ok(!output.is_empty())
    }

    /// Create and checkout a new branch
    pub fn create(name: &str, base: &str) -> AnyResult<String> {
        run("git", &["checkout", "-b", name, base])
    }

    /// Delete a branch
    pub fn delete(name: &str, force: bool) -> AnyResult<String> {
        let flag = if force { "-D" } else { "-d" };
        run("git", &["branch", flag, name])
    }

    /// Delete a branch on a remote
    pub fn delete_remote(remote: &str, name: &str) -> AnyResult<String> {
        run("git", &["push", remote, "--delete", name])
    }

    /// List local branches
    pub fn list() -> AnyResult<Vec<String>> {
        let output = run("git", &["branch", "--format=%(refname:short)"])?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }
}

pub mod checkout {
    use super::*;

    /// Checkout a branch
    pub fn branch(name: &str) -> AnyResult<String> {
        run("git", &["checkout", name])
    }
}

pub mod merge {
    use super::*;

    /// Merge a branch with fast-forward    
    pub fn fast_forward(name: &str) -> AnyResult<String> {
        run("git", &["merge", "--ff", name])
    }

    /// Merge a branch without fast-forward
    pub fn no_fast_forward(name: &str) -> AnyResult<String> {
        run("git", &["merge", "--no-ff", name])
    }

    /// Merge a branch with squash
    pub fn squash(name: &str) -> AnyResult<String> {
        run("git", &["merge", "--squash", name])
    }
}

pub mod config {
    use super::*;

    /// Get git config value
    pub fn get(key: &str) -> AnyResult<String> {
        run("git", &["config", "--get", key])
    }

    /// Set git config value
    pub fn set(key: &str, value: &str) -> AnyResult<String> {
        run("git", &["config", "set", key, value])
    }

    /// Unset git config value
    pub fn unset(key: &str) -> AnyResult<String> {
        run("git", &["config", "--unset", key])
    }
}

pub mod remote {
    use super::*;

    /// Check if there are any remotes
    pub fn has_remotes() -> AnyResult<bool> {
        let output = run("git", &["remote"])?;
        Ok(!output.trim().is_empty())
    }

    /// Get list of remotes
    pub fn list() -> AnyResult<Vec<String>> {
        let output = run("git", &["remote"])?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    /// Push a branch to remote
    pub fn push(remote: &str, branch: &str) -> AnyResult<String> {
        run("git", &["push", remote, branch])
    }

    /// Push a branch to remote and set upstream tracking
    pub fn push_upstream(remote: &str, branch: &str) -> AnyResult<String> {
        run("git", &["push", "-u", remote, branch])
    }

    /// Pull from remote
    pub fn pull(remote: &str, branch: &str) -> AnyResult<String> {
        run("git", &["pull", remote, branch])
    }

    /// Fetch from remote
    pub fn fetch(remote: &str) -> AnyResult<String> {
        run("git", &["fetch", remote])
    }

    /// Check if a branch exists on a remote
    pub fn branch_exists(remote: &str, branch: &str) -> AnyResult<bool> {
        let output = run("git", &["ls-remote", "--heads", remote, branch])?;
        Ok(!output.is_empty())
    }
}

pub mod status {
    use super::*;

    /// Check if the working directory is clean
    pub fn is_clean() -> AnyResult<bool> {
        let output = run("git", &["status", "--porcelain"])?;
        Ok(output.is_empty())
    }
}

pub mod tag {
    use super::*;

    /// Check if a tag exists
    pub fn exists(name: &str) -> AnyResult<bool> {
        let output = run("git", &["tag", "--list", name])?;
        Ok(!output.is_empty())
    }

    /// Tag a commit
    pub fn create(name: &str, message: &str) -> AnyResult<String> {
        run("git", &["tag", "-a", name, "-m", message])
    }
}

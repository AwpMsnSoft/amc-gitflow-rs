# amc-gitflow-rs

A Gitflow implementation in Rust, integrated with GitHub CLI (`gh`) to automate Pull Request and Release workflows. This tool follows the standard Gitflow methodology but shifts the branch merging and lifecycle management to GitHub’s PR system for better collaboration and audit trails.

## Prerequisites

- **Git**: Installed and configured.
- **GitHub CLI (`gh`)**: Installed and authenticated (`gh auth login`).
- **Rust**: To build from source (`cargo install`).

## GitHub Repository Setup

To ensure all automated features (especially for Release and Bugfix) work correctly, configure your repository as follows:

1.  **Labels**:
    *   Create a label named `release` (used to categorize release PRs).
    *   Create a label named `bugfix` (used for bugfix PRs).
    *   Create a label named `feature` (used for feature PRs).
2.  **Branch Protection Rules**:
    *   It is recommended to protect your `master`/`main` and `develop` branches.
    *   Enable "Require a pull request before merging".
3.  **Merge Settings**:
    *   The tool works best if **"Allow squash merging"** or **"Allow rebase merging"** is enabled on GitHub to keep a clean history.
4.  **Workflow Permissions**:
    *   The Token used by `gh` must have `repo` and `workflow` scopes to allow the tool to create PRs, create Tags, and publish GitHub Releases.

## Installation

```bash
# Clone the repository and install
git clone https://github.com/your-repo/amc-gitflow-rs.git
cd amc-gitflow-rs
cargo install --path .
```

## Workflows

### 1. Initialization
Set up branch names and prefixes for the current repository:
```bash
amc-gitflow-rs init
```

### 2. Feature Workflow
Used for developing new features.
- **Start**: `amc-gitflow-rs feature start <name>` - Creates a branch from `develop`.
- **Publish**: `amc-gitflow-rs feature publish` - Pushes the branch and creates a GitHub PR into `develop`.
- **Finish**: `amc-gitflow-rs feature finish` - Checks if PR is merged, pulls changes, and deletes the local branch.

### 3. Release Workflow
Used for preparing a production release.
- **Start**: `amc-gitflow-rs release start <version>` - Creates a branch from `develop`.
- **Publish**: `amc-gitflow-rs release publish` - Pushes to remotes and creates a GitHub PR into `master`. It pre-fills a comprehensive release checklist and changelog template.
- **Finish**: `amc-gitflow-rs release finish [--auto]`
    - **Step 1**: Verifies the PR is merged on GitHub.
    - **Step 2**: Syncs `master`, creates a Git Tag, and pushes to all remotes.
    - **Step 3**: Creates a **GitHub Release** (interactive editor or auto-generated with `--auto`).
    - **Step 4**: Back-merges `master` into `develop` to keep them in sync.
    - **Step 5**: Deletes the release branch locally and on all remotes.
    - **Note**: `--auto` flag skips all prompts and automatically bumps the project patch version.

### 4. Bugfix Workflow
Integrated with GitHub Issues.
- **Start**: `amc-gitflow-rs bugfix start` - Interactively lists open GitHub Issues for you to pick. Creates a branch named after the issue.
- **Publish**: `amc-gitflow-rs bugfix publish` - Creates a PR into `develop` that automatically "Closes #IssueID".
- **Finish**: `amc-gitflow-rs bugfix finish` - Finalizes the branch after the fix is merged.

## Configuration

Settings are stored in the local `.git/config` of your project.

| Key | Description | Default |
|-----|-------------|---------|
| `amc-gitflow-rs.branch.product` | Main production branch | `master` |
| `amc-gitflow-rs.branch.develop` | Main development branch | `develop` |
| `amc-gitflow-rs.prefix.feature` | Prefix for feature branches | `feature/` |
| `amc-gitflow-rs.prefix.release` | Prefix for release branches | `release/` |
| `amc-gitflow-rs.prefix.versiontag` | Prefix for git tags | (empty) |

Manage them via terminal:
```bash
amc-gitflow-rs config list
amc-gitflow-rs config set <key> <value>
```

## License

MIT / Apache 2.0


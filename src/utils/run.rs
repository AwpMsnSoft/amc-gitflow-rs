#![allow(unused)]

use std::{
    fs::File,
    io::{Read, Write},
    process::{Command, Stdio},
};

use anyhow::{Context, Result as AnyResult, bail};
use auto_context::auto_context as anyhow_context;
use duct::cmd;

use crate::{debug, error};

#[anyhow_context]
pub fn run(command: &str, args: &[&str]) -> AnyResult<String> {
    let output = match cmd(command, args)
        .stderr_capture()
        .stdout_capture()
        .unchecked()
        .run()
    {
        Ok(output) => output,
        Err(e) => {
            error!(
                "Failed to execute command `{command}` with args {args:?}"
            );
            bail!(e);
        }
    };

    debug!(
        "Command `{command}` executed with args {args:?}: {output:?}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        let error_msg = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            let code = output.status.code().unwrap_or(-1);
            format!("exited with code {code}")
        };
        error!("Command `{command}` failed with args {args:?}");
        bail!(error_msg)
    }
}

/// Run a command interactively, inheriting stdin/stdout/stderr.
/// Used for commands that need user interaction (e.g. opening an editor).
#[anyhow_context]
pub fn run_interactive(command: &str, args: &[&str]) -> AnyResult<()> {
    let status = Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        bail!(
            "Command `{command}` exited with code {code}"
        );
    }
    Ok(())
}

/// Open the user's $EDITOR to edit text interactively.
/// `initial_content` is pre-populated in the temp file.
/// Returns the edited content.
#[anyhow_context]
pub fn edit_in_editor(initial_content: &str) -> AnyResult<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile()?;
    tmp.write_all(initial_content.as_bytes())?;
    tmp.flush()?;

    let path = tmp.path().to_path_buf();
    run_interactive(&editor, &[path.to_str().unwrap()])?;

    let mut content = String::new();
    File::open(&path)?.read_to_string(&mut content)?;

    // Strip trailing whitespace
    let content = content.trim().to_string();
    if content.is_empty() {
        bail!("Aborted: empty content after editing");
    }
    Ok(content)
}

#[anyhow_context]
pub fn run_uncheck(command: &str, args: &[&str]) -> AnyResult<()> {
    let output = cmd(command, args).run()?;

    debug!(
        "Command `{command}` executed with args {args:?}: {output:?}"
    );
    Ok(())
}

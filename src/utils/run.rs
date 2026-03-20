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
                "Failed to execute command `{}` with args {:?}",
                command, args
            );
            bail!(e);
        }
    };

    debug!(
        "Command `{}` executed with args {:?}: {:?}",
        command, args, output
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
            format!("exited with code {}", output.status.code().unwrap_or(-1))
        };
        error!("Command `{}` failed with args {:?}", command, args);
        bail!(error_msg)
    }
}

#[anyhow_context]
pub fn run_uncheck(command: &str, args: &[&str]) -> AnyResult<()> {
    let output = cmd(command, args).run()?;

    debug!(
        "Command `{}` executed with args {:?}: {:?}",
        command, args, output
    );
    Ok(())
}

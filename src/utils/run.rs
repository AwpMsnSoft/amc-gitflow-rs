use anyhow::{Context, Result as AnyResult, bail};
use auto_context::auto_context as anyhow_context;
use duct::cmd;

use crate::debug;

#[anyhow_context]
pub fn run(command: &str, args: &[&str]) -> AnyResult<String> {
    let output = cmd(command, args).stderr_capture().stdout_capture().run()?;
    debug!(
        "Command `{}` executed with args {:?}: {:?}",
        command, args, output
    );

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        bail!(
            "Command `{}` failed with args {:?}: {}",
            command,
            args,
            String::from_utf8(output.stderr)?
        )
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

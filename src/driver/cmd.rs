use std::process::{Command, Output};
use std::path::Path;

use crate::error::{wrap, Error};

/// Execute a command from a string.
pub fn execute<P: AsRef<Path>>(cmd: &str, dir: P) -> Result<Output, Error> {
    let mut process = Command::new("sh");
    process.arg("-c").arg(cmd).current_dir(dir);
    let output = wrap(process.output(), "Could not run command")?;
    Ok(output)
}

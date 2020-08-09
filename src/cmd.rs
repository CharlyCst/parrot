use std::process::{ Command, Output};

use crate::error::{wrap, Error};

/// Execute a command from a string.
pub fn execute(cmd: &str) -> Result<Output, Error> {
    let mut process = Command::new("sh");
    process.arg("-c").arg(cmd);
    let output = wrap(process.output(), "Could not run command")?;
    Ok(output)
}

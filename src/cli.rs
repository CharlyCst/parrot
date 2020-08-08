extern crate clap;

use clap::Clap;
use std::path::PathBuf;

/// A CLI snapshot tool.
#[derive(Clap)]
#[clap(version = "0.1.0")]
pub struct Config {
    #[clap(subcommand)]
    pub cmd: Command,

    /// Base path.
    #[clap(default_value = ".", parse(from_os_str))]
    pub path: PathBuf,
}

#[derive(Clap)]
pub enum Command {
     /// Add a new snapshot for the given command.
    Add {
        /// The command to execute.
        cmd: String,

        /// Optional name for the snapshot.
        name: Option<String>,
    },

    /// Initialize Parrot.
    Init {},

    /// Run snapshot tests.
    Run {},
}

/// Parse CLI args, may terminate the program.
pub fn parse() -> Config {
    Config::parse()
}


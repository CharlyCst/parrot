extern crate clap;

use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "0.0.2")]
#[clap(verbatim_doc_comment)]
/// A colorful and chatty CLI snapshot tool.
/// |
/// |      (o>
/// |______(()___
/// |      ||
/// |
pub struct Config {
    #[clap(subcommand)]
    pub cmd: Option<Command>,

    /// Base path
    #[clap(short, long, default_value = ".", parse(from_os_str))]
    pub path: PathBuf,

    /// Verbode mode
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Clap)]
pub enum Command {
    /// Add a new snapshot for the given command
    Add {
        /// The command to execute
        cmd: String,

        /// Optional name for the snapshot
        #[clap(short, long)]
        name: Option<String>,

        /// Accept the snapshot
        #[clap(short, long)]
        yes: bool,
    },

    /// Execute a script
    Exec { cmd: String },

    /// Initialize Parrot
    Init {},

    /// Run snapshot tests
    Run {},
}

/// Parse CLI args, may terminate the program
pub fn parse() -> Config {
    Config::parse()
}

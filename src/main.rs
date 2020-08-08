use cli::Command;
use std::path::PathBuf;

mod cli;
mod data;

fn main() {
    let config = cli::parse();
    match config.cmd {
        Command::Init {} => {
            data::initialize(PathBuf::from(config.path)).unwrap();
        }
        Command::Add { .. } => (),
        Command::Run {} => (),
    }
}


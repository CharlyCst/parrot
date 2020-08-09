use cli::Command;
use error::unwrap_log;

mod cli;
mod data;
mod error;

fn main() {
    let config = cli::parse();
    let data = unwrap_log(data::DataManager::new(config.path));
    match config.cmd {
        Command::Init {} => {
            unwrap_log(data.initialize());
            println!("Parrot has been initialized.")
        }
        Command::Add { .. } => (),
        Command::Run {} => (),
    }
}


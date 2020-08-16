use cli::Command;
use error::unwrap_log;

mod cli;
mod data;
mod diff;
mod driver;
mod editor;
mod error;
mod term;

fn main() {
    let config = cli::parse();
    match config.cmd {
        Command::Init {} => {
            let mut data = unwrap_log(data::DataManager::new(config.path));
            unwrap_log(data.initialize());
            println!("Parrot has been initialized.")
        }
        Command::Add {
            ref cmd,
            ref name,
            yes,
        } => driver::add(cmd, name, yes, config.path),
        Command::Run {} => driver::run(config.path),
    }
}

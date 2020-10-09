use cli::Command;
use std::process::exit;
use error::Log;

mod cli;
mod data;
mod diff;
mod driver;
mod editor;
mod error;
mod term;
mod parser;

fn main() {
    let config = cli::parse();
    let mut context = driver::Context::new(config.path).unwrap_log();
    match config.cmd {
        Some(Command::Init {}) => {
            context.init();
        }
        Some(Command::Add {
            ref cmd,
            ref name,
            yes,
        }) => context.add(cmd, name, yes),
        Some(Command::Run {}) => {
            if context.run() {
                exit(0);
            } else {
                exit(1);
            }
        }
        None => {
            context.repl();
        }
    }
}

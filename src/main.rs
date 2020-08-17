use cli::Command;
use std::process::exit;
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
    let mut context = unwrap_log(driver::Context::new(config.path));
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

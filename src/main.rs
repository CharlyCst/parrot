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
    let mut context = unwrap_log(driver::Context::new(config.path));
    match config.cmd {
        Command::Init {} => {
            context.init();
        }
        Command::Add {
            ref cmd,
            ref name,
            yes,
        } => context.add(cmd, name, yes),
        Command::Run {} => context.run(),
    }
}

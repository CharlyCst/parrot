use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::path::PathBuf;

use cli::Command;
use error::unwrap_log;

mod cli;
mod cmd;
mod data;
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
        } => add(cmd, name, yes, config.path),
        Command::Run {} => run(config.path),
    }
}

/// Handle add subcomand.
fn add(cmd: &str, name: &Option<String>, yes: bool, path: PathBuf) {
    let mut data = unwrap_log(data::DataManager::new(path));
    let snap = unwrap_log(cmd::execute(&cmd));
    let save = if yes {
        true
    } else {
        term::color_box("Snapshot", &snap.stdout);
        unwrap_log(term::binary_qestion("Save this snapshot?"))
    };
    if save {
        // Get snapshot name
        let name = if let Some(name) = name {
            name.to_owned()
        } else {
            if yes {
                get_random_name()
            } else {
                let mut name =
                    unwrap_log(term::string_question("Snapshot name? (empty for random):"));
                name = normalize_name(&name);
                if name.len() == 0 {
                    get_random_name()
                } else {
                    name
                }
            }
        };
        unwrap_log(data.add_snapshot(&cmd, &name, &snap.stdout));
    }
}

/// Handle run subcomand.
fn run(path: PathBuf) {
    let mut data = unwrap_log(data::DataManager::new(path));
    let mut success = true;
    let snaps = unwrap_log(data.get_all_snapshots());
    for snap in &snaps {
        let result = unwrap_log(cmd::execute(&snap.cmd));
        if result.stdout != snap.content {
            success = false;
            println!("Test failed for '{}'.", &snap.cmd);
        }
    }
    if success {
        println!("Success !");
    } else {
        println!("Failure...");
    }
}

/// Normalize a string for use a file name.
fn normalize_name(name: &str) -> String {
    name.trim().replace(' ', "_").replace('\t', "_")
}

fn get_random_name() -> String {
    let mut random_name = String::from("_");
    random_name.extend(thread_rng().sample_iter(&Alphanumeric).take(30));
    random_name
}

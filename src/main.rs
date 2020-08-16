use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::path::PathBuf;
use std::process::Output;

use cli::Command;
use data::{Snapshot, SnapshotData};
use error::unwrap_log;

mod cli;
mod cmd;
mod data;
mod diff;
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
        } => add(cmd, name, yes, config.path),
        Command::Run {} => run(config.path),
    }
}

/// Handles add subcomand.
fn add(cmd: &str, name: &Option<String>, yes: bool, path: PathBuf) {
    let mut data = unwrap_log(data::DataManager::new(&path));
    let snap = unwrap_log(cmd::execute(&cmd));
    let save = if yes {
        true
    } else {
        term::color_box("Snapshot", &snap.stdout);
        unwrap_log(term::binary_qestion("Save this snapshot?"))
    };
    if save {
        // Get snapshot name
        let mut description = None;
        let mut tags = Vec::new();
        let name = if let Some(name) = name {
            name.to_owned()
        } else {
            if yes {
                get_random_name()
            } else {
                let edit_result = unwrap_log(editor::open_empty(&path, cmd));
                description = edit_result.description;
                tags = edit_result.tags;
                if let Some(name) = edit_result.name {
                    normalize_name(&name)
                } else {
                    get_random_name()
                }
            }
        };
        let snapshot = to_snapshot(name, description, tags, cmd.to_owned(), snap);
        unwrap_log(data.add_snapshot(&snapshot));
    }
}

/// Handles run subcomand.
fn run(path: PathBuf) {
    let mut data = unwrap_log(data::DataManager::new(path));
    let mut success = true;
    let snaps = unwrap_log(data.get_all_snapshots());
    for snap in &snaps { // TODO: handle stderr and exit_code
        let result = unwrap_log(cmd::execute(&snap.cmd));
        if &result.stdout != &snap.stdout.as_ref().unwrap().body {
            success = false;
            println!("Test {} failed.", &snap.name);
        }
    }
    if success {
        println!("Success !");
    } else {
        println!("Failure...");
    }
}

/// Creates a snapshot out of an execution result
fn to_snapshot(
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    cmd: String,
    snap: Output,
) -> Snapshot {
    let exit_code = snap.status.code();
    let stdout = to_snapshot_data(snap.stdout, &name, ".out");
    let stderr = to_snapshot_data(snap.stderr, &name, ".err");
    Snapshot {
        cmd,
        name,
        description,
        tags,
        exit_code,
        stdout,
        stderr,
    }
}

/// Creates a snapshot_data item from raw body.
fn to_snapshot_data(body: Vec<u8>, path: &str, path_extension: &str) -> Option<SnapshotData> {
    if body.len() > 0 {
        let mut path = path.to_owned();
        path.push_str(path_extension);
        Some(SnapshotData { body, path })
    } else {
        None
    }
}

/// Normalizes a string for use a file name.
fn normalize_name(name: &str) -> String {
    name.trim().replace(' ', "_").replace('\t', "_")
}

/// Generates a random name starting with '_'.
fn get_random_name() -> String {
    let mut random_name = String::from("_");
    random_name.extend(thread_rng().sample_iter(&Alphanumeric).take(30));
    random_name
}

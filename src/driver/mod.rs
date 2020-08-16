use std::io::stdout;
use std::path::PathBuf;

use crate::data::{Snapshot, SnapshotData, DataManager};
use crate::error::unwrap_log;
use crate::term;
use crate::editor;

use util::*;

mod util;
mod cmd;

/// Handles add subcomand.
pub fn add(cmd: &str, name: &Option<String>, yes: bool, path: PathBuf) {
    let mut data = unwrap_log(DataManager::new(&path));
    let snap = unwrap_log(cmd::execute(&cmd));
    let save = if yes {
        true
    } else {
        term::color_box("Snapshot", &snap.stdout, &mut stdout());
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
pub fn run(path: PathBuf) {
    let mut data = unwrap_log(DataManager::new(path));
    let mut success = true;
    let mut stdout = stdout();
    let empty_body = Vec::new();
    let snaps = unwrap_log(data.get_all_snapshots());
    for snap in &snaps {
        let mut close_separator = false;
        let result = unwrap_log(cmd::execute(&snap.cmd));
        let old_stdout = if let Some(ref stdout) = snap.stdout {
            &stdout.body
        } else {
            &empty_body
        };
        let old_stderr = if let Some(ref stderr) = snap.stderr {
            &stderr.body
        } else {
            &empty_body
        };
        if &result.stdout != old_stdout {
            close_separator = true;
            term::title_separator("stdout", &mut stdout);
            term::write_diff(old_stdout, &result.stdout, &mut stdout);
            success = false;
        }
        if &result.stderr != old_stderr {
            close_separator = true;
            term::title_separator("stderr", &mut stdout);
            term::write_diff(old_stderr, &result.stderr, &mut stdout);
        }
        if close_separator {
            term::separator(6, &mut stdout);
        }
        println!("Test {} failed.", &snap.name);
    }
    if success {
        println!("Success !");
    } else {
        println!("Failure...");
    }
}

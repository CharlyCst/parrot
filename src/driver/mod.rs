use std::io::stdout;
use std::path::PathBuf;

use crate::data::{DataManager, Snapshot, SnapshotData};
use crate::editor;
use crate::error::{unwrap_log, Error};
use crate::term;

use util::*;

mod cmd;
mod util;

pub struct Context {
    path: PathBuf,
    data: DataManager,
}

impl Context {
    /// Creates a new context.
    pub fn new(path: PathBuf) -> Result<Context, Error> {
        let data = DataManager::new(&path)?;
        Ok(Context { path, data })
    }

    /// Handles init subcommand.
    pub fn init(&mut self) {
        unwrap_log(self.data.initialize());
        println!("Parrot has been initialized.")
    }

    /// Handles add subcommand.
    pub fn add(&mut self, cmd: &str, name: &Option<String>, yes: bool) {
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
                    let edit_result = unwrap_log(editor::open_empty(&self.path, cmd));
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
            unwrap_log(self.data.add_snapshot(&snapshot));
        }
    }

    /// Handles run subcommand.
    /// Returns true in case of success, false otherwise.
    pub fn run(&mut self) -> bool {
        let mut success = true;
        let mut stdout = stdout();
        let empty_body = Vec::new();
        let snaps = unwrap_log(self.data.get_all_snapshots());
        for snap in &snaps {
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
            let stdout_eq = &result.stdout == old_stdout;
            let stderr_eq = &result.stderr == old_stderr;
            let code_eq = snap.exit_code == result.status.code();
            let failed = !stdout_eq || !stderr_eq || !code_eq;
            // Draw test summary
            if failed {
                term::title_separator("info", 2, &mut stdout);
                term::snap_summary(&snap.name, snap.description.as_ref(), &snap.cmd, &mut stdout);
            }
            if &result.stdout != old_stdout {
                term::title_separator("stdout", 0, &mut stdout);
                term::write_diff(old_stdout, &result.stdout, &mut stdout);
                success = false;
            }
            if &result.stderr != old_stderr {
                term::title_separator("stderr", 0, &mut stdout);
                term::write_diff(old_stderr, &result.stderr, &mut stdout);
            }
            if failed {
                term::separator(6, &mut stdout);
            }
        }
        if success {
            term::success(&mut stdout);
            true
        } else {
            term::failure(&mut stdout);
            false
        }
    }
}

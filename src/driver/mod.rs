use std::io::{stdin, stdout};
use std::path::PathBuf;
use std::rc::Rc;

use crate::data::{DataManager, Snapshot, SnapshotData};
use crate::editor;
use crate::error::{unwrap_log, Error};
use crate::term;
use crate::term::Input;

use parser::Script;
use util::*;

mod cmd;
mod parser;
mod repl;
mod scanner;
mod util;

pub use repl::View;

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
            unwrap_log(self.data.add_snapshot(Rc::new(snapshot)));
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
                term::snap_summary(
                    &snap.name,
                    snap.description.as_ref(),
                    &snap.cmd,
                    &mut stdout,
                );
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

    /// Starts the REPL.
    pub fn repl(&mut self) {
        let snapshots = unwrap_log(self.data.get_all_snapshots());
        let mut view = repl::View::new(snapshots);
        let stdout = stdout();
        let stdin = stdin();
        let mut repl = term::Repl::new(stdin, stdout);
        let mut scanner = scanner::Scanner::new();
        let mut parser = parser::Parser::new();
        loop {
            match repl.run(&view) {
                Input::Up => view.up(),
                Input::Down => view.down(),
                Input::Quit => break,
                Input::Command(cmd) => {
                    let tokens = scanner.scan(cmd);
                    match parser.parse(tokens) {
                        Ok(script) => match script {
                            Script::Quit => break,
                            Script::Help => {
                                repl.clear();
                                term::help::write_help(&mut repl.stdout);
                                repl.checkpoint();
                            }
                            Script::Filter(args) => view.apply_filter(args),
                            Script::Clear => view.clear_filters(),
                            _ => break, // TODO
                        },
                        Err(error) => {
                            repl.clear();
                            repl.writeln(&error.message);
                            repl.checkpoint()
                        }
                    }
                }
            }
        }
        repl.clear();
    }
}

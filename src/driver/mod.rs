use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

use crate::data::{DataManager, Snapshot, SnapshotData};
use crate::editor;
use crate::error::{unwrap_log, Error};
use crate::term;
use crate::term::{Input, SanitizerWriter};

use parser::{Script, Target};
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
            unwrap_log(self.data.add_snapshot(snapshot));
        }
    }

    /// Handles run subcommand.
    /// Returns true in case of success, false otherwise.
    pub fn run(&mut self) -> bool {
        let mut stdout = stdout();
        let snapshots = unwrap_log(self.data.get_all_snapshots());
        let view = repl::View::new(snapshots);
        if self.run_view(&view, &mut stdout) {
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
                                repl.suspend();
                                term::help::write_help(&mut repl.stdout);
                                repl.restore();
                            }
                            Script::Edit => {
                                repl.suspend();
                                if let Some(mut snap) = view.get_selected_mut() {
                                    if self.edit_snapshot(&mut snap, &mut repl.stdout) {
                                        drop(snap); // Release the mutable borrow to allow data.persist
                                        unwrap_log(self.data.persist());
                                    }
                                } else {
                                    repl.writeln("No snapshot to edit.")
                                }
                                repl.restore();
                            }
                            Script::Filter(args) => view.apply_filter(args),
                            Script::Clear => view.clear_filters(),
                            Script::Run(target) => {
                                repl.suspend();
                                let success = match target {
                                    Target::All => self.run_view(&view, &mut repl.stdout),
                                    Target::Selected => match view.get_selected() {
                                        Some(snap) => self.run_snapshot(&snap, &mut repl.stdout),
                                        None => true,
                                    },
                                };
                                if success {
                                    term::success(&mut repl.stdout);
                                } else {
                                    term::failure(&mut repl.stdout);
                                }
                                repl.restore();
                            }
                            Script::Show(target) => {
                                repl.suspend();
                                match target {
                                    Target::Selected => match view.get_selected() {
                                        Some(snap) => self.show_snapshot(&snap, &mut repl.stdout),
                                        None => (),
                                    },
                                    Target::All => {
                                        for snap in view.get_view() {
                                            self.show_snapshot(&snap.borrow(), &mut repl.stdout);
                                        }
                                    }
                                }
                                repl.restore();
                            }
                        },
                        Err(error) => {
                            repl.suspend();
                            repl.writeln(&error.message);
                            repl.restore();
                        }
                    }
                }
            }
        }
        repl.clear();
    }

    /// Runs only commands from the given view.
    fn run_view<B: Write>(&mut self, view: &View, buffer: &mut B) -> bool {
        let mut success = true;
        for snap in view.get_view() {
            let pass = self.run_snapshot(&snap.borrow(), buffer);
            success = success && pass;
        }
        success
    }

    /// Runs a single snapshot.
    fn run_snapshot<B: Write>(&self, snap: &Snapshot, buffer: &mut B) -> bool {
        let empty_body = Vec::new();
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
            term::title_separator("info", 2, buffer);
            term::snap_summary(&snap.name, snap.description.as_ref(), &snap.cmd, buffer);
        }
        if &result.stdout != old_stdout {
            term::title_separator("stdout", 0, buffer);
            term::write_diff(old_stdout, &result.stdout, buffer);
        }
        if &result.stderr != old_stderr {
            term::title_separator("stderr", 0, buffer);
            term::write_diff(old_stderr, &result.stderr, buffer);
        }
        if failed {
            term::separator(6, buffer);
        }
        !failed
    }

    /// Shows a single test.
    fn show_snapshot<B: Write>(&self, snap: &Snapshot, buffer: &mut B) {
        term::title_separator("info", 2, buffer);
        term::snap_summary(&snap.name, snap.description.as_ref(), &snap.cmd, buffer);
        if let Some(stdout) = &snap.stdout {
            term::title_separator("stdout", 0, buffer);
            buffer.sanitized_write(&stdout.body).unwrap();
        }
        if let Some(stderr) = &snap.stderr {
            term::title_separator("stderr", 0, buffer);
            buffer.sanitized_write(&stderr.body).unwrap();
        }
        term::separator(6, buffer);
    }

    /// Edits the selected snapshot.
    /// Returns true if there was a change, false otherwise.
    fn edit_snapshot<B: Write>(&self, snap: &mut Snapshot, buffer: &mut B) -> bool {
        let description = match snap.description.as_ref() {
            Some(desc) => desc,
            None => "",
        };
        match editor::open_snap(&self.path, &snap.name, description, &snap.cmd) {
            Ok(edit) => {
                let mut has_changed = false;
                if let Some(name) = edit.name {
                    if name != snap.name {
                        snap.name = name;
                        has_changed = true;
                    }
                }
                if edit.description != snap.description {
                    snap.description = edit.description;
                    has_changed = true;
                }
                if has_changed {
                    term::writeln("Updated.", buffer);
                    true
                } else {
                    term::writeln("Nothing to change.", buffer);
                    false
                }
            }
            Err(err) => {
                term::writeln(&err.message, buffer);
                false
            }
        }
    }
}

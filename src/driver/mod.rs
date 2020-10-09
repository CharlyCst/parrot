use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

use crate::data::{DataManager, Snapshot, SnapshotStatus};
use crate::editor;
use crate::error::{Error, Log};
use crate::term;
use crate::term::{BoxedWriter, Input, SeparatorKind};
use crate::parser;

use parser::{Filter, Command, Target, parse};
use util::*;

mod cmd;
mod repl;
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
        self.data.initialize().unwrap_log();
        println!("Parrot has been initialized.")
    }

    /// Handles add subcommand.
    pub fn add(&mut self, cmd: &str, name: &Option<String>, yes: bool) {
        let snap = cmd::execute(&cmd, &self.path).unwrap_log();
        let save = if yes {
            true
        } else {
            term::snap_preview(&snap, &mut stdout());
            term::binary_qestion("Save this snapshot?").unwrap_log()
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
                    let edit_result = editor::open_empty(&self.path, cmd).unwrap_log();
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
            self.data.add_snapshot(snapshot).unwrap_log();
        }
    }

    /// Handles run subcommand.
    /// Returns true in case of success, false otherwise.
    pub fn run(&mut self) -> bool {
        let mut stdout = stdout();
        let snapshots = self.data.get_all_snapshots().unwrap_log();
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
        let snapshots = self.data.get_all_snapshots().unwrap_log();
        let mut view = repl::View::new(snapshots);
        let stdout = stdout();
        let stdin = stdin();
        let mut repl = term::Repl::new(stdin, stdout);
        loop {
            match repl.run(&view) {
                Input::Up => view.up(),
                Input::Down => view.down(),
                Input::Quit => break,
                Input::Command(cmd) => {
                    match parse(&cmd) {
                        Ok(cmd) => match cmd {
                            Command::Quit => break,
                            Command::Help => self.execute_help(&mut repl),
                            Command::Edit => self.execute_edit(&mut repl, &view),
                            Command::Clear => view.clear_filters(),
                            Command::Filter(args) => view.apply_filter(args),
                            Command::Run(target) => self.execute_run(&mut repl, &view, target),
                            Command::Show(target) => self.execute_show(&mut repl, &view, target),
                            Command::Update(target) => self.execute_update(&mut repl, &view, target),
                            Command::Delete(target) => {
                                self.execute_delete(&mut repl, &mut view, target)
                            }
                        },
                        Err(error) => {
                            repl.suspend();
                            repl.writeln(&error);
                            repl.restore();
                        }
                    }
                }
            }
        }
        // Clear the REPL befor exiting
        repl.suspend();
    }

    /// Executes the help command.
    fn execute_help(&self, repl: &mut term::Repl) {
        repl.suspend();
        term::help::write_help(&mut repl.stdout);
        repl.restore();
    }

    /// Executes the edit command.
    fn execute_edit(&self, repl: &mut term::Repl, view: &View) {
        repl.suspend();
        if let Some(mut snap) = view.get_selected_mut() {
            if self.edit_snapshot(&mut snap, &mut repl.stdout) {
                drop(snap); // Release the mutable borrow to allow data.persist
                self.data.persist_metadata().unwrap_log();
            }
        } else {
            repl.writeln("No snapshot to edit.")
        }
        repl.restore();
    }

    /// Executes the run command.
    fn execute_run(&mut self, repl: &mut term::Repl, view: &View, target: Target) {
        repl.suspend();
        let success = match target {
            Target::All => self.run_view(&view, &mut repl.stdout),
            Target::Selected => match view.get_selected_mut() {
                Some(mut snap) => self.run_snapshot(&mut snap, &mut repl.stdout),
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

    /// Executes the run command.
    fn execute_update(&mut self, repl: &mut term::Repl, view: &View, target: Target) {
        repl.suspend();
        match target {
            Target::All => self.update_view(repl, view),
            Target::Selected => self.update_selected(repl, view),
        };
        repl.restore();
    }

    /// Executes the show command.
    fn execute_show(&self, repl: &mut term::Repl, view: &View, target: Target) {
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

    /// Executes the delete command.
    fn execute_delete(&self, repl: &mut term::Repl, view: &mut View, target: Target) {
        repl.suspend();
        match target {
            Target::Selected => match view.get_selected_mut() {
                Some(mut snap) => {
                    snap.deleted = true;
                    repl.writeln("Deleted 1 snapshot");
                }
                None => repl.writeln("No snapshot to delete."),
            },
            Target::All => {
                let mut count = 0;
                for snap in view.get_view() {
                    let mut snap = snap.borrow_mut();
                    snap.deleted = true;
                    count += 1;
                }
                if count > 1 {
                    repl.writeln(&format!("Deleted {} snapshots.", count));
                } else {
                    repl.writeln(&format!("Deleted {} snapshot.", count));
                }
            }
        }
        self.data.gc_snapshots().unwrap_log();
        view.apply_filter(Filter::Deleted);
        repl.restore();
    }

    /// Runs only commands from the given view.
    fn run_view<B: Write>(&mut self, view: &View, buffer: &mut B) -> bool {
        let mut success = true;
        for snap in view.get_view() {
            let pass = self.run_snapshot(&mut snap.borrow_mut(), buffer);
            success = success && pass;
        }
        success
    }

    /// Runs a single snapshot.
    fn run_snapshot<B: Write>(&self, snap: &mut Snapshot, buffer: &mut B) -> bool {
        let empty_body = Vec::new();
        let result = cmd::execute(&snap.cmd, &self.path).unwrap_log();
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
            term::box_separator(&snap.name, SeparatorKind::Top, buffer);
            term::snap_summary(snap.description.as_ref(), &snap.cmd, snap.exit_code, buffer);
        }
        if &result.stdout != old_stdout {
            term::box_separator("stdout", SeparatorKind::Middle, buffer);
            term::write_diff(old_stdout, &result.stdout, buffer);
        }
        if &result.stderr != old_stderr {
            term::box_separator("stderr", SeparatorKind::Middle, buffer);
            term::write_diff(old_stderr, &result.stderr, buffer);
        }
        if failed {
            term::box_separator("", SeparatorKind::Bottom, buffer);
            snap.status = SnapshotStatus::Failed;
        } else {
            snap.status = SnapshotStatus::Passed;
        }
        !failed
    }

    /// Shows a single test.
    fn show_snapshot<B: Write>(&self, snap: &Snapshot, buffer: &mut B) {
        term::box_separator(&snap.name, SeparatorKind::Top, buffer);
        term::snap_summary(snap.description.as_ref(), &snap.cmd, snap.exit_code, buffer);
        if let Some(stdout) = &snap.stdout {
            term::box_separator("stdout", SeparatorKind::Middle, buffer);
            buffer.boxed_write(&stdout.body).unwrap();
        }
        if let Some(stderr) = &snap.stderr {
            term::box_separator("stderr", SeparatorKind::Middle, buffer);
            buffer.boxed_write(&stderr.body).unwrap();
        }
        term::box_separator("", SeparatorKind::Bottom, buffer);
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
                    snap.tags = edit.tags;
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

    /// Updates all the snapshots of the current view.
    fn update_view(&self, repl: &mut term::Repl, view: &View) {
        let mut count = 0;
        for snap in view.get_view() {
            let mut snap = snap.borrow_mut();
            if self.update_snapshot(&mut snap) {
                self.data.persist_snapshot_data(&snap).unwrap_log();
                count += 1;
            }
        }
        if count > 0 {
            if count == 1 {
                repl.writeln("Updated 1 snapshot.");
            } else {
                repl.writeln(&format!("Updated {} snapshots.", count));
            }
            self.data.persist_metadata().unwrap_log();
        } else {
            repl.writeln("Nothing to do.");
        }
    }

    /// Updates the snapshot selected in the current view.
    fn update_selected(&self, repl: &mut term::Repl, view: &View) {
        match view.get_selected_mut() {
            Some(mut snap) => {
                if self.update_snapshot(&mut snap) {
                    self.data.persist_snapshot_data(&snap).unwrap_log();
                    drop(snap); // Release mut ref before persisting
                    self.data.persist_metadata().unwrap_log();
                    repl.writeln("Updated 1 snapshot.")
                } else {
                    repl.writeln("Nothing to do.")
                }
            }
            None => repl.writeln("No snapshot to update."),
        }
    }

    /// Updates a single snapshot.
    /// Returns true if there was a change, false otherwise.
    /// The command will be run to get the new output, there is no caching for
    /// now.
    fn update_snapshot(&self, snap: &mut Snapshot) -> bool {
        let result = cmd::execute(&snap.cmd, &self.path).unwrap_log();
        let mut has_changed = false;
        let new_stdout = util::to_snapshot_data(result.stdout, &snap.name, ".out");
        let new_stderr = util::to_snapshot_data(result.stderr, &snap.name, ".err");
        if snap.exit_code != result.status.code() {
            snap.exit_code = result.status.code();
            has_changed = true;
        }
        if snap.stdout != new_stdout {
            snap.stdout = new_stdout;
            has_changed = true;
        }
        if snap.stderr != new_stderr {
            snap.stderr = new_stderr;
            has_changed = true;
        }
        snap.status = SnapshotStatus::Passed;
        has_changed
    }
}

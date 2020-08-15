use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{wrap, Error};

mod metadata;
mod snapshots;

pub const PARROT_PATH: &'static str = ".parrot";
const SNAPSHOT_PATH: &'static str = "snapshots";
const METADATA_PATH: &'static str = "metadata.json";

pub struct Snapshot {
    pub exit_code: Option<i32>,
    pub stderr: Option<SnapshotData>,
    pub stdout: Option<SnapshotData>,
    pub cmd: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

pub struct SnapshotData {
    pub path: String,
    pub body: Vec<u8>,
}

pub struct DataManager {
    metadata: metadata::MetadataManager,
    snaps: snapshots::SnapshotsManager,
    path: PathBuf,
}

impl DataManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<DataManager, Error> {
        let path = path.as_ref();

        // Ensure that the path exists.
        if !path.is_dir() {
            return Err(Error {
                message: format!(
                    "Path is not a directory: {}",
                    path.to_str().unwrap_or("PATH_ERROR")
                ),
                cause: None,
            });
        }

        let path = path.join(PARROT_PATH);
        let metadata_path = path.join(METADATA_PATH);
        let snapshots_path = path.join(SNAPSHOT_PATH);
        Ok(DataManager {
            metadata: metadata::MetadataManager::new(metadata_path),
            snaps: snapshots::SnapshotsManager::new(snapshots_path),
            path,
        })
    }

    pub fn initialize(&mut self) -> Result<(), Error> {
        if self.path.exists() {
            return Error::from_str("A parrot folder already exists.");
        }
        wrap(
            fs::create_dir(&self.path),
            "Unable to create a parrot folder.",
        )?;
        self.metadata.write_empty()?;
        self.snaps.create_empty()?;
        Ok(())
    }

    pub fn add_snapshot(&mut self, snap: &Snapshot) -> Result<(), Error> {
        self.snaps.create(snap)?;
        self.metadata.register_snap(snap)?;
        Ok(())
    }

    /// Return a copy of all the snapshots and their metadata.
    pub fn get_all_snapshots(&mut self) -> Result<Vec<Snapshot>, Error> {
        let mut snapshots = Vec::new();
        let metadata = self.metadata.get_metadatas()?;
        for snap in &metadata.snapshots {
            let stdout = if let Some(path) = snap.stdout.clone() {
                Some(SnapshotData {
                    body: self.snaps.get(&path)?,
                    path,
                })
            } else {
                None
            };
            let stderr = if let Some(path) = snap.stderr.clone() {
                Some(SnapshotData {
                    body: self.snaps.get(&path)?,
                    path,
                })
            } else {
                None
            };
            snapshots.push(Snapshot {
                name: snap.name.clone(),
                description: snap.description.clone(),
                tags: snap.tags.clone(),
                cmd: snap.cmd.clone(),
                exit_code: snap.exit_code.clone(),
                stdout,
                stderr,
            })
        }

        Ok(snapshots)
    }
}

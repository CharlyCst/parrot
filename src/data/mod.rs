use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{wrap, Error};

mod metadata;
mod snapshots;

const PARROT_PATH: &'static str = ".parrot";
const SNAPSHOT_PATH: &'static str = "snapshots";
const METADATA_PATH: &'static str = "metadata.json";

pub struct Snapshot {
    pub content: Vec<u8>,
    pub name: String,
    pub cmd: String,
}

pub struct DataManager {
    metadata: metadata::ConfigManager,
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
            metadata: metadata::ConfigManager::new(metadata_path),
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

    pub fn add_snapshot(&mut self, cmd: &str, name: &str, snap: &Vec<u8>) -> Result<(), Error> {
        self.snaps.create(name, snap)?;
        self.metadata.register_snap(cmd, name)?;
        Ok(())
    }

    /// Return a copy of all the snapshots and their metadata.
    pub fn get_all_snapshots(&mut self) -> Result<Vec<Snapshot>, Error> {
        let mut snapshots = Vec::new();
        let metadata = self.metadata.get_metadata()?;
        for snap in &metadata.snapshots {
            let content = self.snaps.get(&snap.snap)?;
            snapshots.push(Snapshot {
                content,
                name: snap.snap.clone(),
                cmd: snap.cmd.clone(),
            })
        }

        Ok(snapshots)
    }
}

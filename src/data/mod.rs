use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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
    snaps: Option<Vec<Rc<RefCell<Snapshot>>>>,
    metadata_manager: metadata::MetadataManager,
    snap_manager: snapshots::SnapshotsManager,
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
            snaps: None,
            metadata_manager: metadata::MetadataManager::new(metadata_path),
            snap_manager: snapshots::SnapshotsManager::new(snapshots_path),
            path,
        })
    }

    /// Initializes the Parrot storage folder.
    pub fn initialize(&mut self) -> Result<(), Error> {
        if self.path.exists() {
            return Error::from_str("A parrot folder already exists.");
        }
        wrap(
            fs::create_dir(&self.path),
            "Unable to create a parrot folder.",
        )?;
        self.metadata_manager.write_empty()?;
        self.snap_manager.create_empty()?;
        Ok(())
    }

    /// Adds a snapshot and persist all snapshots to file system.
    pub fn add_snapshot(&mut self, snap: Snapshot) -> Result<(), Error> {
        self.snap_manager.create(&snap)?;
        let snaps = self.get_snaps()?;
        snaps.push(Rc::new(RefCell::new(snap)));
        // Unwrap is safe because `self.get_snaps` caches snaps.
        self.metadata_manager
            .persist(self.snaps.as_ref().unwrap())?;
        Ok(())
    }

    /// Persists the snapshots to file system, should be used after any
    /// snapshot update.
    pub fn persist(&self) -> Result<(), Error> {
        if let Some(snaps) = self.snaps.as_ref() {
            self.metadata_manager.persist(snaps)?;
        }
        Ok(())
    }

    /// Returns a vector of snapshot references.
    pub fn get_all_snapshots(&mut self) -> Result<Vec<Rc<RefCell<Snapshot>>>, Error> {
        let mut snaps = Vec::new();
        for snap in self.get_snaps()? {
            snaps.push(Rc::clone(snap));
        }
        Ok(snaps)
    }

    /// Lazyly loads snapshots.
    fn get_snaps(&mut self) -> Result<&mut Vec<Rc<RefCell<Snapshot>>>, Error> {
        if let Some(ref mut snaps) = self.snaps {
            Ok(snaps)
        } else {
            self.load()?;
            Ok(self.snaps.as_mut().unwrap())
        }
    }

    /// Loads all the snapshots from file system and cache them.
    /// `self.snaps` is Some after this function.
    fn load(&mut self) -> Result<(), Error> {
        let metadatas = self.metadata_manager.get_metadata()?;
        let mut snaps = Vec::with_capacity(metadatas.snapshots.len());
        for snap in metadatas.snapshots {
            let stdout = self.load_snapshot_body(snap.stdout)?;
            let stderr = self.load_snapshot_body(snap.stderr)?;
            snaps.push(Rc::new(RefCell::new(Snapshot {
                exit_code: snap.exit_code,
                stderr,
                stdout,
                cmd: snap.cmd,
                name: snap.name,
                description: snap.description,
                tags: snap.tags,
            })))
        }
        self.snaps = Some(snaps);
        Ok(())
    }

    /// Loads the body of a snapshot from an Option<body_path>.
    fn load_snapshot_body(&self, path: Option<String>) -> Result<Option<SnapshotData>, Error> {
        match path {
            None => Ok(None),
            Some(path) => Ok(Some(SnapshotData {
                body: self.snap_manager.get(&path)?,
                path: path,
            })),
        }
    }
}

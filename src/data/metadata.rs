use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

use super::Snapshot;
use crate::error::{wrap, Error};

#[derive(Serialize, Deserialize)]
pub struct Metadatas {
    pub snapshots: Vec<Metadata>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub cmd: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub struct MetadataManager {
    path: PathBuf,
}

impl MetadataManager {
    /// Initialize a new MetadataManager.
    pub fn new(confg_path: PathBuf) -> MetadataManager {
        MetadataManager { path: confg_path }
    }

    /// Write an empty metadata file.
    /// Be careful: this will override the current metadata if any.
    pub fn write_empty(&mut self) -> Result<(), Error> {
        let metadatas = Metadatas {
            snapshots: Vec::new(),
        };
        self.write(&metadatas)?;
        Ok(())
    }

    /// Persists metadata to the file system from the list of snapshots.
    /// Borrows an immutable reference to the snapshots.
    pub fn persist(&self, snaps: &Vec<Rc<RefCell<Snapshot>>>) -> Result<(), Error> {
        let mut snapshots = Vec::with_capacity(snaps.len());
        for snap in snaps {
            let snap = snap.borrow();
            if snap.deleted {
                continue;
            }
            let stdout = match &snap.stdout {
                Some(data) => Some(data.path.clone()),
                None => None,
            };
            let stderr = match &snap.stderr {
                Some(data) => Some(data.path.clone()),
                None => None,
            };
            snapshots.push(Metadata {
                cmd: snap.cmd.clone(),
                name: snap.name.clone(),
                description: snap.description.clone(),
                tags: snap.tags.clone(),
                exit_code: snap.exit_code.clone(),
                stdout,
                stderr,
            })
        }
        self.write(&Metadatas { snapshots })?;
        Ok(())
    }

    /// Reads and return metadatas from file system.
    pub fn get_metadata(&self) -> Result<Metadatas, Error> {
        let file = wrap(fs::File::open(&self.path), "Could not fine metadata.json.")?;
        let metadatas = wrap(
            serde_json::from_reader(file),
            "Failed to parse metadata.json.",
        )?;
        Ok(metadatas)
    }

    /// Writes metadatas to the file system.
    fn write(&self, metadatas: &Metadatas) -> Result<(), Error> {
        let metadata_file = wrap(
            fs::File::create(&self.path),
            "Failed to create metadata.json.",
        )?;
        wrap(
            serde_json::to_writer_pretty(metadata_file, metadatas),
            "Failed to write metadata.json.",
        )?;
        Ok(())
    }
}

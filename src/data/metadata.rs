use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
    metadatas: Option<Metadatas>,
}

impl MetadataManager {
    /// Initialize a new MetadataManager.
    pub fn new(confg_path: PathBuf) -> MetadataManager {
        MetadataManager {
            path: confg_path,
            metadatas: None,
        }
    }

    /// Write an empty metadata file.
    /// Be careful: this will override the current metadata if any.
    pub fn write_empty(&mut self) -> Result<(), Error> {
        self.metadatas = Some(Metadatas {
            snapshots: Vec::new(),
        });
        self.write()?;
        Ok(())
    }

    /// Register a new snapshot with its associated command.
    pub fn register_snap(&mut self, snap: &Snapshot) -> Result<(), Error> {
        let metadata = self.get_metadatas()?;
        let cmd = snap.cmd.clone();
        let name = snap.name.clone();
        let description = snap.description.clone();
        let tags = snap.tags.clone();
        let exit_code = snap.exit_code.clone();
        let stdout = if let Some(stdout) = &snap.stdout {
            Some(stdout.path.clone())
        } else {
            None
        };
        let stderr = if let Some(stderr) = &snap.stderr {
            Some(stderr.path.clone())
        } else {
            None
        };
        let snap = Metadata {
            cmd,
            name,
            description,
            tags,
            exit_code,
            stdout,
            stderr,
        };
        metadata.snapshots.push(snap);
        self.write()?;
        Ok(())
    }

    /// Return the metadatas.
    pub fn get_metadatas(&mut self) -> Result<&mut Metadatas, Error> {
        if let Some(ref mut metadata) = self.metadatas {
            Ok(metadata)
        } else {
            self.read()?;
            let metadatas = self.metadatas.as_mut().unwrap();
            Ok(metadatas)
        }
    }

    /// Read and cache the metadata from file.
    /// self.metadata is Some after this function (expect in case of error).
    fn read(&mut self) -> Result<(), Error> {
        let file = wrap(fs::File::open(&self.path), "Could not fine metadata.json.")?;
        let metadatas = wrap(
            serde_json::from_reader(file),
            "Failed to parse metadata.json.",
        )?;
        self.metadatas = Some(metadatas);
        Ok(())
    }

    /// Write the current metadata.
    fn write(&self) -> Result<(), Error> {
        let metadata_file = wrap(
            fs::File::create(&self.path),
            "Failed to create metadata.json.",
        )?;
        wrap(
            serde_json::to_writer_pretty(metadata_file, &self.metadatas),
            "Failed to write metadata.json.",
        )?;
        Ok(())
    }
}

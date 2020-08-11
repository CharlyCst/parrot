use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{wrap, Error};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub snapshots: Vec<Metadata>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub cmd: String,
    pub snap: String,
}

pub struct ConfigManager {
    path: PathBuf,
    metadata: Option<Config>,
}

impl ConfigManager {
    /// Initialize a new ConfigManager.
    pub fn new(confg_path: PathBuf) -> ConfigManager {
        ConfigManager {
            path: confg_path,
            metadata: None,
        }
    }

    /// Write an empty metadata file.
    /// Be careful: this will override the current metadata if any.
    pub fn write_empty(&mut self) -> Result<(), Error> {
        self.metadata = Some(Config {
            snapshots: Vec::new(),
        });
        self.write()?;
        Ok(())
    }

    /// Register a new snapshot with its associated command.
    pub fn register_snap(&mut self, cmd: &str, snap_name: &str) -> Result<(), Error> {
        let metadata = self.get_metadata()?;
        let snap = Metadata {
            cmd: cmd.to_owned(),
            snap: snap_name.to_owned(),
        };
        metadata.snapshots.push(snap);
        self.write()?;
        Ok(())
    }

    /// Return the metadata.
    pub fn get_metadata(&mut self) -> Result<&mut Config, Error> {
        if let Some(ref mut metadata) = self.metadata {
            Ok(metadata)
        } else {
            self.read()?;
            let metadata = self.metadata.as_mut().unwrap();
            Ok(metadata)
        }
    }

    /// Read and cache the metadata from file.
    /// self.metadata is Some after this function (expect in case of error).
    fn read(&mut self) -> Result<(), Error> {
        let file = wrap(fs::File::open(&self.path), "Could not fine metadata.json.")?;
        let metadata = wrap(
            serde_json::from_reader(file),
            "Failed to parse metadata.json.",
        )?;
        self.metadata = Some(metadata);
        Ok(())
    }

    /// Write the current metadata.
    fn write(&self) -> Result<(), Error> {
        let metadata_file = wrap(
            fs::File::create(&self.path),
            "Failed to create metadata.json.",
        )?;
        wrap(
            serde_json::to_writer(metadata_file, &self.metadata),
            "Failed to write metadata.json.",
        )?;
        Ok(())
    }
}

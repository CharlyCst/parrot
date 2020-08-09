use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;

mod config;
mod snapshots;

const PARROT_PATH: &'static str = "parrot";
const SNAPSHOT_PATH: &'static str = "snapshots";
const CONFIG_PATH: &'static str = "config.json";

pub struct DataManager {
    config: config::ConfigManager,
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
        let config_path = path.join(CONFIG_PATH);
        let snapshots_path = path.join(SNAPSHOT_PATH);
        Ok(DataManager {
            config: config::ConfigManager::new(config_path),
            snaps: snapshots::SnapshotsManager::new(snapshots_path),
            path,
        })
    }

    pub fn initialize(&self) -> Result<(), Error> {
        if self.path.exists() {
            return Err(Error {
                message: String::from("A parrot folder already exists."),
                cause: None,
            });
        }
        if let Err(err) = fs::create_dir(&self.path) {
            return Err(Error {
                message: String::from("Unable to create a parrot folder."),
                cause: Some(Box::new(err)),
            });
        }
        if let Err(err) = self.config.write_empty() {
            return Err(Error {
                message: String::from("Unable to create config.json file."),
                cause: Some(err),
            });
        }
        if let Err(err) = self.snaps.create_empty() {
            return Err(Error {
                message: String::from("Unable to create a snapshots folder."),
                cause: Some(err),
            });
        }

        Ok(())
    }
}

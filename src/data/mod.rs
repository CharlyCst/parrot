use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{wrap, Error};

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

    pub fn initialize(&mut self) -> Result<(), Error> {
        if self.path.exists() {
            return Error::from_str("A parrot folder already exists.");
        }
        wrap(
            fs::create_dir(&self.path),
            "Unable to create a parrot folder.",
        )?;
        self.config.write_empty()?;
        self.snaps.create_empty()?;
        Ok(())
    }

    pub fn add_snapshot(&mut self, cmd: &str, name: &str, snap: &Vec<u8>) -> Result<(), Error> {
        self.snaps.create(name, snap)?;
        self.config.register_snap(cmd, name)?;
        Ok(())
    }
}

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    snapshots: Vec<Snapshot>,
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    cmd: String,
    snap: String,
}

pub struct ConfigManager {
    path: PathBuf,
}

impl ConfigManager {
    /// Initialize a new ConfigManager.
    pub fn new(confg_path: PathBuf) -> ConfigManager {
        ConfigManager { path: confg_path }
    }

    /// Write an empty configuration file.
    /// Be careful: this will override the current configuration if any.
    pub fn write_empty(&self) -> Result<(), Box<dyn Error>> {
        let config = Config {
            snapshots: Vec::new(),
        };
        let config_file = fs::File::create(&self.path)?;
        serde_json::to_writer(config_file, &config)?;
        Ok(())
    }
}


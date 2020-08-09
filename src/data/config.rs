use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{wrap, Error};

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
    config: Option<Config>,
}

impl ConfigManager {
    /// Initialize a new ConfigManager.
    pub fn new(confg_path: PathBuf) -> ConfigManager {
        ConfigManager {
            path: confg_path,
            config: None,
        }
    }

    /// Write an empty configuration file.
    /// Be careful: this will override the current configuration if any.
    pub fn write_empty(&mut self) -> Result<(), Error> {
        self.config = Some(Config {
            snapshots: Vec::new(),
        });
        self.write()?;
        Ok(())
    }

    /// Register a new snapshot with its associated command.
    pub fn register_snap(&mut self, cmd: &str, snap_name: &str) -> Result<(), Error> {
        let config = self.get_config()?;
        let snap = Snapshot {
            cmd: cmd.to_owned(),
            snap: snap_name.to_owned(),
        };
        config.snapshots.push(snap);
        self.write()?;
        Ok(())
    }

    /// Return the configuration.
    fn get_config(&mut self) -> Result<&mut Config, Error> {
        if let Some(ref mut config) = self.config {
            Ok(config)
        } else {
            self.read()?;
            let config = self.config.as_mut().unwrap();
            Ok(config)
        }
    }

    /// Read and cache the configuration from file.
    /// self.config is Some after this function (expect in case of error).
    fn read(&mut self) -> Result<(), Error> {
        let file = wrap(fs::File::open(&self.path), "Could not fine config.json.")?;
        let config = wrap(
            serde_json::from_reader(file),
            "Failed to parse config.json.",
        )?;
        self.config = Some(config);
        Ok(())
    }

    /// Write the current configuration.
    fn write(&self) -> Result<(), Error> {
        let config_file = wrap(
            fs::File::create(&self.path),
            "Failed to create config.json.",
        )?;
        wrap(
            serde_json::to_writer(config_file, &self.config),
            "Failed to write config.json.",
        )?;
        Ok(())
    }
}

use std::fs;
use std::io::prelude::{Read, Write};
use std::path::PathBuf;

use crate::error::{wrap, Error};

const FILE_EXTENSION: &'static str = ".txt";

pub struct SnapshotsManager {
    path: PathBuf,
}

impl SnapshotsManager {
    /// Initialize a new SnapshotsManager.
    pub fn new(snapshots_path: PathBuf) -> SnapshotsManager {
        SnapshotsManager {
            path: snapshots_path,
        }
    }

    /// Create an empty snapshot folder.
    pub fn create_empty(&self) -> Result<(), Error> {
        wrap(
            fs::create_dir(&self.path),
            "Failed to create a snapshots folder.",
        )?;
        Ok(())
    }

    /// Create a new snapshot file, abort if the file already exists.
    pub fn create(&self, name: &str, snap: &Vec<u8>) -> Result<(), Error> {
        let mut name = name.to_owned();
        name.push_str(FILE_EXTENSION);
        let path = self.path.join(name);
        if path.exists() {
            return Error::from_str("A snapshot with that name already exists.");
        }
        let mut file = wrap(fs::File::create(path), "Failed to create a snapshot file")?;
        wrap(file.write_all(&snap), "Faile to write down the snapshot")?;
        Ok(())
    }

    /// Read a snapshot from file.
    pub fn get(&self, name: &str) -> Result<Vec<u8>, Error> {
        let mut snap = Vec::new();
        let mut name = name.to_owned();
        name.push_str(FILE_EXTENSION);
        let path = self.path.join(&name);
        let mut file = wrap(
            fs::File::open(path),
            &format!("Could not open snapshot {}.", name),
        )?;
        wrap(
            file.read_to_end(&mut snap),
            &format!("Failed to open snapshot {}.", name),
        )?;
        Ok(snap)
    }
}

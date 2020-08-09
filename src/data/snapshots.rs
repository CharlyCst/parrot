use std::path::PathBuf;
use std::fs;
use std::error::Error;

pub struct SnapshotsManager {
    path: PathBuf,
}

impl SnapshotsManager {
    /// Initialize a new SnapshotsManager.
    pub fn new(snapshots_path: PathBuf) -> SnapshotsManager {
        SnapshotsManager { path: snapshots_path }
    }

    /// Create an empty snapshot folder.
    pub fn create_empty(&self) -> Result<(), Box<dyn Error>> {
        fs::create_dir(&self.path)?;
        Ok(())
    }
}

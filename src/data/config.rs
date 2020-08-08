use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    snapshots: Vec<Snapshot>,
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    cmd: String,
    snap: String,
}

pub fn get_default_config() -> Config {
    Config {
        snapshots: Vec::new(),
    }
}

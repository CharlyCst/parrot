use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::process::Output;

use crate::data::{Snapshot, SnapshotData, SnapshotStatus};

/// Creates a snapshot out of an execution result
pub fn to_snapshot(
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    cmd: String,
    snap: Output,
) -> Snapshot {
    let exit_code = snap.status.code();
    let stdout = to_snapshot_data(snap.stdout, &name, ".out");
    let stderr = to_snapshot_data(snap.stderr, &name, ".err");
    Snapshot {
        cmd,
        name,
        description,
        tags,
        exit_code,
        stdout,
        stderr,
        status: SnapshotStatus::Waiting,
        deleted: false,
    }
}

/// Creates a snapshot_data item from raw body.
pub fn to_snapshot_data(body: Vec<u8>, path: &str, path_extension: &str) -> Option<SnapshotData> {
    if body.len() > 0 {
        let mut path = path.to_owned();
        path.push_str(path_extension);
        Some(SnapshotData { body, path })
    } else {
        None
    }
}

/// Normalizes a string for use a file name.
pub fn normalize_name(name: &str) -> String {
    name.trim().replace(' ', "_").replace('\t', "_")
}

/// Generates a random name starting with '_'.
pub fn get_random_name() -> String {
    let mut random_name = String::from("_");
    random_name.extend(thread_rng().sample_iter(&Alphanumeric).take(30));
    random_name
}

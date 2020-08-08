use serde_json;
use std::fs;
use std::path::{Path};

mod config;

const PARROT_PATH: &'static str = "parrot";
const SNAPSHOT_PATH: &'static str = "snapshots";
const CONFIG_PATH: &'static str = "config.json";

pub fn hello_world() -> String {
    String::from("Hello, World!")
}

pub fn initialize<P: AsRef<Path>>(path: P) -> Result<(), ()> {
    let path = path.as_ref();

    // Ensure that the environment is clean.
    if !path.is_dir() {
        println!(
            "Path is not a directory: {}",
            path.to_str().unwrap_or("PATH_ERROR")
        );
        return Err(());
    }
    let path = path.join(PARROT_PATH);
    if path.exists() {
        println!("A 'parrot' folder already exists.");
        return Err(());
    }

    // Create and initialize main folder.
    let snapshots_path = path.join(SNAPSHOT_PATH);
    let config_path = path.join(CONFIG_PATH);
    let config_value = config::get_default_config();
    if let Err(err) = fs::create_dir(&path) {
        println!("An error occurred while creating a folder: {}", err);
        return Err(());
    }
    if let Err(err) = fs::create_dir(&snapshots_path) {
        println!("An error occurred while creating a folder: {}", err);
        return Err(());
    }
    let config_file = match fs::File::create(&config_path) {
        Ok(file) => file,
        Err(err) => {
            println!(
                "An error occurred while creating the configuration file: {}",
                err
            );
            return Err(());
        }
    };
    if let Err(err) = serde_json::to_writer(config_file, &config_value) {
        println!("An error occurred while writting to config file: {}", err);
        return Err(());
    }

    Ok(())
}

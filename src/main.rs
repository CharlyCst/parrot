use cli::Command;
use error::unwrap_log;

mod cli;
mod cmd;
mod data;
mod error;
mod term;

fn main() {
    let config = cli::parse();
    let mut data = unwrap_log(data::DataManager::new(config.path));
    match config.cmd {
        Command::Init {} => {
            unwrap_log(data.initialize());
            println!("Parrot has been initialized.")
        }
        Command::Add { cmd, name, yes } => {
            let snap = unwrap_log(cmd::execute(&cmd));
            let name = if let Some(name) = name {
                name
            } else {
                String::from("default")
            };
            let save = if yes {
                true
            } else {
                term::color_box("Snapshot", &snap.stdout);
                unwrap_log(term::binary_qestion("Save this snapshot?"))
            };
            if save {
                unwrap_log(data.add_snapshot(&cmd, &name, &snap.stdout));
            }
        }
        Command::Run {} => {
            let mut success = true;
            let snaps = unwrap_log(data.get_all_snapshots());
            for snap in &snaps {
                let result = unwrap_log(cmd::execute(&snap.cmd));
                if result.stdout != snap.content {
                    success = false;
                    println!("Test failed for '{}'.", &snap.cmd);
                }
            }
            if success {
                println!("Success !");
            } else {
                println!("Failure...");
            }
        }
    }
}

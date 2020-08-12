use std::env::var;
use std::fs::{remove_file, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

use crate::data::PARROT_PATH;
use crate::error::{wrap, Error};

const FILE_NAME: &'static str = "PARROT_SNAPSHOT";

/// Open an empty description in the user's favorite editor.
pub fn open_empty<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let content = open(path, "", "")?;
    println!("File content:\n{}", content); // TODO: remove that
    Ok(())
}

/// Open a new description file in the user's favorite editor.
fn open<P: AsRef<Path>>(path: P, name: &str, description: &str) -> Result<String, Error> {
    let editor = var("EDITOR").unwrap();
    let mut file_path = path.as_ref().to_owned();
    file_path.push(PARROT_PATH);
    file_path.push(FILE_NAME);
    let mut file = wrap(
        File::create(&file_path),
        "Could not create description file",
    )?;
    wrap(
        write!(
            file,
            "{}\n\n\
             {}\n\n\

             // The first line will be used as snapshot name, the following as description.\n\
             // If the first line is blank, a random name will be used.\n\
             // Hastag in the description (#example) will serve as tag for the snapshot.\n\
             // Characters after '//' are ignored.\n",
            name, description
        ),
        "Could not write description file",
    )?;

    let status = wrap(
        Command::new(editor).arg(&file_path).status(),
        "An error occured with the text editor",
    )?;
    if !status.success() {
        return Error::from_str("Aborting");
    }

    let mut content = String::new();
    wrap(
        wrap(
            File::open(&file_path),
            "Could not open description file after editing",
        )?
        .read_to_string(&mut content),
        "Could not read the description file",
    )?;

    let _ = remove_file(&file_path);
    Ok(content)
}

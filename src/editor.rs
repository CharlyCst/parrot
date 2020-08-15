use std::env::var;
use std::fs::{remove_file, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use regex::Regex;

use crate::data::PARROT_PATH;
use crate::error::{wrap, Error};

const FILE_NAME: &'static str = "PARROT_SNAPSHOT";

pub struct EditResult {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

/// Open an empty description in the user's favorite editor.
pub fn open_empty<P: AsRef<Path>>(path: P, cmd: &str) -> Result<EditResult, Error> {
    open(path, "", "", cmd)
}

/// Open a new description file in the user's favorite editor.
fn open<P: AsRef<Path>>(path: P, name: &str, description: &str, cmd: &str) -> Result<EditResult, Error> {
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
            "{}\n\
             {}\n\n\

             // The first line will be used as snapshot name, the following as description.\n\
             // If the first line is blank, a random name will be used.\n\
             // Hastag in the description (#example) will serve as tag for the snapshot.\n\
             // Characters after '//' are ignored.\n\
             //\n\
             // Test command: {}",
            name, description, cmd
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

    Ok(parse_file(content))
}

/// Parse the content of the description file and return both title, description
/// and tags.
fn parse_file(content: String) -> EditResult {
    let lines = content.split('\n');
    let mut name = String::from("");
    let mut description = String::from("");
    let mut tags = Vec::new();
    let mut is_title = true;
    for line in lines {
        let (line, has_comment)  = strip_comment(line);
        if is_title {
            name.push_str(line.trim());
            is_title = false;
            continue;
        }
       
        if line.len() > 0 || !has_comment {
            description.push_str(line);
            description.push_str("\n");
        }
    }

    // Remove leadin/trainling whitespaces
    description = description.trim().to_owned();

    let re = Regex::new(r"#[a-zA-Z0-9_-]+").unwrap();
    for tag in re.captures_iter(&description) {
        tags.push(tag[0][1..].to_owned());
    }
    
    let name = if name.len() > 0 {
        Some(name)
    } else {
        None
    };
    let description = if description.len() > 0 {
        Some(description)
    } else {
        None
    };
    EditResult {
        name,
        description,
        tags
    }
}

/// Return a string slice stripped form the eventual comment.
/// A flag indicate if a comment was found.
fn strip_comment(line: &str) -> (&str, bool) {
    let mut iterator = line.split("//");
    let line = iterator.next().unwrap_or("");
    let has_comment = iterator.next().is_some();
    (line, has_comment)
}

use std::io::{stdin, stdout, Write};
use termion::{color, style};

use crate::error::{wrap, Error};

mod diff;
pub mod help;
mod repl;

pub use diff::write_diff;
pub use repl::Input;
pub use repl::Repl;

/// Ask a binary question to the user. Return true for yes, false for no.
pub fn binary_qestion(question: &str) -> Result<bool, Error> {
    let stdin = stdin();
    let mut stdout = stdout();
    loop {
        let mut buffer = String::new();
        print!("{} y(es) or n(o): ", question);
        wrap(stdout.flush(), "Unable to write to stdout")?;
        wrap(stdin.read_line(&mut buffer), "Undable to read from stdin")?;
        let buffer = buffer.trim().to_lowercase();
        if buffer == "yes" || buffer == "ye" || buffer == "y" {
            return Ok(true);
        }
        if buffer == "no" || buffer == "n" {
            return Ok(false);
        }
    }
}

/// Draws a colored box with a title and a given content.
pub fn color_box<B: Write>(title: &str, content: &Vec<u8>, mut buffer: &mut B) {
    title_separator(title, 0, &mut buffer);
    buffer.write_all(content).unwrap();
    // If no line break at the end, add one
    if *content.last().unwrap_or(&0x00) != 0x0a {
        write!(buffer, "\n\r").unwrap();
    }
    separator(title.len(), &mut buffer);
    buffer.flush().unwrap();
}

/// Writes a summary of a given snapshot.
pub fn snap_summary<B: Write>(name: &str, description: Option<&String>, cmd: &str, buffer: &mut B) {
    let bold = style::Bold;
    let reset = style::Reset;
    write!(buffer, "Test:    {}{}{}\n\r", bold, name, reset).unwrap();
    write!(buffer, "Command: {}{}{}\n\r", bold, cmd, reset).unwrap();
    if let Some(description) = description {
        write!(buffer, "\n\r{}\n\r", description).unwrap();
    }
}

/// Writes the success message.
pub fn success<B: Write>(buffer: &mut B) {
    write!(
        buffer,
        "{}{}Success ✓{}{}\n\r",
        color::Fg(color::LightGreen),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset)
    )
    .unwrap();
}

/// Writes the failure message.
pub fn failure<B: Write>(buffer: &mut B) {
    write!(
        buffer,
        "{}{}Failure ✗{}{}\n\r",
        color::Fg(color::LightRed),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset)
    )
    .unwrap();
}

/// Writes a separator featuring a title. Padding is rounded down to the closest
/// multiple of 2.
pub fn title_separator<B: Write>(title: &str, padding: usize, buffer: &mut B) {
    let padding = padding / 2 + 1;
    write!(
        buffer,
        "{blue}{s:/<2}{green}{s:/<2}{yellow}{s:/<4}{red}{s:/<7}{reset}{p:<padding$}{title}{p:<padding$}{red}{s:/<7}{yellow}{s:/<4}{green}{s:/<2}{blue}{s:/<2}{reset}\n\r",
        s = "/",
        p = " ",
        padding = padding,
        title = title,
        red = color::Fg(color::LightRed),
        yellow = color::Fg(color::LightYellow),
        green = color::Fg(color::LightGreen),
        blue = color::Fg(color::LightBlue),
        reset = color::Fg(color::Reset)
    )
    .unwrap();
}

/// Writes a separator, the `extra_width` parameter allow to match the width
/// of a title separator by passing the title length + padding.
pub fn separator<B: Write>(extra_width: usize, buffer: &mut B) {
    write!(
        buffer,
        "{blue}{s:/<2}{green}{s:/<2}{yellow}{s:/<4}{red}{s:/<width$}{yellow}{s:/<4}{green}{s:/<2}{blue}{s:/<2}{reset}\n\r",
        s = "/",
        width = 16 + extra_width,
        red = color::Fg(color::LightRed),
        yellow = color::Fg(color::LightYellow),
        green = color::Fg(color::LightGreen),
        blue = color::Fg(color::LightBlue),
        reset = color::Fg(color::Reset)
    )
    .unwrap();
}

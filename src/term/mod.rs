use std::io;
use std::io::{stdin, stdout, Write};
use termion::{color, style};

use crate::error::{wrap, Error};

mod diff;
pub mod help;
mod repl;

pub use diff::write_diff;
pub use repl::Input;
pub use repl::Repl;

pub enum SeparatorKind {
    Top,
    Middle,
    Bottom,
    _Standalone,
}

/// Writes a single line to the buffer.
pub fn writeln<B: Write>(message: &str, buffer: &mut B) {
    write!(buffer, "{}\n\r", message).unwrap();
}

/// Asks a binary question to the user. Return true for yes, false for no.
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
pub fn color_box<B: Write>(title: &str, content: &Vec<u8>, buffer: &mut B) {
    box_separator(title, SeparatorKind::Top, buffer);
    buffer.boxed_write(content).unwrap();
    box_separator("", SeparatorKind::Bottom, buffer);
    buffer.flush().unwrap();
}

/// Writes a summary of a given snapshot.
pub fn snap_summary<B: Write>(name: &str, description: Option<&String>, cmd: &str, buffer: &mut B) {
    let bold = style::Bold;
    let reset = style::Reset;
    buffer
        .boxed_write_str(&format!(
            "\
            Test:    {}{}{}\n\
            Command: {}{}{}",
            bold, name, reset, bold, cmd, reset
        ))
        .unwrap();
    if let Some(description) = description {
        buffer
            .boxed_write_str(&format!("\n{}\n", description))
            .unwrap();
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

pub fn box_separator<B: Write>(title: &str, kind: SeparatorKind, buffer: &mut B) {
    let corner = match kind {
        SeparatorKind::Top => '┌',
        SeparatorKind::Middle => '├',
        SeparatorKind::Bottom => '└',
        SeparatorKind::_Standalone => '─',
    };
    write!(
        buffer,
        "{}{}────{} {}{}{}\n\r",
        color::Fg(color::LightBlue),
        corner,
        color::Fg(color::Reset),
        style::Bold,
        title,
        style::Reset
    )
    .unwrap();
}

/// Allows to write boxed messages.
/// Will sanitize line breaks to handle raw terminal mode.
pub trait BoxedWriter: Write {
    fn boxed_write(&mut self, buf: &[u8]) -> io::Result<()>;
    fn boxed_write_str(&mut self, string: &str) -> io::Result<()>;
}

impl<W: Write> BoxedWriter for W {
    fn boxed_write(&mut self, buf: &[u8]) -> io::Result<()> {
        let colorize = color::Fg(color::LightBlue);
        let reset_color = color::Fg(color::Reset);
        for line in buf.split(|c| c == &b'\n') {
            write!(self, "{}│{} ", colorize, reset_color)?;
            self.write_all(line)?;
            self.write_all(&[b'\n', b'\r'])?;
        }
        Ok(())
    }

    fn boxed_write_str(&mut self, string: &str) -> io::Result<()> {
        let colorize = color::Fg(color::LightBlue);
        let reset_color = color::Fg(color::Reset);
        for line in string.lines() {
            write!(self, "{}│{} {}\n\r", colorize, reset_color, line)?;
        }
        Ok(())
    }
}

use std::io;
use std::io::{stdin, stdout, Write};
use termion::{color, style};

use crate::error::{wrap, Error};

mod diff;
pub mod help;
mod repl;
mod theme;

pub use diff::write_diff;
pub use repl::Input;
pub use repl::Repl;
pub use theme::Theme;

pub enum SeparatorKind {
    Top,
    Middle,
    Bottom,
    _Standalone,
}

/// Writes a single line to the buffer.
pub fn writeln<B: Write>(message: &str, buffer: &mut B) {
    write!(buffer, "{}\r\n", message).unwrap();
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

pub fn snap_preview<B: Write>(snap: &std::process::Output, buffer: &mut B, theme: &Theme) {
    box_separator("status code", SeparatorKind::Top, buffer, theme);
    let exit_code = snap.status.code();
    if let Some(code) = exit_code {
        buffer
            .boxed_write_str(&format!("{}{}{}", style::Bold, code, style::Reset), theme)
            .unwrap();
    } else {
        buffer
            .boxed_write_str(&format!("{}None{}", style::Bold, style::Reset), theme)
            .unwrap();
    }
    if snap.stdout.len() > 0 {
        box_separator("stdout", SeparatorKind::Middle, buffer, theme);
        buffer.boxed_write(&snap.stdout, theme).unwrap();
    }
    if snap.stderr.len() > 0 {
        box_separator("stderr", SeparatorKind::Middle, buffer, theme);
        buffer.boxed_write(&snap.stderr, theme).unwrap();
    }
    box_separator("", SeparatorKind::Bottom, buffer, theme);
}

/// Writes a summary of a given snapshot.
pub fn snap_summary<B: Write>(
    description: Option<&String>,
    cmd: &str,
    status_code: Option<i32>,
    buffer: &mut B,
    theme: &Theme,
) {
    let bold = style::Bold;
    let reset = style::Reset;
    let code = if let Some(code) = status_code {
        format!("{}", code)
    } else {
        String::from("None")
    };
    buffer
        .boxed_write_str(
            &format!(
                "\
            cmd:  {}{}{}\n\
            code: {}{}{}",
                bold, cmd, reset, bold, code, reset
            ),
            theme,
        )
        .unwrap();
    if let Some(description) = description {
        buffer.boxed_write_str(&format!("\n{}\n", description), theme).unwrap();
    }
}

/// Writes the success message.
pub fn success<B: Write>(buffer: &mut B) {
    write!(
        buffer,
        "{}{}Success ✓{}{}\r\n",
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
        "{}{}Failure ✗{}{}\r\n",
        color::Fg(color::LightRed),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset)
    )
    .unwrap();
}

/// Draws a separator for boxed messages.
pub fn box_separator<B: Write>(title: &str, kind: SeparatorKind, buffer: &mut B, theme: &Theme) {
    let corner = match kind {
        SeparatorKind::Top => '┌',
        SeparatorKind::Middle => '├',
        SeparatorKind::Bottom => '└',
        SeparatorKind::_Standalone => '─',
    };
    write!(
        buffer,
        "{}{}────{} {}{}{}\r\n",
        theme.blue,
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
    fn boxed_write(&mut self, buf: &[u8], theme: &Theme) -> io::Result<()>;
    fn boxed_write_str(&mut self, string: &str, theme: &Theme) -> io::Result<()>;
}

impl<W: Write> BoxedWriter for W {
    fn boxed_write(&mut self, buf: &[u8], theme: &Theme) -> io::Result<()> {
        let colorize = &theme.blue;
        let reset_color = color::Fg(color::Reset);
        for line in buf.split(|c| c == &b'\n') {
            write!(self, "{}│{} ", colorize, reset_color)?;
            self.write_all(line)?;
            self.write_all(&[b'\n', b'\r'])?;
        }
        Ok(())
    }

    fn boxed_write_str(&mut self, string: &str, theme: &Theme) -> io::Result<()> {
        let colorize = &theme.blue;
        let reset_color = color::Fg(color::Reset);
        for line in string.lines() {
            write!(self, "{}│{} {}\r\n", colorize, reset_color, line)?;
        }
        Ok(())
    }
}

use std::io::{stdout, stdin, Write};
use termion::color;

use crate::error::{Error, wrap};

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
            return Ok(true)
        }
        if buffer =="no" || buffer == "n" {
            return Ok(false)
        }
    }
}

/// Draw a colored box with a title and a given content.
pub fn color_box(title: &str, content: &Vec<u8>) {
    let mut stdout = stdout();
    writeln!(
        stdout,
        "{blue}{s:/<2}{green}{s:/<2}{yellow}{s:/<4}{red}{s:/<7}{reset} {title} {red}{s:/<7}{yellow}{s:/<4}{green}{s:/<2}{blue}{s:/<2}{reset}\n",
        s = "/",
        title = title,
        red = color::Fg(color::LightRed),
        yellow = color::Fg(color::LightYellow),
        green = color::Fg(color::LightGreen),
        blue = color::Fg(color::LightBlue),
        reset = color::Fg(color::Reset)
    )
    .unwrap();
    stdout.write_all(content).unwrap();
    writeln!(
        stdout,
        "\n{blue}{s:/<2}{green}{s:/<2}{yellow}{s:/<4}{red}{s:/<width$}{yellow}{s:/<4}{green}{s:/<2}{blue}{s:/<2}{reset}",
        s = "/",
        width = 16 + title.len(),
        red = color::Fg(color::LightRed),
        yellow = color::Fg(color::LightYellow),
        green = color::Fg(color::LightGreen),
        blue = color::Fg(color::LightBlue),
        reset = color::Fg(color::Reset)
    )
    .unwrap();
    stdout.flush().unwrap();
}


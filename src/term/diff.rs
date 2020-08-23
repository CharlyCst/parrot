use std::io::Write;
use termion::color;

use crate::diff::{get_diff, DiffLine};

/// Writes the diff between two snapshots to buffer.
pub fn write_diff<B: Write>(old: &Vec<u8>, new: &Vec<u8>, buffer: &mut B) {
    let old_lines: Vec<&[u8]> = old.split(|byte| *byte == b'\n').collect();
    let new_lines: Vec<&[u8]> = new.split(|byte| *byte == b'\n').collect();
    let diff = get_diff(&old_lines, &new_lines);
    // Define colors
    let bg_color = color::Bg(color::Black);
    let bg_reset = color::Bg(color::Reset);
    let fg_green = color::Fg(color::LightGreen);
    let fg_red = color::Fg(color::LightRed);
    let fg_reset = color::Fg(color::Reset);
    // Display diff
    for line in diff {
        match line {
            DiffLine::Keep(bytes) => {
                write!(buffer, "{} {} ", bg_color, bg_reset).unwrap();
                buffer.write_all(bytes).unwrap();
                write!(buffer, "\n\r").unwrap();
            }
            DiffLine::Delete(bytes) => {
                write!(buffer, "{}{}-{} ", bg_color, fg_red, fg_reset).unwrap();
                buffer.write_all(bytes).unwrap();
                write!(buffer, "{}\n\r", bg_reset).unwrap();
            }
            DiffLine::Insert(bytes) => {
                write!(buffer, "{}{}+{} ", bg_color, fg_green, fg_reset).unwrap();
                buffer.write_all(bytes).unwrap();
                write!(buffer, "{}\n\r", bg_reset).unwrap();
            }
        }
    }
}

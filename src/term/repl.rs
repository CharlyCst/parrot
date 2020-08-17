use std::io::Write;
use termion::{color, style};

use crate::driver::View;

/// Displays the REPL interface.
pub fn repl<B: Write>(view: &View, buffer: &mut B) {
    display_list(view, buffer);
    display_input(buffer);
}

/// Displays the REPL snapshot list.
fn display_list<B: Write>(view: &View, buffer: &mut B) {
    // Style & colors
    let bg = color::Bg(color::Black);
    let clear_bg = color::Bg(color::Reset);
    let red = color::Fg(color::LightRed);
    let clear_red = color::Fg(color::Reset);
    let bold = style::Bold;
    let clear_bold = style::Reset;

    let (min, max) = view.window;
    for (pos, snap) in view.get_view()[min..max].iter().enumerate() {
        if pos == view.cursor {
            writeln!(buffer, "{}{}{}>{} {}{}{}", bg, bold, red, clear_red, snap.name,  clear_bold,clear_bg).unwrap();
        } else {
            writeln!(buffer, "{} {} {}", bg, clear_bg, snap.name).unwrap();
        };
    }
    for _ in (max - min)..view.height {
        writeln!(buffer, "{} {}", bg, clear_bg).unwrap();
    }
    writeln!(buffer, "").unwrap();
}

fn display_input<B: Write>(buffer: &mut B) {
    let blue = color::Fg(color::LightBlue);
    let clear_blue = color::Fg(color::Reset);
    let bold = style::Bold;
    let clear_bold = style::Reset;
    writeln!(buffer, "{}{}>{} {}", bold, blue, clear_blue, clear_bold).unwrap();
}

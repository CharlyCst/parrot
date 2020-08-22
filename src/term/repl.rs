use std::io::{Stdin, Stdout, Write};
use termion::clear;
use termion::cursor;
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::terminal_size;
use termion::{color, style};

use crate::driver::View;

pub enum Input {
    Up,
    Down,
    Quit,
    Command(String),
}

pub struct Repl {
    /// Using raw mode stdout
    pub stdout: RawTerminal<Stdout>,
    stdin: Keys<Stdin>,
    input: String,
}

impl Repl {
    /// Initialize the REPL internal state.
    pub fn new(stdin: Stdin, stdout: Stdout) -> Repl {
        let mut stdout = stdout.into_raw_mode().unwrap();
        let stdin = stdin.keys();
        let input = String::from("");
        write!(stdout, "{}", cursor::Save).unwrap();
        Repl {
            stdout,
            stdin,
            input,
        }
    }

    /// Clears the REPL from the screen.
    pub fn clear(&mut self) {
        write!(self.stdout, "{}{}", cursor::Restore, clear::AfterCursor).unwrap();
    }

    /// Writes a line
    pub fn writeln(&mut self, msg: &str) {
        write!(self.stdout, "{}\n\r", msg).unwrap()
    }

    /// Saves the cursor position, everything before the cursor will be
    /// preserved from any upcoming clear.
    pub fn checkpoint(&mut self) {
        write!(self.stdout, "{}", cursor::Save).unwrap();
    }

    /// Runs the REPL and returns control once a command has been received.
    pub fn run(&mut self, view: &View) -> Input {
        self.render(view);
        loop {
            let key = match self.stdin.next() {
                Some(key) => key.unwrap(),
                None => return Input::Quit,
            };
            match key {
                Key::Down => return Input::Down,
                Key::Up => return Input::Up,
                Key::Esc => return Input::Quit,
                Key::Delete | Key::Backspace => {
                    self.input.pop();
                    self.render(view);
                }
                Key::Char('\n') => {
                    if self.input.len() > 0 {
                        let mut command = String::new();
                        std::mem::swap(&mut self.input, &mut command);
                        return Input::Command(command);
                    }
                }
                Key::Char(c) => {
                    self.input.push(c);
                    self.render(view);
                }
                _ => (),
            }
        }
    }

    /// Displays the REPL.
    fn render(&mut self, view: &View) {
        self.clear();
        self.display_description_box(view);
        self.display_list(view);
        self.display_input();
        self.stdout.flush().unwrap();
    }

    /// Displays the REPL snapshot list.
    fn display_list(&mut self, view: &View) {
        // Style & colors
        let bg = color::Bg(color::Black);
        let clear_bg = color::Bg(color::Reset);
        let red = color::Fg(color::LightRed);
        let clear_red = color::Fg(color::Reset);
        let bold = style::Bold;
        let clear_bold = style::Reset;

        let (min, max) = view.window;
        let data = view.get_view();
        for (pos, snap) in data[min..max].iter().enumerate() {
            if pos == view.cursor {
                write!(
                    self.stdout,
                    "{}{}{}>{}  {}{}{}\n\r",
                    bg, bold, red, clear_red, snap.name, clear_bold, clear_bg
                )
                .unwrap();
            } else {
                write!(self.stdout, "{} {}  {}\n\r", bg, clear_bg, snap.name).unwrap();
            };
        }
        for _ in (max - min)..view.height {
            write!(self.stdout, "{} {}\n\r", bg, clear_bg).unwrap();
        }
        write!(
            self.stdout,
            "  {}{}/{}{}\n\r",
            color::Fg(color::White),
            data.len(),
            view.get_total_item_count(),
            color::Fg(color::Reset)
        )
        .unwrap();
    }

    fn display_input(&mut self) {
        let blue = color::Fg(color::LightBlue);
        let clear_blue = color::Fg(color::Reset);
        let bold = style::Bold;
        let clear_bold = style::Reset;
        write!(
            self.stdout,
            "{}{}>{} {}{}",
            bold, blue, clear_blue, self.input, clear_bold
        )
        .unwrap();
    }

    fn display_description_box(&mut self, view: &View) {
        let (w, _) = terminal_size().unwrap_or((80, 24));

        // Style & colors
        let bold = style::Bold;
        let reset_style = style::Reset;
        let red = color::Fg(color::LightRed);
        let yellow = color::Fg(color::LightYellow);
        let green = color::Fg(color::LightGreen);
        let blue = color::Fg(color::LightBlue);
        let reset_color = color::Fg(color::Reset);

        // Compute sizes
        let w = (w - 2) as usize; // remove 2 units for the box's borders
        let red_width = w / 3;
        let yellow_width = w / 6;
        let green_width = w / 9;
        let blue_width = w / 18;
        let red_width =
            red_width + (w - red_width - 2 * yellow_width - 2 * green_width - 2 * blue_width);
        let cmd_width = w - 7;
        let desc_width = w - 2;
        let name_width = 2 * green_width + 2 * yellow_width + red_width;

        // Get snapshot content
        let snap = view.get_selected();
        let (name, cmd, descs) = if let Some(snap) = snap {
            let desc = if let Some(ref desc) = snap.description {
                // Get descriptions lines and truncate them
                let mut descs = desc.lines();
                let (d1, d2, d3) = (
                    descs.next().unwrap_or(""),
                    descs.next().unwrap_or(""),
                    descs.next().unwrap_or(""),
                );
                let max_w = desc_width;
                (
                    &d1[..std::cmp::min(d1.len(), max_w)],
                    &d2[..std::cmp::min(d2.len(), max_w)],
                    &d3[..std::cmp::min(d3.len(), max_w)],
                )
            } else {
                ("", "", "")
            };
            let cmd = &snap.cmd[..std::cmp::min(snap.cmd.len(), cmd_width)];
            let name = &snap.name[..std::cmp::min(snap.name.len(), name_width - 2)];
            (name, cmd, desc)
        } else {
            ("", "", ("", "", ""))
        };

        // Build the top border
        let n = name.len() + 2;
        let top_border = if n < green_width {
            let remainder = green_width - n;
            format!("{b}┌{x:─<bw$}{rc} {bold}{}{rs} {g}{x:─<rm$}{y}{x:─<yw$}{r}{x:─<rw$}{y}{x:─<yw$}{g}{x:─<gw$}{b}{x:─<bw$}┐{rc}",
                name,
                x = "", // Placeholder
                bold = bold,
                rs = reset_style,
                r = red,
                y = yellow,
                g = green,
                b = blue,
                rc = reset_color,
                bw = blue_width,
                gw = green_width,
                yw = yellow_width,
                rw = red_width,
                rm = remainder,
            )
        } else if n < green_width + yellow_width {
            let remainder = green_width + yellow_width - n;
            format!("{b}┌{x:─<bw$}{rc} {bold}{}{rs} {y}{x:─<rm$}{r}{x:─<rw$}{y}{x:─<yw$}{g}{x:─<gw$}{b}{x:─<bw$}┐{rc}",
                name,
                x = "", // Placeholder
                bold = bold,
                rs = reset_style,
                r = red,
                y = yellow,
                g = green,
                b = blue,
                rc = reset_color,
                bw = blue_width,
                gw = green_width,
                yw = yellow_width,
                rw = red_width,
                rm = remainder,
            )
        } else if n < green_width + yellow_width + red_width {
            let remainder = green_width + yellow_width + red_width - n;
            format!("{b}┌{x:─<bw$}{rc} {bold}{}{rs} {r}{x:─<rm$}{y}{x:─<yw$}{g}{x:─<gw$}{b}{x:─<bw$}┐{rc}",
                name,
                x = "", // Placeholder
                bold = bold,
                rs = reset_style,
                r = red,
                y = yellow,
                g = green,
                b = blue,
                rc = reset_color,
                bw = blue_width,
                gw = green_width,
                yw = yellow_width,
                rm = remainder,
            )
        } else if n < green_width + 2 * yellow_width + red_width {
            let remainder = green_width + 2 * yellow_width + red_width - n;
            format!(
                "{b}┌{x:─<bw$}{rc} {bold}{}{rs} {y}{x:─<rm$}{g}{x:─<gw$}{b}{x:─<bw$}┐{rc}",
                name,
                x = "", // Placeholder
                bold = bold,
                rs = reset_style,
                y = yellow,
                g = green,
                b = blue,
                rc = reset_color,
                bw = blue_width,
                gw = green_width,
                rm = remainder,
            )
        } else {
            let remainder = 2 * green_width + 2 * yellow_width + red_width - n;
            format!(
                "{b}┌{x:─<bw$}{rc} {bold}{}{rs} {g}{x:─<rm$}{b}{x:─<bw$}┐{rc}",
                name,
                x = "", // Placeholder
                bold = bold,
                rs = reset_style,
                g = green,
                b = blue,
                rc = reset_color,
                bw = blue_width,
                rm = remainder,
            )
        };

        // Write down the description box
        write!(
            self.stdout,
            "\
            {top_border}\n\r\
            {b}│{rc} cmd: {bold}{cmd:<cmd_width$}{rs} {b}│{rc}\n\r\
            {b}│{rc} {desc_1:<desc_width$} {b}│{rc}\n\r\
            {b}│{rc} {desc_2:<desc_width$} {b}│{rc}\n\r\
            {b}│{rc} {desc_3:<desc_width$} {b}│{rc}\n\r\
            {b}└{x:─<bw$}{g}{x:─<gw$}{y}{x:─<yw$}{r}{x:─<rw$}{y}{x:─<yw$}{g}{x:─<gw$}{b}{x:─<bw$}┘{rc}\n\r\
            ",
            top_border = top_border,
            x = "", // Placeholder
            bold = bold,
            rs = reset_style,
            r = red,
            y = yellow,
            g = green,
            b = blue,
            rc = reset_color,
            bw = blue_width,
            gw = green_width,
            yw = yellow_width,
            rw = red_width,
            cmd = cmd,
            cmd_width = cmd_width,
            desc_1 = descs.0,
            desc_2 = descs.1,
            desc_3 = descs.2,
            desc_width = desc_width,
        )
        .unwrap();
    }
}

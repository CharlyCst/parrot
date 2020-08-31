use std::io::{BufWriter, Stdin, Stdout, Write};
use termion::clear;
use termion::cursor;
use termion::cursor::DetectCursorPos;
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
    pub stdout: RawTerminal<BufWriter<Stdout>>,
    stdin: Keys<Stdin>,
    input: String,
    cursor_pos: (u16, u16),
    height: u16,
}

impl Repl {
    /// Initialize the REPL internal state.
    pub fn new(stdin: Stdin, stdout: Stdout) -> Repl {
        let mut stdout = BufWriter::new(stdout).into_raw_mode().unwrap();
        let stdin = stdin.keys();
        let input = String::from("");
        write!(stdout, "{}", cursor::Save).unwrap();
        let cursor_pos = stdout.cursor_pos().unwrap();
        let mut repl = Repl {
            stdout,
            stdin,
            input,
            cursor_pos,
            height: 8 + 5,
        };
        repl.restore();
        repl
    }

    /// Clears the REPL from the screen.
    fn clear(&mut self) {
        write!(self.stdout, "{}{}", cursor::Restore, clear::AfterCursor).unwrap();
    }

    /// Suspend REPL mode, stdout can be used normally until restored.
    /// The repl should not be use while suspended.
    pub fn suspend(&mut self) {
        self.clear();
        self.stdout.flush().unwrap();
    }

    /// Restore REPL mode, the repl can be re-started safely.
    pub fn restore(&mut self) {
        let (_, cursor_y) = self.stdout.cursor_pos().unwrap();
        let (_, term_height) = terminal_size().unwrap();
        // If the cursor reached the bottom, make some space
        if term_height >= self.height && term_height - cursor_y < self.height {
            write!(
                self.stdout,
                "{}{}{}",
                "\n\r".repeat(self.height as usize),
                cursor::Goto(1, term_height - self.height),
                clear::AfterCursor
            )
            .unwrap();
        }
        self.checkpoint();
    }

    /// Writes a single line to the output. The REPL must have been suspended.
    pub fn writeln(&mut self, message: &str) {
        write!(self.stdout, "{}\n\r", message).unwrap();
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
                Key::Ctrl('l') => {
                    self.clear();
                    write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
                    self.checkpoint();
                    self.render(view);
                }
                _ => (),
            }
        }
    }

    /// Saves the cursor position, everything before the cursor will be
    /// preserved from any upcoming clear.
    fn checkpoint(&mut self) {
        write!(self.stdout, "{}", cursor::Save).unwrap();
        let cursor_pos = self.stdout.cursor_pos().unwrap();
        self.cursor_pos = cursor_pos;
    }

    /// Displays the REPL.
    fn render(&mut self, view: &View) {
        self.clear();
        self.display_description_box(view);
        let input_offset = self.display_input();
        self.display_list(view);
        let (x, y) = self.cursor_pos;
        let y = y + 6;
        let x = x + input_offset;
        write!(self.stdout, "{}", cursor::Goto(x, y)).unwrap();
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
            let snap = snap.borrow();
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
        let current = if data.len() == 0 {
            0
        } else {
            min + view.cursor + 1
        };
        write!(
            self.stdout,
            "  {}{}/{}{}",
            color::Fg(color::White),
            current,
            data.len(),
            color::Fg(color::Reset)
        )
        .unwrap();
    }

    /// Displays the input, return the offset of the input line.
    fn display_input(&mut self) -> u16 {
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
        write!(self.stdout, "\n\r").unwrap();
        2 + self.input.chars().count() as u16
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
        let (name, cmd, descs) = if let Some(snap) = snap.as_ref() {
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
        let first_green_width;
        let mut first_yellow_width = yellow_width;
        let mut first_red_width = red_width;
        let mut second_yellow_width = yellow_width;
        let mut second_green_width = green_width;
        if n >= green_width {
            first_green_width = 0;
            if n >= green_width + yellow_width {
                first_yellow_width = 0;
                if n >= green_width + yellow_width + red_width {
                    first_red_width = 0;
                    if n >= green_width + 2 * yellow_width + red_width {
                        second_yellow_width = 0;
                        second_green_width = 2 * green_width + 2 * yellow_width + red_width - n;
                    } else {
                        second_yellow_width = green_width + 2 * yellow_width + red_width - n;
                    }
                } else {
                    first_red_width = green_width + yellow_width + red_width - n;
                }
            } else {
                first_yellow_width = green_width + yellow_width - n;
            }
        } else {
            first_green_width = green_width - n;
        }
        let top_border = format!("{b}┌{x:─<bw$}{rc} {bold}{}{rs} {g}{x:─<fgw$}{y}{x:─<fyw$}{r}{x:─<frw$}{y}{x:─<syw$}{g}{x:─<sgw$}{b}{x:─<bw$}┐{rc}",
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
            fgw = first_green_width,
            fyw = first_yellow_width,
            frw = first_red_width,
            syw = second_yellow_width,
            sgw = second_green_width,
        );

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

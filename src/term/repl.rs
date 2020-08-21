use std::io::{Stdin, Stdout, Write};
use termion::clear;
use termion::cursor;
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
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
}

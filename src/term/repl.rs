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
}

pub struct Repl {
    /// Using raw mode stdout
    pub stdout: RawTerminal<Stdout>,
    stdin: Keys<Stdin>,
}

impl Repl {
    /// Displays the REPL interface.
    pub fn new(stdin: Stdin, stdout: Stdout) -> Repl {
        let stdout = stdout.into_raw_mode().unwrap();
        let stdin = stdin.keys();
        Repl { stdout, stdin }
    }

    pub fn run(&mut self, view: &View) -> Input {
        write!(self.stdout, "{}{}", clear::BeforeCursor, cursor::Goto(1, 1)).unwrap();
        self.display_list(view);
        self.display_input();
        self.stdout.flush().unwrap();
        for key in &mut self.stdin {
            match key.unwrap() {
                Key::Down => return Input::Down,
                Key::Up => return Input::Up,
                Key::Esc => return Input::Quit,
                _ => (),
            }
        }
        Input::Quit
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
        for (pos, snap) in view.get_view()[min..=max].iter().enumerate() {
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
        for _ in (max - min + 1)..view.height {
            write!(self.stdout, "{} {}\n\r", bg, clear_bg).unwrap();
        }
        write!(
            self.stdout,
            "  {}{}/{}{}\n\r",
            color::Fg(color::White),
            view.nb_items,
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
            "{}{}>{} {}",
            bold, blue, clear_blue, clear_bold
        )
        .unwrap();
    }
}

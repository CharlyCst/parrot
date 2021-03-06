use std::io::Write;
use termion::color;
use termion::style;

/// Writes the help message.
pub fn write_help<B: Write>(buffer: &mut B) {
    let bold = style::Bold;
    let reset_style = style::Reset;
    let red = color::Fg(color::LightRed);
    let yellow = color::Fg(color::LightYellow);
    let green = color::Fg(color::LightGreen);
    let blue = color::Fg(color::LightBlue);
    let reset_color = color::Fg(color::Reset);
    write!(
        buffer,
        "{b}┌──{g}──────{y}──────────{r}────────────────────────────{y}──────────{g}──────{b}──┐{rc}\r\n\
        {b}│{rc} {bold}Parrot script cheat-sheet{rs}                                      {b}│{rc}\r\n\
        {b}│{rc}                                                                {b}│{rc}\r\n\
        {b}│{rc} {bold}clear   c{rs}  Remove any filter                                   {b}│{rc}\r\n\
        {b}│{rc} {bold}edit    e{rs}  Edit the name or description                        {b}│{rc}\r\n\
        {b}│{rc} {bold}filter  f{rs}  Filter by name (contains) or by #tag (exact match)  {b}│{rc}\r\n\
        {b}│{rc} {bold}help    h{rs}  Print this help                                     {b}│{rc}\r\n\
        {b}│{rc} {bold}quit    q{rs}  Exit from Parrot REPL                               {b}│{rc}\r\n\
        {b}│{rc} {bold}run     r{rs}  Run the selected test, or all tests by passing '*'  {b}│{rc}\r\n\
        {b}│{rc} {bold}show    s{rs}  Show the selected test, or all tests by passing '*' {b}│{rc}\r\n\
        {b}└──{g}──────{y}──────────{r}────────────────────────────{y}──────────{g}──────{b}──┘{rc}\r\n\
        ",
        bold = bold,
        rs = reset_style,
        r = red,
        y = yellow,
        g = green,
        b = blue,
        rc = reset_color
    )
    .unwrap();
}

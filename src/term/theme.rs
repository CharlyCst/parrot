use std::env;
use termion::color;

pub struct Theme {
    pub red: String,
    pub yellow: String,
    pub green: String,
    pub blue: String,
    pub cursor: String,
    pub input: String,
}

impl Theme {
    pub fn new() -> Self {
        let theme = env::var("PARROT_THEME").unwrap_or("scarlet".to_string());
        match theme.as_str() {
            "scarlet" => Theme {
                red: color::Rgb(241, 9, 6).fg_string(),
                yellow: color::Rgb(254, 222, 18).fg_string(),
                green: color::Rgb(54, 178, 52).fg_string(),
                blue: color::Rgb(59, 99, 172).fg_string(),
                cursor: color::Rgb(241, 9, 6).fg_string(),
                input: color::Rgb(59, 99, 172).fg_string(),
            },
            "blue-and-yellow" => Theme {
                red: color::Rgb(22, 157, 215).fg_string(),
                yellow: color::Rgb(22, 157, 215).fg_string(),
                green: color::Rgb(255, 211, 47).fg_string(),
                blue: color::Rgb(255, 211, 47).fg_string(),
                cursor: color::Rgb(22, 157, 215).fg_string(),
                input: color::Rgb(255, 211, 47).fg_string(),
            },
            "hyacinth" => Theme {
                red: color::Rgb(74, 95, 188).fg_string(),
                yellow: color::Rgb(74, 95, 188).fg_string(),
                green: color::Rgb(74, 95, 188).fg_string(),
                blue: color::Rgb(74, 95, 188).fg_string(),
                cursor: color::Rgb(255, 204, 85).fg_string(),
                input: color::Rgb(74, 95, 188).fg_string(),
            },
            "military" => Theme {
                red: color::Rgb(109, 207, 60).fg_string(),
                yellow: color::Rgb(109, 207, 60).fg_string(),
                green: color::Rgb(109, 207, 60).fg_string(),
                blue: color::Rgb(42, 200, 255).fg_string(),
                cursor: color::Rgb(109, 207, 60).fg_string(),
                input: color::Rgb(59, 99, 172).fg_string(),
            },
            "gray" => Theme {
                red: color::Rgb(177, 176, 194).fg_string(),
                yellow: color::Rgb(177, 176, 194).fg_string(),
                green: color::Rgb(177, 176, 194).fg_string(),
                blue: color::Rgb(177, 176, 194).fg_string(),
                cursor: color::Rgb(177, 176, 194).fg_string(),
                input: color::Rgb(59, 99, 172).fg_string(),
            },
            "yellow-crested" => Theme {
                red: color::Rgb(177, 176, 194).fg_string(),
                yellow: color::Rgb(177, 176, 194).fg_string(),
                green: color::Rgb(177, 176, 194).fg_string(),
                blue: color::Rgb(235, 226, 95).fg_string(),
                cursor: color::Rgb(235, 226, 95).fg_string(),
                input: color::Rgb(59, 99, 172).fg_string(),
            },
            "ansi" | _ => Theme {
                red: color::LightRed.fg_str().to_string(),
                yellow: color::LightYellow.fg_str().to_string(),
                green: color::LightGreen.fg_str().to_string(),
                blue: color::LightBlue.fg_str().to_string(),
                cursor: color::LightRed.fg_str().to_string(),
                input: color::LightBlue.fg_str().to_string(),
            },
        }
    }
}

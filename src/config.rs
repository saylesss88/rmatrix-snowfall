use clap::{ArgAction, Parser};
use crossterm::style::Color;

#[derive(Debug, Parser)]
#[command(author, version, about)]
#[allow(clippy::struct_excessive_bools)]
struct Opt {
    /// Bold characters on (can be used multiple times: -b, -bb)
    #[arg(short = 'b', action = ArgAction::Count)]
    bold: u8,

    /// Linux mode (use matrix console font)
    #[arg(short = 'l', long = "console")]
    console: bool,

    /// Use old-style scrolling
    #[arg(short = 'o', long = "oldstyle")]
    oldstyle: bool,

    /// "Screensaver" mode, exits on first keystroke
    #[arg(short = 's', long = "screensaver")]
    screensaver: bool,

    /// X window mode, use if your xterm is using mtx.pcf
    #[arg(short = 'x', long = "xwindow")]
    xwindow: bool,

    /// Screen update delay
    #[arg(
        short = 'u',
        long = "update",
        default_value = "6",
        value_parser = validate_update
    )]
    update: usize,

    /// Colour of the snowfall
    #[arg(
        short = 'C',
        long = "colour",
        default_value = "blue",
        value_parser = ["green", "red", "blue", "white", "yellow", "cyan", "magenta", "black"]
    )]
    colour: String,

    /// Rainbow mode
    #[arg(short = 'r', long = "rainbow")]
    rainbow: bool,
}

/// Validates that the update speed is between 1 and 10
fn validate_update(n: &str) -> Result<usize, String> {
    let val = n
        .parse::<usize>()
        .map_err(|_| "must be a number".to_string())?;
    if (1..=10).contains(&val) {
        Ok(val)
    } else {
        Err("must be between 1 and 10".to_string())
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub bold: isize,
    pub console: bool,
    pub oldstyle: bool,
    pub screensaver: bool,
    pub xwindow: bool,
    pub update: usize,
    pub colour: Color,
    pub rainbow: bool,
    pub pause: bool,
}

impl Default for Config {
    fn default() -> Self {
        // In Clap v4, we use parse() instead of from_args()
        let opt = Opt::parse();

        let colour = match opt.colour.as_str() {
            "green" => Color::Green,
            "red" => Color::Red,
            "white" => Color::White,
            "yellow" => Color::Yellow,
            "cyan" => Color::Cyan,
            "magenta" => Color::Magenta,
            "black" => Color::Black,
            _ => Color::Blue,
        };

        Self {
            bold: opt.bold as isize,
            console: opt.console,
            oldstyle: opt.oldstyle,
            screensaver: opt.screensaver,
            xwindow: opt.xwindow,
            update: opt.update,
            rainbow: opt.rainbow,
            colour,
            pause: false,
        }
    }
}

impl Config {
    pub const fn handle_keypress(&mut self, keypress: char) -> bool {
        if self.screensaver {
            return true;
        }

        match keypress {
            'q' => return true,
            'b' => self.bold = 1,
            'B' => self.bold = 2,
            'n' => self.bold = 0,
            '!' => {
                self.colour = Color::Red;
                self.rainbow = false;
            }
            '@' => {
                self.colour = Color::Green;
                self.rainbow = false;
            }
            '#' => {
                self.colour = Color::Yellow;
                self.rainbow = false;
            }
            '$' => {
                self.colour = Color::Blue;
                self.rainbow = false;
            }
            '%' => {
                self.colour = Color::Magenta;
                self.rainbow = false;
            }
            'r' => self.rainbow = true,
            '^' => {
                self.colour = Color::Cyan;
                self.rainbow = false;
            }
            '&' => {
                self.colour = Color::White;
                self.rainbow = false;
            }
            'p' | 'P' => self.pause = !self.pause,
            '1'..='9' => self.update = (keypress as usize).saturating_sub(48),
            '0' => self.update = 0,
            _ => {}
        }
        false
    }
}

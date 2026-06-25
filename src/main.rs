use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use r_matrix_snowfall::{
    config::{Config, Opt},
    Matrix,
};
use std::io::stdout;
use std::time::Duration;

/// A terminal-based screensaver matrix snowfall
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {}

fn main() -> std::io::Result<()> {
    let opt = Opt::parse();
    // 1. Setup
    let mut config = Config::from(opt);
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut matrix = Matrix::default();
    let mut running = true;

    // 2. Loop
    while running {
        // Poll for input (non-blocking)
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    if config.handle_keypress(c) {
                        running = false;
                    }
                }
                Event::Resize(_, _) => {
                    matrix.resize();
                    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                }
                _ => {}
            }
        }

        // Draw
        if !config.pause {
            matrix.arrange(&config);
            matrix.draw(&mut stdout, &config)?;
        }
    }

    // 3. Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

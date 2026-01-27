use crossterm::{
    cursor, queue,
    style::{self, Color, Stylize},
    terminal,
};
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{self, Write};

pub mod config;
use config::Config;

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}

fn rng<T>() -> T
where
    Standard: Distribution<T>,
{
    RNG.with(|rng| (*rng).borrow_mut().r#gen::<T>())
}

fn rand_char() -> char {
    // Mix of snowflakes and lambda symbols for NixOS theme
    let snowflake_chars = ['❄', '❅', '❆', '*', '·', '•', 'λ', 'Λ'];
    RNG.with(|rng| {
        let idx = (*rng).borrow_mut().r#gen::<usize>() % snowflake_chars.len();
        snowflake_chars[idx]
    })
}

#[derive(Clone)]
pub struct Block {
    val: char,
    white: bool,
    color: Color,
}

impl Block {
    fn is_space(&self) -> bool {
        self.val == ' '
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            val: ' ',
            white: false,
            color: COLOR::RED,
        }
    }
}

pub struct Column {
    length: usize,        // The length of the stream
    spaces: usize,        // The spaces between streams
    col: VecDeque<Block>, // The actual column
}

impl Column {
    /// Return a column keyed by a random number generator
    fn new(lines: usize) -> Self {
        Column {
            length: rng::<usize>() % (lines - 1) + 1, // Shorter streams
            spaces: rng::<usize>() % (lines * 2) + 1, // More spacing
            col: (0..lines).map(|_| Block::default()).collect(),
        }
    }
    fn head_is_empty(&self) -> bool {
        self.col[1].val == ' '
    }
    fn new_rand_char(&mut self) {
        self.col[0].val = rand_char();
        self.col[0].color = self.col[1].color;
    }
    fn new_rand_head(&mut self, config: &Config) {
        self.col[0].val = rand_char();
        self.col[0].color = if config.rainbow {
            match rng::<usize>() % 6 {
                0 => Color::Green,
                1 => Color::Blue,
                2 => Color::White,
                3 => Color::Yellow,
                4 => Color::Cyan,
                5 => Color::Magenta,
                _ => unreachable!(),
            }
        } else {
            config.colour
        };
        self.col[0].white = rng::<u8>() < 64;
    }
}

impl std::ops::Index<usize> for Column {
    type Output = Block;
    fn index(&self, i: usize) -> &Self::Output {
        &self.col[i]
    }
}

pub struct Matrix {
    m: Vec<Column>,
}

impl std::ops::Index<usize> for Matrix {
    type Output = Column;
    fn index(&self, i: usize) -> &Self::Output {
        &self.m[i]
    }
}

impl Default for Matrix {
    /// Create a new matrix with the dimensions of the screen
    fn default() -> Self {
        // Get the screen dimensions
        let (lines, cols) = get_term_size();

        // Create the matrix
        Matrix {
            m: (0..cols).map(|_| Column::new(lines)).collect(),
        }
    }
}

impl Matrix {
    fn num_columns(&self) -> usize {
        self.m.len()
    }

    fn num_lines(&self) -> usize {
        self[0].col.len()
    }

    /// Make the next iteration of matrix
    pub fn arrange(&mut self, config: &Config) {
        let lines = self.num_lines();

        self.m.iter_mut().for_each(|col| {
            if col.head_is_empty() && col.spaces != 0 {
                // Decrement the spaces until the next stream starts
                col.spaces -= 1;
            } else if col.head_is_empty() && col.spaces == 0 {
                // Start a new stream
                col.new_rand_head(config);

                // Decrement length of stream
                col.length -= 1;

                // Reset number of spaces until next stream
                col.spaces = rng::<usize>() % lines + 1;
            } else if col.length != 0 {
                // Continue producing stream
                col.new_rand_char();
                col.length -= 1;
            } else {
                // Display spaces until next stream
                col.col[0].val = ' ';
                col.length = rng::<usize>() % (lines - 1) + 1; // Shorter streams
            }
        });
        if config.oldstyle {
            self.old_style_move_down();
        } else {
            self.move_down();
        }
    }
    fn move_down(&mut self) {
        self.m.iter_mut().for_each(|col| {
            // Reset for each column
            let mut in_stream = false;

            let mut last_was_white = false; // Keep track of white heads
            let mut running_color = COLOR_CYAN;

            col.col.iter_mut().for_each(|block| {
                if !in_stream {
                    if !block.is_space() {
                        block.val = ' ';
                        in_stream = true; // We're now in a stream
                        running_color = block.color;
                    }
                } else if block.is_space() {
                    // New rand char for head of stream
                    block.val = rand_char();
                    block.white = last_was_white;
                    in_stream = false;
                }
                // Swapped to "pass on" whiteness and prepare the variable for the next iteration
                std::mem::swap(&mut last_was_white, &mut block.white);
                block.color = running_color;
            })
        })
    }
    fn old_style_move_down(&mut self) {
        // Iterate over all columns and swap spaces
        self.m.iter_mut().for_each(|col| {
            col.col.pop_back();
            col.col.push_back(Block::default()); // Put a Blank space at the head.
            col.col.rotate_right(1)
        });
    }
    /// Draw the matrix on the screen
    pub fn draw(&self, w: &mut impl Write, config: &Config) -> io::Result<()> {
        for j in 1..self.num_lines() {
            for i in 0..self.num_columns() {
                let block = &self[i][j];

                // Skip drawing spaces to optimize?
                // Ncurses overwrites; crossterm needs explicit handling or just overwrite.

                let color = if block.white {
                    Color::White
                } else {
                    block.color
                };

                // Bounds check to prevent panic
                // Crossterm is 0-indexed for 0.28+ (usually), but safe to clamp
                let x = (2 * i) as u16;
                let y = (j as u16).saturating_sub(1);

                // Simple optimization: don't draw if off screen
                // But we need to check terminal size dynamically or assume
                // the matrix size matches the terminal.

                queue!(
                    w,
                    cursor::MoveTo(x, y),
                    style::SetForegroundColor(color),
                    style::Print(block.val)
                )?;
            }
        }
        w.flush()?;
        // Sleep is handled in main loop in crossterm usually, but we can keep it here or move it.
        // Ncurses 'napms' -> std::thread::sleep
        std::thread::sleep(std::time::Duration::from_millis(config.update as u64 * 10));
        Ok(())
    }

    pub fn resize(&mut self) {
        *self = Matrix::default();
    }
}

fn get_term_size() -> (usize, usize) {
    match term_size::dimensions() {
        Some((mut width, mut height)) => {
            // Minimum size for terminal
            if width < 10 {
                width = 10
            }
            if height < 10 {
                height = 10
            }
            if width % 2 != 0 {
                // Makes odd-columned screens print on the rightmost edge
                (height + 1, (width / 2) + 1)
            } else {
                (height + 1, width / 2)
            }
        }
        None => (10, 10),
    }
}

use crossterm::{
    cursor, queue,
    style::{self, Color},
};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{self, Write};

pub mod config;
use config::Config;

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(rand::make_rng());
}

fn rand_char() -> char {
    let snowflake_chars = ['❄', '❅', '❆', '*', '·', '•', 'λ', 'Λ'];
    RNG.with(|rng| {
        let mut r = rng.borrow_mut();
        let idx = r.random_range(0..snowflake_chars.len());
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
    const fn is_space(&self) -> bool {
        self.val == ' '
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            val: ' ',
            white: false,
            color: Color::Red,
        }
    }
}

pub struct Column {
    length: usize,        // The length of the stream
    spaces: usize,        // The spaces between streams
    col: VecDeque<Block>, // The actual column
}

impl Column {
    fn new(lines: usize) -> Self {
        RNG.with(|rng| {
            let mut r = rng.borrow_mut();
            Self {
                length: r.random_range(1..lines),
                spaces: r.random_range(1..(lines * 2 + 1)),
                col: (0..lines).map(|_| Block::default()).collect(),
            }
        })
    }

    fn new_rand_head(&mut self, config: &Config) {
        self.col[0].val = rand_char();
        self.col[0].color = if config.rainbow {
            RNG.with(|rng| match rng.borrow_mut().random_range(0..6) {
                0 => Color::Green,
                1 => Color::Blue,
                2 => Color::White,
                3 => Color::Yellow,
                4 => Color::Cyan,
                5 => Color::Magenta,
                _ => unreachable!(),
            })
        } else {
            config.color
        };

        self.col[0].white = RNG.with(|rng| rng.borrow_mut().random_bool(0.25)); // ~64/256 chance
    }

    fn head_is_empty(&self) -> bool {
        self.col[1].val == ' '
    }

    fn new_rand_char(&mut self) {
        self.col[0].val = rand_char();
        self.col[0].color = self.col[1].color;
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
    fn default() -> Self {
        let (lines, cols) = get_term_size();
        Self {
            m: (0..cols).map(|_| Column::new(lines)).collect(),
        }
    }
}

impl Matrix {
    const fn num_columns(&self) -> usize {
        self.m.len()
    }

    fn num_lines(&self) -> usize {
        self[0].col.len()
    }

    pub fn arrange(&mut self, config: &Config) {
        let lines = self.num_lines();

        self.m.iter_mut().for_each(|col| {
            if col.head_is_empty() && col.spaces != 0 {
                col.spaces -= 1;
            } else if col.head_is_empty() && col.spaces == 0 {
                col.new_rand_head(config);
                col.length -= 1;
                col.spaces = RNG.with(|rng| rng.borrow_mut().random_range(1..=lines));
            } else if col.length != 0 {
                col.new_rand_char();
                col.length -= 1;
            } else {
                col.col[0].val = ' ';
                col.length = RNG.with(|rng| rng.borrow_mut().random_range(1..lines));
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
            let mut in_stream = false;
            let mut last_was_white = false;
            let mut running_color = Color::Cyan;

            col.col.iter_mut().for_each(|block| {
                if !in_stream {
                    if !block.is_space() {
                        block.val = ' ';
                        in_stream = true;
                        running_color = block.color;
                    }
                } else if block.is_space() {
                    block.val = rand_char();
                    block.white = last_was_white;
                    in_stream = false;
                }
                std::mem::swap(&mut last_was_white, &mut block.white);
                block.color = running_color;
            });
        });
    }

    fn old_style_move_down(&mut self) {
        self.m.iter_mut().for_each(|col| {
            col.col.pop_back();
            col.col.push_front(Block::default());
        });
    }

    /// Draws the matrix state to the provided writer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying writer fails to flush
    /// or if `crossterm` encounters an issue writing to the terminal buffer.
    pub fn draw(&self, w: &mut impl Write, config: &Config) -> io::Result<()> {
        for j in 1..self.num_lines() {
            for i in 0..self.num_columns() {
                let block = &self[i][j];
                let color = if block.white {
                    Color::White
                } else {
                    block.color
                };
                let x = u16::try_from(2 * i).unwrap_or(0);
                let y = u16::try_from(j).map_or(0, |val| val.saturating_sub(1));

                queue!(
                    w,
                    cursor::MoveTo(x, y),
                    style::SetForegroundColor(color),
                    style::Print(block.val)
                )?;
            }
        }
        w.flush()?;
        std::thread::sleep(std::time::Duration::from_millis(config.update as u64 * 10));
        Ok(())
    }

    pub fn resize(&mut self) {
        *self = Self::default();
    }
}

fn get_term_size() -> (usize, usize) {
    match term_size::dimensions() {
        Some((width, height)) => {
            let w = if width < 10 { 10 } else { width };
            let h = if height < 10 { 10 } else { height };
            if w % 2 != 0 {
                (h + 1, (w / 2) + 1)
            } else {
                (h + 1, w / 2)
            }
        }
        None => (10, 10),
    }
}

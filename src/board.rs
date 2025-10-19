use colored::Colorize; // Import ONLY the trait to enable coloring methods on strings
use std::fmt::{Debug, Display, Formatter};
use macroquad::prelude::*; // Import Macroquad drawing functions (Color is now unambiguously from Macroquad)

// CORRECTION: Explicitly import the Rng trait using absolute path to resolve ambiguity
use ::rand::Rng as _;

// --- RENDERING CONSTANTS (MACROQUAD) ---
// Dimensions and styles for the grid
pub const WINDOW_WIDTH: f32 = 600.0;
const PADDING: f32 = 10.0;
const UI_HEIGHT: f32 = 60.0; // Extra space for statistics
const GRID_SIZE: f32 = WINDOW_WIDTH - 2.0 * PADDING;
// Tile size calculation
const TILE_SIZE: f32 = (GRID_SIZE - (N as f32 + 1.0) * PADDING) / N as f32;
const FONT_SIZE: f32 = 40.0;
const BORDER_COLOR: Color = Color::new(0.53, 0.49, 0.45, 1.0); // #bbada0

// A board on which the next thing to do is to play (Agent's turn - MAX Node).
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct PlayableBoard(Board);

impl PlayableBoard {
    /// Returns an initial board, with a single random tile.
    pub fn init() -> PlayableBoard {
        let mut board = Board::EMPTY;
        board.add_random();
        PlayableBoard(board)
    }

    /// Applies an action and returns the next board state (RandableBoard), or None if the action is invalid.
    pub fn apply(&self, action: Action) -> Option<RandableBoard> {
        match self.0.apply(action) {
            Some(board) => Some(RandableBoard(board)),
            None => None,
        }
    }

    /// Checks if the board contains at least a tile with the given exponent (i).
    pub fn has_at_least_tile(&self, i: u8) -> bool {
        self.0.cells.iter().flatten().any(|tile| *tile >= i)
    }

    /// Draws the board onto the Macroquad window.
    pub fn draw(&self, num_moves: u32, decision_time_ms: f64) {
        clear_background(Color::new(0.98, 0.97, 0.94, 1.0)); // Window background (#faf8ef)

        // Draw the main grid background
        draw_rectangle(
            PADDING,
            PADDING + UI_HEIGHT,
            GRID_SIZE,
            GRID_SIZE,
            BORDER_COLOR,
        );

        // Draw statistics (Text)
        draw_text(
            &format!("Moves: {}", num_moves),
            PADDING,
            30.0,
            FONT_SIZE / 2.0,
            BLACK,
        );
        draw_text(
            &format!("Dec. Time: {:.2}ms", decision_time_ms),
            PADDING,
            55.0,
            FONT_SIZE / 2.0,
            BLACK,
        );

        // Draw cells and tiles
        for i in 0..N {
            for j in 0..N {
                let cell_value = self.0.cells[i][j];
                let (x, y) = self.get_tile_position(j, i);

                // Draw the empty cell background
                draw_rectangle(
                    x,
                    y,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(0.8, 0.75, 0.69, 1.0), // #cdc1b4
                );

                if cell_value != 0 {
                    let value = 2u32.pow(cell_value as u32);
                    let (bg_color, text_color) = self.get_tile_colors(value);

                    // 1. Draw the tile background
                    draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, bg_color);

                    // 2. Draw the tile value text
                    let text = value.to_string();
                    let font_size = if value > 1024 { FONT_SIZE * 0.7 } else { FONT_SIZE };

                    let text_dim = measure_text(&text, None, font_size as u16, 1.0);

                    // Center the text
                    let text_x = x + (TILE_SIZE - text_dim.width) / 2.0;
                    let text_y = y + (TILE_SIZE + text_dim.height) / 2.0;

                    draw_text(
                        &text,
                        text_x,
                        text_y,
                        font_size,
                        text_color,
                    );
                }
            }
        }
    }

    /// Helper function to calculate the screen position of a tile
    fn get_tile_position(&self, col: usize, row: usize) -> (f32, f32) {
        let x = PADDING + (col as f32 + 1.0) * PADDING + col as f32 * TILE_SIZE;
        let y = PADDING + UI_HEIGHT + (row as f32 + 1.0) * PADDING + row as f32 * TILE_SIZE;
        (x, y)
    }

    /// Helper function to get tile colors based on its value (exponent)
    fn get_tile_colors(&self, value: u32) -> (Color, Color) {
        let text_color = BLACK;
        let bg_color = match value {
            2 => Color::new(0.93, 0.90, 0.85, 1.0),   // #eee4da
            4 => Color::new(0.92, 0.88, 0.78, 1.0),   // #ede0c8
            8 => Color::new(0.95, 0.69, 0.47, 1.0),   // #f2b179
            16 => Color::new(0.96, 0.58, 0.39, 1.0),  // #f59563
            32 => Color::new(0.96, 0.49, 0.36, 1.0),  // #f67c5f
            64 => Color::new(0.96, 0.37, 0.23, 1.0),  // #f65e3b
            128 => Color::new(0.92, 0.81, 0.45, 1.0), // #edcf72
            256 => Color::new(0.92, 0.80, 0.38, 1.0), // #edcc61
            512 => Color::new(0.92, 0.78, 0.31, 1.0), // #edc850
            1024 => Color::new(0.92, 0.76, 0.25, 1.0),// #edc53f
            2048 => Color::new(0.92, 0.75, 0.18, 1.0),// #edc22e
            _ => Color::new(0.92, 0.75, 0.18, 1.0),   // 4096+
        };
        (bg_color, text_color)
    }
}

// Implement Display for PlayableBoard (needed for bench.rs console output)
impl Display for PlayableBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A board on which the next thing to do is to randomly place a tile (Chance turn - CHANCE Node).
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RandableBoard(Board);

impl RandableBoard {
    /// Adds a random tile (2 or 4) to the board, returning the next PlayableBoard state.
    pub fn with_random_tile(&self) -> PlayableBoard {
        let mut board = self.0;
        board.add_random();
        PlayableBoard(board)
    }

    /// Returns the list of possible successors after placing a random tile, along with their probabilities.
    /// This is crucial for the Expectimax algorithm.
    pub fn successors(&self) -> impl Iterator<Item = (f32, PlayableBoard)> + '_ {
        self.0
            .random_successors()
            .map(|(proba, board)| (proba, PlayableBoard(board)))
    }

    /// Evaluates the current board state using the heuristic function from `eval.rs`.
    pub fn evaluate(&self) -> f32 {
        crate::eval::eval(&self.0)
    }
}

// Implement Display for RandableBoard (needed for bench.rs console output)
impl Display for RandableBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Size of board
pub const N: usize = 4;

// A board is an NxN matrix where each entry represents a tile.
//
// A tile is encoded by an 8-bits unsigned int where:
//
//  - 0 represents the empty tile
//  - n > 0 represents the tile `2^n`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    pub cells: [[u8; N]; N],
}

impl Board {
    /// The completely empty board. Not the initial board.
    const EMPTY: Board = Board { cells: [[0; N]; N] };

    /// Returns the board resulting from the action, or None if the action is not applicable (no tiles moved).
    pub fn apply(&self, action: Action) -> Option<Board> {
        let mut next = self.clone();
        // We only implement push_left, so we use symmetries (transpose/swap_lr)
        // to map all actions to push_left and then revert the symmetries.
        match action {
            Action::Left => {
                next.push_left();
            }
            Action::Up => {
                next.transpose();
                next.push_left();
                next.transpose();
            }
            Action::Down => {
                next.transpose();
                next.swap_lr();
                next.push_left();
                next.swap_lr();
                next.transpose();
            }
            Action::Right => {
                next.swap_lr();
                next.push_left();
                next.swap_lr();
            }
        }
        if *self != next {
            // The board has changed, the action is applicable
            Some(next)
        } else {
            // Nothing changed, the action is not applicable
            None
        }
    }

    /// Places a random tile (2 or 4) on an empty cell of the board
    pub fn add_random(&mut self) {
        // compute the number of empty cells
        let n = self.num_empty();

        // decide which empty cell to update in [0,n)
        // Use absolute path ::rand::rng() to resolve Macroquad ambiguity
        let picked = ::rand::rng().random_range(0..n);

        // get a mutable reference of the cell
        let picked = self
            .cells
            .iter_mut()
            .map(|row| row.iter_mut())
            .flatten()
            .filter(|cell| **cell == 0)
            .nth(picked)
            .unwrap();

        // decide which value to put in the cell (2^1 = 2 with probability 0.9, 2^2 = 4 with probability 0.1)
        // Use absolute path ::rand::rng() to resolve Macroquad ambiguity
        let value = if ::rand::rng().random_bool(0.9) { 1 } else { 2 };

        // update the board by setting the value to the selected empty cell
        *picked = value;
    }

    /// Counts the number of empty tiles on the board
    pub fn num_empty(&self) -> usize {
        self.cells
            .iter()
            .flatten()
            .filter(|&&cell| cell == 0)
            .count()
    }

    /// Given a board for which an action has already been applied, returns the list of possible successors as a result of placing a random tile (2 or 4) on an empty cell.
    ///
    /// ```rust
    /// // Example of use:
    /// // let init = Board::init(); // Assuming init() exists or Board is created
    /// // let current = init.apply(Action::Left).expect("oups");
    /// // for (proba, succ_board) in current.random_successors() {
    /// //   println!("May get the following board with probability {proba}:\n{succ_board}");
    /// // }
    /// ```
    pub fn random_successors(&self) -> impl Iterator<Item = (f32, Board)> + '_ {
        let n = self.num_empty() as f32;

        let empty_cells = self.cells.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(j, &cell)| if cell == 0 { Some((i, j)) } else { None })
        });

        empty_cells.flat_map(move |(i, j)| {
            [(1, 0.9), (2, 0.1)] // (value_exponent, probability)
                .into_iter()
                .map(move |(new_value, proba)| {
                    let mut next = self.clone();
                    next.cells[i][j] = new_value;
                    // Probability is split evenly among all empty spots
                    (proba / n, next)
                })
        })
    }

    /// Switches the matrix left/right
    fn swap_lr(&mut self) {
        for row in &mut self.cells {
            let mut i = 0;
            let mut j = N - 1;
            while i < j {
                row.swap(i, j);
                i += 1;
                j -= 1;
            }
        }
    }

    /// Transposes the matrix, inverting lines and columns
    fn transpose(&mut self) {
        for i in 0..N {
            for j in 0..i {
                let tmp = self.cells[i][j];
                self.cells[i][j] = self.cells[j][i];
                self.cells[j][i] = tmp;
            }
        }
    }

    /// Builds an equivalent board where the lines and columns have been transposed
    pub fn transposed(&self) -> Board {
        let mut transposed = self.clone();
        transposed.transpose();
        transposed
    }

    /// Applies the action of playing *Left* on all rows
    fn push_left(&mut self) {
        // apply the push left method on each line
        for row in &mut self.cells {
            push_left(row);
        }
    }
}

// Implement Display for Board
impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", format!("╔═{}╗", "═".repeat(8 * N)).bold())?;
        for row in &self.cells {
            write!(f, "{}", "║ ".bold())?;
            for &cell in row {
                if cell != 0 {
                    let value = 2u32.pow(cell as u32);
                    let formatted = format!("{:^7}", value);
                    let colored = match value {
                        2 => formatted.black().on_truecolor(238, 228, 218), // #eee4da
                        4 => formatted.black().on_truecolor(237, 224, 200), // #ede0c8
                        8 => formatted.black().on_truecolor(242, 177, 121), // #f2b179
                        16 => formatted.black().on_truecolor(245, 149, 99), // #f59563
                        32 => formatted.black().on_truecolor(246, 124, 95), // #f67c5f
                        64 => formatted.black().on_truecolor(246, 94, 59),  // #f65e3b
                        128 => formatted.black().on_truecolor(237, 207, 114), // #edcf72
                        256 => formatted.black().on_truecolor(237, 204, 97), // #edcc61
                        512 => formatted.black().on_truecolor(237, 200, 80), // #edc850
                        1024 => formatted.black().on_truecolor(237, 197, 63), // #edc53f
                        2048 => formatted.black().on_truecolor(237, 194, 46), // 2048 -> #edc22e
                        _ => formatted.bold().black().on_truecolor(237, 194, 46), // 4096+ -> #edc22e + bold
                    };
                    write!(f, "{} ", colored)?;
                } else {
                    let formatted = format!("   .   ");
                    let colored = formatted.black().on_truecolor(205, 193, 180); // #cdc1b4
                    write!(f, "{} ", colored)?;
                }
            }
            writeln!(f, "{} ", "║".bold())?;
        }
        writeln!(f, "{}", format!("╚═{}╝", "═".repeat(8 * N)).bold())?;
        Ok(())
    }
}

/// The set of possible actions to apply on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

/// An iterable list of all possible actions.
pub const ALL_ACTIONS: [Action; 4] = [Action::Up, Action::Down, Action::Left, Action::Right];

/// Applies the core logic of pushing tiles "left" on a single Row
fn push_left(row: &mut [u8; N]) {
    let mut write_index = 0; // Position to write next non-zero tile
    let mut read_index = 0; // Reading index

    // Move non-zero tiles forward and merge adjacent ones
    while read_index < N {
        if row[read_index] == 0 {
            read_index += 1;
            continue;
        }

        let value = row[read_index];
        read_index += 1;

        // Merge with the next non-zero value if it matches
        if read_index < N {
            while read_index < N && row[read_index] == 0 {
                read_index += 1; // Skip empty cell
            }
            if read_index < N && row[read_index] == value {
                row[write_index] = value + 1;
                read_index += 1; // Skip merged cell
            } else {
                row[write_index] = value;
            }
        } else {
            row[write_index] = value;
        }

        write_index += 1;
    }

    // Fill the remaining cells with zero (empty)
    row[write_index..].fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_left() {
        fn check(row: [u8; N], expected: [u8; N]) {
            let mut pushed = row;
            push_left(&mut pushed);
            assert_eq!(pushed, expected);
        }
        check([0, 0, 0, 0], [0, 0, 0, 0]);
        check([0, 1, 0, 0], [1, 0, 0, 0]);
        check([0, 0, 1, 0], [1, 0, 0, 0]);
        check([0, 0, 0, 1], [1, 0, 0, 0]);
        check([0, 0, 0, 0], [0, 0, 0, 0]);
        check([1, 1, 0, 1], [2, 1, 0, 0]);
        check([0, 0, 1, 1], [2, 0, 0, 0]);
        check([0, 1, 0, 1], [2, 0, 0, 0]);
        check([1, 2, 0, 1], [1, 2, 1, 0]);
    }

    #[test]
    fn test_actions() {
        let board = Board {
            cells: [[1, 2, 1, 0], [4, 1, 0, 0], [3, 0, 0, 0], [0, 0, 0, 0]],
        };
        let target = Board {
            cells: [[0, 0, 0, 0], [1, 0, 0, 0], [4, 2, 0, 0], [3, 1, 1, 0]],
        };
        // The test checks the Down action (which requires transpose, swap_lr, push_left, swap_lr, transpose)
        assert_eq!(board.apply(Action::Down), Some(target));
    }
}

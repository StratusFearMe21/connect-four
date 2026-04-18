// core/board.rs - Board implementation with extensive functionality
use crate::core::cell::Cell;
use crate::core::coordinates::{Direction, Position};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::ops::{Index, IndexMut};
use std::str::FromStr;

/// Represents a game board with configurable dimensions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Board {
    rows: usize,
    cols: usize,
    cells: Vec<Vec<Cell>>,
}

impl Board {
    /// Creates a new empty board with the given dimensions
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![vec![Cell::Empty; cols]; rows];
        Board { rows, cols, cells }
    }

    /// Creates a standard Connect Four board (6 rows, 7 columns)
    pub fn standard() -> Self {
        Self::new(6, 7)
    }

    /// Creates a board from a 2D slice of cells
    pub fn from_cells(cells: &[Vec<Cell>]) -> Result<Self, String> {
        if cells.is_empty() {
            return Err("Board cannot be empty".to_string());
        }

        let rows = cells.len();
        let cols = cells[0].len();

        for row in cells {
            if row.len() != cols {
                return Err("All rows must have the same length".to_string());
            }
        }

        Ok(Board {
            rows,
            cols,
            cells: cells.to_vec(),
        })
    }

    /// Returns the number of rows in the board
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the number of columns in the board
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Returns the dimensions as a tuple (rows, cols)
    pub fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Returns the total number of cells in the board
    pub fn size(&self) -> usize {
        self.rows * self.cols
    }

    /// Checks if a position is valid on this board
    pub fn is_valid_position(&self, pos: &Position) -> bool {
        pos.row < self.rows && pos.col < self.cols
    }

    /// Gets the cell at the given position
    pub fn get(&self, pos: &Position) -> Option<Cell> {
        if self.is_valid_position(pos) {
            Some(self.cells[pos.row][pos.col])
        } else {
            None
        }
    }

    /// Gets the cell at the given row and column
    pub fn get_at(&self, row: usize, col: usize) -> Option<Cell> {
        if row < self.rows && col < self.cols {
            Some(self.cells[row][col])
        } else {
            None
        }
    }

    /// Sets the cell at the given position
    pub fn set(&mut self, pos: &Position, cell: Cell) -> Result<(), String> {
        if self.is_valid_position(pos) {
            self.cells[pos.row][pos.col] = cell;
            Ok(())
        } else {
            Err(format!("Invalid position: {:?}", pos))
        }
    }

    /// Sets the cell at the given row and column
    pub fn set_at(&mut self, row: usize, col: usize, cell: Cell) -> Result<(), String> {
        if row < self.rows && col < self.cols {
            self.cells[row][col] = cell;
            Ok(())
        } else {
            Err(format!("Invalid position: ({}, {})", row, col))
        }
    }

    /// Clears the entire board, setting all cells to Empty
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::Empty;
            }
        }
    }

    /// Clears a specific cell
    pub fn clear_cell(&mut self, pos: &Position) -> Result<(), String> {
        self.set(pos, Cell::Empty)
    }

    /// Clears a specific cell by coordinates
    pub fn clear_cell_at(&mut self, row: usize, col: usize) -> Result<(), String> {
        self.set_at(row, col, Cell::Empty)
    }

    /// Checks if a cell at the given position is empty
    pub fn is_empty(&self, pos: &Position) -> bool {
        self.get(pos).map_or(false, |c| c == Cell::Empty)
    }

    /// Checks if a cell at the given coordinates is empty
    pub fn is_empty_at(&self, row: usize, col: usize) -> bool {
        self.get_at(row, col).map_or(false, |c| c == Cell::Empty)
    }

    /// Checks if a column is full (top cell is not empty)
    pub fn is_column_full(&self, col: usize) -> bool {
        if col >= self.cols {
            return true;
        }
        self.cells[0][col] != Cell::Empty
    }

    /// Checks if any column is valid for a move
    pub fn has_valid_column(&self) -> bool {
        (0..self.cols).any(|col| !self.is_column_full(col))
    }

    /// Returns the number of empty cells in a column
    pub fn column_empty_count(&self, col: usize) -> usize {
        if col >= self.cols {
            return 0;
        }
        self.cells.iter().filter(|row| row[col] == Cell::Empty).count()
    }

    /// Returns the number of pieces in a column
    pub fn column_piece_count(&self, col: usize) -> usize {
        if col >= self.cols {
            return 0;
        }
        self.cells.iter().filter(|row| row[col] != Cell::Empty).count()
    }

    /// Returns the lowest empty row in a column (where a piece would land)
    pub fn lowest_empty_row(&self, col: usize) -> Option<usize> {
        if col >= self.cols {
            return None;
        }

        for row in (0..self.rows).rev() {
            if self.cells[row][col] == Cell::Empty {
                return Some(row);
            }
        }

        None
    }

    /// Returns the highest occupied row in a column
    pub fn highest_occupied_row(&self, col: usize) -> Option<usize> {
        if col >= self.cols {
            return None;
        }

        for row in 0..self.rows {
            if self.cells[row][col] != Cell::Empty {
                return Some(row);
            }
        }

        None
    }

    /// Counts how many cells of a given type are on the board
    pub fn count_cells(&self, cell: Cell) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c == cell)
            .count()
    }

    /// Counts empty cells on the board
    pub fn empty_count(&self) -> usize {
        self.count_cells(Cell::Empty)
    }

    /// Counts occupied cells on the board
    pub fn occupied_count(&self) -> usize {
        self.size() - self.empty_count()
    }

    /// Checks if the board is completely empty
    pub fn is_empty_board(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|&c| c == Cell::Empty))
    }

    /// Checks if the board is completely full
    pub fn is_full(&self) -> bool {
        self.empty_count() == 0
    }

    /// Gets a row as a slice
    pub fn row(&self, row: usize) -> Option<&[Cell]> {
        if row < self.rows {
            Some(&self.cells[row])
        } else {
            None
        }
    }

    /// Gets a column as a vector
    pub fn column(&self, col: usize) -> Option<Vec<Cell>> {
        if col >= self.cols {
            return None;
        }

        Some(self.cells.iter().map(|row| row[col]).collect())
    }

    /// Returns an iterator over all positions on the board
    pub fn positions(&self) -> BoardPositionIterator {
        BoardPositionIterator::new(self.rows, self.cols)
    }

    /// Returns an iterator over all cells on the board
    pub fn cells_iter(&self) -> impl Iterator<Item = (Position, Cell)> + '_ {
        self.positions().map(move |pos| {
            let cell = self.get(&pos).unwrap_or(Cell::Empty);
            (pos, cell)
        })
    }

    /// Returns an iterator over all empty cells
    pub fn empty_cells(&self) -> impl Iterator<Item = Position> + '_ {
        self.positions().filter(|pos| self.is_empty(pos))
    }

    /// Returns an iterator over all occupied cells
    pub fn occupied_cells(&self) -> impl Iterator<Item = (Position, Cell)> + '_ {
        self.cells_iter().filter(|(_, cell)| *cell != Cell::Empty)
    }

    /// Returns all cells of a specific type
    pub fn cells_of_type(&self, cell: Cell) -> Vec<Position> {
        self.cells_iter()
            .filter(|&(_, c)| c == cell)
            .map(|(pos, _)| pos)
            .collect()
    }

    /// Gets all cells in a line in the given direction from a starting position
    pub fn get_line(&self, start: &Position, direction: Direction) -> Vec<Position> {
        let mut positions = Vec::new();
        let mut current = *start;

        while let Some(next) = current + direction {
            if self.is_valid_position(&next) {
                positions.push(next);
                current = next;
            } else {
                break;
            }
        }

        positions
    }

    /// Gets cells in both directions along a line (includes start in result)
    pub fn get_line_bidirectional(
        &self,
        start: &Position,
        direction: Direction,
    ) -> Vec<(Position, Cell)> {
        let mut result = vec![(*start, self.get(start).unwrap_or(Cell::Empty))];

        // Forward direction
        let mut current = *start;
        while let Some(next) = current + direction {
            if self.is_valid_position(&next) {
                result.push((next, self.get(&next).unwrap_or(Cell::Empty)));
                current = next;
            } else {
                break;
            }
        }

        // Backward direction
        current = *start;
        while let Some(next) = current + direction.opposite() {
            if self.is_valid_position(&next) {
                result.push((next, self.get(&next).unwrap_or(Cell::Empty)));
                current = next;
            } else {
                break;
            }
        }

        result
    }

    /// Gets cells in a range of positions
    pub fn get_range(&self, positions: &[Position]) -> Vec<Cell> {
        positions
            .iter()
            .map(|pos| self.get(pos).unwrap_or(Cell::Empty))
            .collect()
    }

    /// Applies a function to all cells
    pub fn map_cells<F>(&mut self, mut f: F)
    where
        F: FnMut(Position, Cell) -> Cell,
    {
        for pos in self.positions() {
            let current = self.get(&pos).unwrap_or(Cell::Empty);
            let new_cell = f(pos, current);
            let _ = self.set(&pos, new_cell);
        }
    }

    /// Filters cells based on a predicate
    pub fn filter_cells<F>(&self, f: F) -> Vec<(Position, Cell)>
    where
        F: FnMut(&(Position, Cell)) -> bool,
    {
        self.cells_iter().filter(f).collect()
    }

    /// Counts consecutive cells in a direction from a starting position
    pub fn count_consecutive(
        &self,
        start: &Position,
        direction: Direction,
        cell: Cell,
    ) -> usize {
        let mut count = 0;
        let mut current = *start;

        loop {
            let Some(next) = current + direction else {
                break;
            };

            if !self.is_valid_position(&next) {
                break;
            }

            if self.get(&next).map_or(false, |c| c.matches_player(cell)) {
                count += 1;
                current = next;
            } else {
                break;
            }
        }

        count
    }

    /// Counts consecutive cells in both directions from a starting position
    pub fn count_consecutive_bidirectional(
        &self,
        start: &Position,
        direction: Direction,
        cell: Cell,
    ) -> usize {
        let forward = self.count_consecutive(start, direction, cell);
        let backward = self.count_consecutive(start, direction.opposite(), cell);
        forward + backward
    }

    /// Checks if there's a line of connected cells starting from a position
    pub fn has_line(
        &self,
        start: &Position,
        direction: Direction,
        cell: Cell,
        length: usize,
    ) -> bool {
        self.count_consecutive_bidirectional(start, direction, cell) >= length
    }

    /// Rotates the board 90 degrees clockwise
    pub fn rotate_clockwise(&mut self) {
        let new_cells: Vec<Vec<Cell>> = (0..self.cols)
            .map(|col| (0..self.rows).rev().map(|row| self.cells[row][col]).collect())
            .collect();
        std::mem::swap(&mut self.cells, &mut new_cells);
        std::mem::swap(&mut self.rows, &mut self.cols);
    }

    /// Rotates the board 90 degrees counter-clockwise
    pub fn rotate_counter_clockwise(&mut self) {
        let new_cells: Vec<Vec<Cell>> = (0..self.cols)
            .rev()
            .map(|col| (0..self.rows).map(|row| self.cells[row][col]).collect())
            .collect();
        std::mem::swap(&mut self.cells, &mut new_cells);
        std::mem::swap(&mut self.rows, &mut self.cols);
    }

    /// Flips the board horizontally
    pub fn flip_horizontal(&mut self) {
        for row in &mut self.cells {
            row.reverse();
        }
    }

    /// Flips the board vertically
    pub fn flip_vertical(&mut self) {
        self.cells.reverse();
    }

    /// Transposes the board (swaps rows and columns)
    pub fn transpose(&mut self) {
        let new_cells: Vec<Vec<Cell>> = (0..self.cols)
            .map(|col| (0..self.rows).map(|row| self.cells[row][col]).collect())
            .collect();
        std::mem::swap(&mut self.cells, &mut new_cells);
        std::mem::swap(&mut self.rows, &mut self.cols);
    }

    /// Creates a sub-board from the given region
    pub fn sub_board(&self, row_start: usize, col_start: usize, rows: usize, cols: usize) -> Option<Self> {
        if row_start + rows > self.rows || col_start + cols > self.cols {
            return None;
        }

        let cells: Vec<Vec<Cell>> = (row_start..row_start + rows)
            .map(|row| (col_start..col_start + cols).map(|col| self.cells[row][col]).collect())
            .collect();

        Some(Board {
            rows,
            cols,
            cells,
        })
    }

    /// Checks if this board matches another board
    pub fn matches(&self, other: &Board) -> bool {
        self.rows == other.rows
            && self.cols == other.cols
            && self.cells == other.cells
    }

    /// Calculates the hash of the board state
    pub fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.rows.hash(&mut hasher);
        self.cols.hash(&mut hasher);
        for row in &self.cells {
            row.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Creates a string representation of the board
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        // Header
        result.push_str("  ");
        for col in 0..self.cols {
            result.push_str(&format!("{} ", col % 10));
        }
        result.push('\n');

        // Board
        for row in 0..self.rows {
            result.push_str(&format!("{} ", row % 10));
            for col in 0..self.cols {
                result.push(self.cells[row][col].symbol());
                result.push(' ');
            }
            result.push('\n');
        }

        result
    }

    /// Creates an ASCII string representation of the board
    pub fn to_ascii_string(&self) -> String {
        let mut result = String::new();

        result.push_str("  ");
        for col in 0..self.cols {
            result.push_str(&format!("{} ", col % 10));
        }
        result.push('\n');

        for row in 0..self.rows {
            result.push_str(&format!("{} ", row % 10));
            for col in 0..self.cols {
                result.push(self.cells[row][col].ascii_symbol());
                result.push(' ');
            }
            result.push('\n');
        }

        result
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::standard()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for Board {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.lines().collect();

        if lines.is_empty() {
            return Err("Empty board string".to_string());
        }

        let mut cells: Vec<Vec<Cell>> = Vec::new();

        for line in &lines {
            let row: Vec<Cell> = line
                .chars()
                .map(|c| Cell::from_str(&c.to_string()))
                .collect::<Result<Vec<_>, _>>()?;

            if cells.is_empty() {
                cells.push(row);
            } else if row.len() == cells[0].len() {
                cells.push(row);
            } else {
                return Err("Inconsistent row lengths".to_string());
            }
        }

        if cells.is_empty() {
            return Err("No valid rows found".to_string());
        }

        let rows = cells.len();
        let cols = cells[0].len();

        Ok(Board { rows, cols, cells })
    }
}

impl Index<Position> for Board {
    type Output = Cell;

    fn index(&self, pos: Position) -> &Self::Output {
        &self.cells[pos.row][pos.col]
    }
}

impl IndexMut<Position> for Board {
    fn index_mut(&mut self, pos: Position) -> &mut Self::Output {
        &mut self.cells[pos.row][pos.col]
    }
}

impl Index<(usize, usize)> for Board {
    type Output = Cell;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.cells[row][col]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.cells[row][col]
    }
}

/// Iterator for board positions
#[derive(Clone, Debug)]
pub struct BoardPositionIterator {
    rows: usize,
    cols: usize,
    current_row: usize,
    current_col: usize,
}

impl BoardPositionIterator {
    pub fn new(rows: usize, cols: usize) -> Self {
        BoardPositionIterator {
            rows,
            cols,
            current_row: 0,
            current_col: 0,
        }
    }
}

impl Iterator for BoardPositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.rows {
            return None;
        }

        let pos = Position::new(self.current_row, self.current_col);

        self.current_col += 1;
        if self.current_col >= self.cols {
            self.current_col = 0;
            self.current_row += 1;
        }

        Some(pos)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.rows - self.current_row) * self.cols - self.current_col;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for BoardPositionIterator {
    fn len(&self) -> usize {
        (self.rows - self.current_row) * self.cols - self.current_col
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new() {
        let board = Board::new(6, 7);
        assert_eq!(board.rows(), 6);
        assert_eq!(board.cols(), 7);
        assert_eq!(board.size(), 42);
    }

    #[test]
    fn test_board_standard() {
        let board = Board::standard();
        assert_eq!(board.rows(), 6);
        assert_eq!(board.cols(), 7);
    }

    #[test]
    fn test_board_from_cells() {
        let cells = vec![vec![Cell::Empty, Cell::Player1]];
        let board = Board::from_cells(&cells).unwrap();
        assert_eq!(board.rows(), 1);
        assert_eq!(board.cols(), 2);
    }

    #[test]
    fn test_board_set_get() {
        let mut board = Board::new(3, 3);
        let pos = Position::new(1, 1);

        board.set(&pos, Cell::Player1).unwrap();
        assert_eq!(board.get(&pos), Some(Cell::Player1));
    }

    #[test]
    fn test_board_clear() {
        let mut board = Board::new(3, 3);
        board.set_at(1, 1, Cell::Player1).unwrap();
        board.clear();
        assert!(board.is_empty_board());
    }

    #[test]
    fn test_board_is_column_full() {
        let mut board = Board::new(3, 3);
        assert!(!board.is_column_full(0));

        for row in 0..3 {
            board.set_at(row, 0, Cell::Player1).unwrap();
        }
        assert!(board.is_column_full(0));
    }

    #[test]
    fn test_board_lowest_empty_row() {
        let mut board = Board::new(6, 7);

        board.set_at(5, 3, Cell::Player1).unwrap();
        assert_eq!(board.lowest_empty_row(3), Some(4));

        board.set_at(4, 3, Cell::Player2).unwrap();
        assert_eq!(board.lowest_empty_row(3), Some(3));
    }

    #[test]
    fn test_board_count_cells() {
        let mut board = Board::new(3, 3);
        board.set_at(0, 0, Cell::Player1).unwrap();
        board.set_at(0, 1, Cell::Player2).unwrap();

        assert_eq!(board.count_cells(Cell::Player1), 1);
        assert_eq!(board.count_cells(Cell::Player2), 1);
        assert_eq!(board.empty_count(), 7);
    }

    #[test]
    fn test_board_is_full() {
        let mut board = Board::new(2, 2);
        assert!(!board.is_full());

        for row in 0..2 {
            for col in 0..2 {
                board.set_at(row, col, Cell::Player1).unwrap();
            }
        }
        assert!(board.is_full());
    }

    #[test]
    fn test_board_column() {
        let mut board = Board::new(3, 2);
        board.set_at(0, 0, Cell::Player1).unwrap();
        board.set_at(1, 0, Cell::Player2).unwrap();

        let col = board.column(0).unwrap();
        assert_eq!(col, vec![Cell::Player1, Cell::Player2, Cell::Empty]);
    }

    #[test]
    fn test_board_rotate() {
        let mut board = Board::new(2, 3);
        board.set_at(0, 0, Cell::Player1).unwrap();
        board.set_at(0, 2, Cell::Player2).unwrap();

        board.rotate_clockwise();
        assert_eq!(board.rows(), 3);
        assert_eq!(board.cols(), 2);
    }

    #[test]
    fn test_board_flip() {
        let mut board = Board::new(2, 3);
        board.set_at(0, 0, Cell::Player1).unwrap();
        board.set_at(0, 2, Cell::Player2).unwrap();

        board.flip_horizontal();
        assert_eq!(board.get_at(0, 0), Some(Cell::Player2));
        assert_eq!(board.get_at(0, 2), Some(Cell::Player1));
    }

    #[test]
    fn test_board_hash() {
        let board1 = Board::standard();
        let board2 = Board::standard();
        assert_eq!(board1.hash(), board2.hash());
    }

    #[test]
    fn test_board_index() {
        let mut board = Board::new(3, 3);
        let pos = Position::new(1, 1);
        board[pos] = Cell::Player1;
        assert_eq!(board[pos], Cell::Player1);
    }

    #[test]
    fn test_board_iterator() {
        let board = Board::new(2, 2);
        let positions: Vec<Position> = board.positions().collect();
        assert_eq!(positions.len(), 4);
    }
}

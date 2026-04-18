// core/coordinates.rs - Coordinate system for the game board
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Represents a position on the game board with row and column
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    /// Creates a new position at the given row and column
    pub const fn new(row: usize, col: usize) -> Self {
        Position { row, col }
    }

    /// Creates a position at row 0, column 0 (top-left)
    pub const fn origin() -> Self {
        Position::new(0, 0)
    }

    /// Returns true if this position is valid for the given board dimensions
    pub fn is_valid(&self, rows: usize, cols: usize) -> bool {
        self.row < rows && self.col < cols
    }

    /// Returns the position above this one (row - 1)
    pub fn above(&self) -> Option<Self> {
        if self.row > 0 {
            Some(Position::new(self.row - 1, self.col))
        } else {
            None
        }
    }

    /// Returns the position below this one (row + 1)
    pub fn below(&self) -> Position {
        Position::new(self.row + 1, self.col)
    }

    /// Returns the position to the left of this one (col - 1)
    pub fn left(&self) -> Option<Self> {
        if self.col > 0 {
            Some(Position::new(self.row, self.col - 1))
        } else {
            None
        }
    }

    /// Returns the position to the right of this one (col + 1)
    pub fn right(&self) -> Position {
        Position::new(self.row, self.col + 1)
    }

    /// Returns the position diagonally up-left
    pub fn up_left(&self) -> Option<Self> {
        self.above()?.left()
    }

    /// Returns the position diagonally up-right
    pub fn up_right(&self) -> Option<Self> {
        self.above()?.right().into()
    }

    /// Returns the position diagonally down-left
    pub fn down_left(&self) -> Option<Self> {
        Some(Position::new(self.row + 1, self.col))
            .filter(|p| p.col > 0)
            .map(|p| Position::new(p.row, p.col - 1))
    }

    /// Returns the position diagonally down-right
    pub fn down_right(&self) -> Position {
        Position::new(self.row + 1, self.col + 1)
    }

    /// Moves this position in the given direction
    pub fn move_direction(&self, direction: Direction) -> Option<Self> {
        direction.apply(self)
    }

    /// Returns all 8 neighbors of this position
    pub fn neighbors(&self) -> Vec<Position> {
        let mut neighbors = Vec::with_capacity(8);

        if let Some(pos) = self.above() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.below().into() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.left() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.right().into() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.up_left() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.up_right() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.down_left() {
            neighbors.push(pos);
        }
        neighbors.push(self.down_right());

        neighbors
    }

    /// Returns all 4 orthogonal neighbors (up, down, left, right)
    pub fn orthogonal_neighbors(&self) -> Vec<Position> {
        let mut neighbors = Vec::with_capacity(4);

        if let Some(pos) = self.above() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.below().into() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.left() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.right().into() {
            neighbors.push(pos);
        }

        neighbors
    }

    /// Returns all 4 diagonal neighbors
    pub fn diagonal_neighbors(&self) -> Vec<Position> {
        let mut neighbors = Vec::with_capacity(4);

        if let Some(pos) = self.up_left() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.up_right() {
            neighbors.push(pos);
        }
        if let Some(pos) = self.down_left() {
            neighbors.push(pos);
        }
        neighbors.push(self.down_right());

        neighbors
    }

    /// Calculates the Manhattan distance to another position
    pub fn manhattan_distance(&self, other: &Self) -> usize {
        self.row.abs_diff(other.row) + self.col.abs_diff(other.col)
    }

    /// Calculates the Euclidean distance to another position
    pub fn euclidean_distance(&self, other: &Self) -> f64 {
        let dr = self.row as f64 - other.row as f64;
        let dc = self.col as f64 - other.col as f64;
        (dr * dr + dc * dc).sqrt()
    }

    /// Returns the minimum row and column as a tuple
    pub const fn as_tuple(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    /// Returns the 1D index of this position in a flattened array
    pub fn flatten(&self, cols: usize) -> usize {
        self.row * cols + self.col
    }

    /// Creates a position from a 1D index in a flattened array
    pub fn from_flat(index: usize, cols: usize) -> Self {
        Position::new(index / cols, index % cols)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl Add<Direction> for Position {
    type Output = Option<Position>;

    fn add(self, direction: Direction) -> Self::Output {
        direction.apply(&self)
    }
}

impl Add<Offset> for Position {
    type Output = Position;

    fn add(self, offset: Offset) -> Self::Output {
        Position::new(
            self.row.wrapping_add(offset.row_offset),
            self.col.wrapping_add(offset.col_offset),
        )
    }
}

impl AddAssign<Offset> for Position {
    fn add_assign(&mut self, offset: Offset) {
        self.row = self.row.wrapping_add(offset.row_offset);
        self.col = self.col.wrapping_add(offset.col_offset);
    }
}

impl Sub for Position {
    type Output = Offset;

    fn sub(self, other: Self) -> Self::Output {
        Offset::new(
            self.row.wrapping_sub(other.row),
            self.col.wrapping_sub(other.col),
        )
    }
}

/// Represents a 2D offset that can be applied to a position
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Offset {
    pub row_offset: isize,
    pub col_offset: isize,
}

impl Offset {
    pub const fn new(row_offset: isize, col_offset: isize) -> Self {
        Offset {
            row_offset,
            col_offset,
        }
    }

    pub const fn zero() -> Self {
        Offset::new(0, 0)
    }

    pub const fn up() -> Self {
        Offset::new(-1, 0)
    }

    pub const fn down() -> Self {
        Offset::new(1, 0)
    }

    pub const fn left() -> Self {
        Offset::new(0, -1)
    }

    pub const fn right() -> Self {
        Offset::new(0, 1)
    }

    pub const fn up_left() -> Self {
        Offset::new(-1, -1)
    }

    pub const fn up_right() -> Self {
        Offset::new(-1, 1)
    }

    pub const fn down_left() -> Self {
        Offset::new(1, -1)
    }

    pub const fn down_right() -> Self {
        Offset::new(1, 1)
    }

    pub fn magnitude(&self) -> f64 {
        let r = self.row_offset as f64;
        let c = self.col_offset as f64;
        (r * r + c * c).sqrt()
    }

    pub fn manhattan(&self) -> usize {
        self.row_offset.unsigned_abs() + self.col_offset.unsigned_abs()
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row_offset, self.col_offset)
    }
}

/// Represents one of the 8 cardinal and diagonal directions
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

impl Direction {
    /// Returns all 8 directions
    pub fn all() -> [Direction; 8] {
        [
            Direction::North,
            Direction::Northeast,
            Direction::East,
            Direction::Southeast,
            Direction::South,
            Direction::Southwest,
            Direction::West,
            Direction::Northwest,
        ]
    }

    /// Returns the 4 cardinal directions
    pub fn cardinal() -> [Direction; 4] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }

    /// Returns the 4 diagonal directions
    pub fn diagonal() -> [Direction; 4] {
        [
            Direction::Northeast,
            Direction::Southeast,
            Direction::Southwest,
            Direction::Northwest,
        ]
    }

    /// Returns the offset for this direction
    pub fn offset(&self) -> Offset {
        match self {
            Direction::North => Offset::up(),
            Direction::Northeast => Offset::up_right(),
            Direction::East => Offset::right(),
            Direction::Southeast => Offset::down_right(),
            Direction::South => Offset::down(),
            Direction::Southwest => Offset::down_left(),
            Direction::West => Offset::left(),
            Direction::Northwest => Offset::up_left(),
        }
    }

    /// Applies this direction to a position
    pub fn apply(&self, pos: &Position) -> Option<Position> {
        let offset = self.offset();
        let new_row = pos.row as isize + offset.row_offset;
        let new_col = pos.col as isize + offset.col_offset;

        if new_row >= 0 && new_col >= 0 {
            Some(Position::new(new_row as usize, new_col as usize))
        } else {
            None
        }
    }

    /// Returns the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Northeast => Direction::Southwest,
            Direction::Southwest => Direction::Northeast,
            Direction::Northwest => Direction::Southeast,
            Direction::Southeast => Direction::Northwest,
        }
    }

    /// Returns true if this is a cardinal direction
    pub fn is_cardinal(&self) -> bool {
        matches!(
            self,
            Direction::North | Direction::South | Direction::East | Direction::West
        )
    }

    /// Returns true if this is a diagonal direction
    pub fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::Northeast
                | Direction::Southeast
                | Direction::Southwest
                | Direction::Northwest
        )
    }

    /// Returns the direction as an angle in degrees (0 = North, clockwise)
    pub fn as_angle(&self) -> f64 {
        match self {
            Direction::North => 0.0,
            Direction::Northeast => 45.0,
            Direction::East => 90.0,
            Direction::Southeast => 135.0,
            Direction::South => 180.0,
            Direction::Southwest => 225.0,
            Direction::West => 270.0,
            Direction::Northwest => 315.0,
        }
    }

    /// Creates a direction from an angle in degrees
    pub fn from_angle(angle: f64) -> Option<Direction> {
        let angle = angle % 360.0;
        match angle as i32 {
            0 => Some(Direction::North),
            45 => Some(Direction::Northeast),
            90 => Some(Direction::East),
            135 => Some(Direction::Southeast),
            180 => Some(Direction::South),
            225 => Some(Direction::Southwest),
            270 => Some(Direction::West),
            315 => Some(Direction::Northwest),
            _ => None,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::Northeast => write!(f, "Northeast"),
            Direction::East => write!(f, "East"),
            Direction::Southeast => write!(f, "Southeast"),
            Direction::South => write!(f, "South"),
            Direction::Southwest => write!(f, "Southwest"),
            Direction::West => write!(f, "West"),
            Direction::Northwest => write!(f, "Northwest"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.row, 3);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_position_origin() {
        let pos = Position::origin();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_position_is_valid() {
        let pos = Position::new(3, 5);
        assert!(pos.is_valid(6, 7));
        assert!(!pos.is_valid(3, 7));
        assert!(!pos.is_valid(6, 5));
    }

    #[test]
    fn test_position_above() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.above(), Some(Position::new(2, 5)));
        assert_eq!(Position::new(0, 5).above(), None);
    }

    #[test]
    fn test_position_below() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.below(), Position::new(4, 5));
    }

    #[test]
    fn test_position_left() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.left(), Some(Position::new(3, 4)));
        assert_eq!(Position::new(3, 0).left(), None);
    }

    #[test]
    fn test_position_right() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.right(), Position::new(3, 6));
    }

    #[test]
    fn test_position_up_left() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.up_left(), Some(Position::new(2, 4)));
        assert_eq!(Position::new(0, 5).up_left(), None);
    }

    #[test]
    fn test_position_down_right() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.down_right(), Position::new(4, 6));
    }

    #[test]
    fn test_position_manhattan_distance() {
        let pos1 = Position::new(1, 1);
        let pos2 = Position::new(4, 5);
        assert_eq!(pos1.manhattan_distance(&pos2), 7);
    }

    #[test]
    fn test_position_euclidean_distance() {
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 4);
        let dist = pos1.euclidean_distance(&pos2);
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_position_flatten() {
        let pos = Position::new(2, 3);
        assert_eq!(pos.flatten(7), 17); // 2 * 7 + 3
    }

    #[test]
    fn test_position_from_flat() {
        let pos = Position::from_flat(17, 7);
        assert_eq!(pos, Position::new(2, 3));
    }

    #[test]
    fn test_offset_new() {
        let offset = Offset::new(-1, 2);
        assert_eq!(offset.row_offset, -1);
        assert_eq!(offset.col_offset, 2);
    }

    #[test]
    fn test_offset_directions() {
        assert_eq!(Offset::up(), Offset::new(-1, 0));
        assert_eq!(Offset::down(), Offset::new(1, 0));
        assert_eq!(Offset::left(), Offset::new(0, -1));
        assert_eq!(Offset::right(), Offset::new(0, 1));
    }

    #[test]
    fn test_offset_magnitude() {
        let offset = Offset::new(3, 4);
        assert!((offset.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_offset_manhattan() {
        let offset = Offset::new(3, 4);
        assert_eq!(offset.manhattan(), 7);
    }

    #[test]
    fn test_direction_all() {
        let directions = Direction::all();
        assert_eq!(directions.len(), 8);
    }

    #[test]
    fn test_direction_cardinal() {
        let cardinal = Direction::cardinal();
        assert_eq!(cardinal.len(), 4);
        assert!(cardinal.contains(&Direction::North));
        assert!(!cardinal.contains(&Direction::Northeast));
    }

    #[test]
    fn test_direction_offset() {
        assert_eq!(Direction::North.offset(), Offset::up());
        assert_eq!(Direction::South.offset(), Offset::down());
        assert_eq!(Direction::Northeast.offset(), Offset::up_right());
    }

    #[test]
    fn test_direction_apply() {
        let pos = Position::new(3, 3);
        assert_eq!(Direction::North.apply(&pos), Some(Position::new(2, 3)));
        assert_eq!(Direction::East.apply(&pos), Some(Position::new(3, 4)));
        assert_eq!(Direction::West.apply(&Position::new(0, 0)), None);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::Northeast.opposite(), Direction::Southwest);
    }

    #[test]
    fn test_direction_is_cardinal() {
        assert!(Direction::North.is_cardinal());
        assert!(Direction::East.is_cardinal());
        assert!(!Direction::Northeast.is_cardinal());
    }

    #[test]
    fn test_direction_is_diagonal() {
        assert!(Direction::Northeast.is_diagonal());
        assert!(Direction::Southwest.is_diagonal());
        assert!(!Direction::North.is_diagonal());
    }

    #[test]
    fn test_direction_angle() {
        assert_eq!(Direction::North.as_angle(), 0.0);
        assert_eq!(Direction::East.as_angle(), 90.0);
        assert_eq!(Direction::South.as_angle(), 180.0);
    }

    #[test]
    fn test_direction_from_angle() {
        assert_eq!(Direction::from_angle(0.0), Some(Direction::North));
        assert_eq!(Direction::from_angle(90.0), Some(Direction::East));
        assert_eq!(Direction::from_angle(45.0), Some(Direction::Northeast));
        assert_eq!(Direction::from_angle(100.0), None);
    }

    #[test]
    fn test_position_add_direction() {
        let pos = Position::new(3, 3);
        let result = pos + Direction::North;
        assert_eq!(result, Some(Position::new(2, 3)));
    }

    #[test]
    fn test_position_add_offset() {
        let pos = Position::new(3, 3);
        let offset = Offset::new(1, 2);
        let result = pos + offset;
        assert_eq!(result, Position::new(4, 5));
    }

    #[test]
    fn test_position_sub() {
        let pos1 = Position::new(5, 7);
        let pos2 = Position::new(3, 4);
        let offset = pos1 - pos2;
        assert_eq!(offset, Offset::new(2, 3));
    }
}

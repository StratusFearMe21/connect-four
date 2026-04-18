// core/direction.rs - Direction system for game logic
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Represents the 8 cardinal and ordinal directions on a 2D grid
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Direction {
    /// North (up)
    North,
    /// Northeast (up and right)
    Northeast,
    /// East (right)
    East,
    /// Southeast (down and right)
    Southeast,
    /// South (down)
    South,
    /// Southwest (down and left)
    Southwest,
    /// West (left)
    West,
    /// Northwest (up and left)
    Northwest,
}

impl Direction {
    /// Returns all 8 directions
    pub const fn all() -> [Direction; 8] {
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

    /// Returns the 4 cardinal directions (N, E, S, W)
    pub const fn cardinal() -> [Direction; 4] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }

    /// Returns the 4 ordinal/diagonal directions (NE, SE, SW, NW)
    pub const fn diagonal() -> [Direction; 4] {
        [
            Direction::Northeast,
            Direction::Southeast,
            Direction::Southwest,
            Direction::Northwest,
        ]
    }

    /// Returns the row delta for this direction (-1, 0, or 1)
    pub const fn row_delta(&self) -> isize {
        match self {
            Direction::North | Direction::Northeast | Direction::Northwest => -1,
            Direction::South | Direction::Southeast | Direction::Southwest => 1,
            Direction::East | Direction::West => 0,
        }
    }

    /// Returns the column delta for this direction (-1, 0, or 1)
    pub const fn col_delta(&self) -> isize {
        match self {
            Direction::East | Direction::Northeast | Direction::Southeast => 1,
            Direction::West | Direction::Northwest | Direction::Southwest => -1,
            Direction::North | Direction::South => 0,
        }
    }

    /// Returns the deltas as a tuple
    pub const fn deltas(&self) -> (isize, isize) {
        (self.row_delta(), self.col_delta())
    }

    /// Returns the opposite direction (180 degrees)
    pub const fn opposite(&self) -> Direction {
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

    /// Returns the direction rotated 90 degrees clockwise
    pub const fn clockwise(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            Direction::Northeast => Direction::Southeast,
            Direction::Southeast => Direction::Southwest,
            Direction::Southwest => Direction::Northwest,
            Direction::Northwest => Direction::Northeast,
        }
    }

    /// Returns the direction rotated 90 degrees counter-clockwise
    pub const fn counter_clockwise(&self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            Direction::Northeast => Direction::Northwest,
            Direction::Northwest => Direction::Southwest,
            Direction::Southwest => Direction::Southeast,
            Direction::Southeast => Direction::Northeast,
        }
    }

    /// Returns true if this is a cardinal direction
    pub const fn is_cardinal(&self) -> bool {
        matches!(
            self,
            Direction::North | Direction::East | Direction::South | Direction::West
        )
    }

    /// Returns true if this is a diagonal direction
    pub const fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::Northeast
                | Direction::Southeast
                | Direction::Southwest
                | Direction::Northwest
        )
    }

    /// Returns true if this direction is pointing north (N, NE, NW)
    pub const fn is_north(&self) -> bool {
        matches!(self, Direction::North | Direction::Northeast | Direction::Northwest)
    }

    /// Returns true if this direction is pointing south (S, SE, SW)
    pub const fn is_south(&self) -> bool {
        matches!(
            self,
            Direction::South | Direction::Southeast | Direction::Southwest
        )
    }

    /// Returns true if this direction is pointing east (E, NE, SE)
    pub const fn is_east(&self) -> bool {
        matches!(
            self,
            Direction::East | Direction::Northeast | Direction::Southeast
        )
    }

    /// Returns true if this direction is pointing west (W, NW, SW)
    pub const fn is_west(&self) -> bool {
        matches!(
            self,
            Direction::West | Direction::Northwest | Direction::Southwest
        )
    }

    /// Returns the direction as an angle in degrees from north (clockwise)
    pub const fn as_angle(&self) -> f64 {
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

    /// Returns the direction as radians
    pub fn as_radians(&self) -> f64 {
        self.as_angle() * std::f64::consts::PI / 180.0
    }

    /// Creates a direction from an angle in degrees from north
    pub fn from_angle_degrees(angle: f64) -> Option<Direction> {
        let angle = angle % 360.0;
        let angle = if angle < 0.0 { angle + 360.0 } else { angle };

        let sector = (angle / 45.0) as usize;
        match sector {
            0 => Some(Direction::North),
            1 => Some(Direction::Northeast),
            2 => Some(Direction::East),
            3 => Some(Direction::Southeast),
            4 => Some(Direction::South),
            5 => Some(Direction::Southwest),
            6 => Some(Direction::West),
            7 => Some(Direction::Northwest),
            _ => None,
        }
    }

    /// Creates a direction from an angle in radians
    pub fn from_angle_radians(radians: f64) -> Option<Direction> {
        let degrees = radians * 180.0 / std::f64::consts::PI;
        Self::from_angle_degrees(degrees)
    }

    /// Returns a short single-character representation
    pub const fn char(&self) -> char {
        match self {
            Direction::North => 'N',
            Direction::Northeast => '↗',
            Direction::East => 'E',
            Direction::Southeast => '↘',
            Direction::South => 'S',
            Direction::Southwest => '↙',
            Direction::West => 'W',
            Direction::Northwest => '↖',
        }
    }

    /// Returns an arrow character for this direction
    pub const fn arrow(&self) -> char {
        match self {
            Direction::North => '↑',
            Direction::Northeast => '↗',
            Direction::East => '→',
            Direction::Southeast => '↘',
            Direction::South => '↓',
            Direction::Southwest => '↙',
            Direction::West => '←',
            Direction::Northwest => '↖',
        }
    }

    /// Returns the combination of this direction with another
    pub fn combine(&self, other: Direction) -> Option<Direction> {
        let (dr1, dc1) = self.deltas();
        let (dr2, dc2) = other.deltas();

        let dr = dr1 + dr2;
        let dc = dc1 + dc2;

        match (dr, dc) {
            (-1, 0) => Some(Direction::North),
            (-1, 1) => Some(Direction::Northeast),
            (0, 1) => Some(Direction::East),
            (1, 1) => Some(Direction::Southeast),
            (1, 0) => Some(Direction::South),
            (1, -1) => Some(Direction::Southwest),
            (0, -1) => Some(Direction::West),
            (-1, -1) => Some(Direction::Northwest),
            _ => None,
        }
    }

    /// Returns the horizontal component (None, Some(Left), Some(Right))
    pub const fn horizontal(&self) -> Option<Horizontal> {
        match self {
            Direction::East | Direction::Northeast | Direction::Southeast => {
                Some(Horizontal::Right)
            }
            Direction::West | Direction::Northwest | Direction::Southwest => {
                Some(Horizontal::Left)
            }
            Direction::North | Direction::South => None,
        }
    }

    /// Returns the vertical component (None, Some(Up), Some(Down))
    pub const fn vertical(&self) -> Option<Vertical> {
        match self {
            Direction::North | Direction::Northeast | Direction::Northwest => Some(Vertical::Up),
            Direction::South | Direction::Southeast | Direction::Southwest => {
                Some(Vertical::Down)
            }
            Direction::East | Direction::West => None,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.arrow())
    }
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "n" | "north" | "up" | "↑" => Ok(Direction::North),
            "ne" | "northeast" | "up-right" | "↗" => Ok(Direction::Northeast),
            "e" | "east" | "right" | "→" => Ok(Direction::East),
            "se" | "southeast" | "down-right" | "↘" => Ok(Direction::Southeast),
            "s" | "south" | "down" | "↓" => Ok(Direction::South),
            "sw" | "southwest" | "down-left" | "↙" => Ok(Direction::Southwest),
            "w" | "west" | "left" | "←" => Ok(Direction::West),
            "nw" | "northwest" | "up-left" | "↖" => Ok(Direction::Northwest),
            _ => Err(format!("Invalid direction: {}", s)),
        }
    }
}

impl std::iter::Iterator for Direction {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        let directions = Direction::all();
        let current_idx = directions.iter().position(|&d| d == *self)?;

        if current_idx < directions.len() - 1 {
            let next = directions[current_idx + 1];
            *self = next;
            Some(next)
        } else {
            None
        }
    }
}

/// Represents horizontal direction component
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Horizontal {
    Left,
    Right,
}

impl Horizontal {
    pub const fn delta(&self) -> isize {
        match self {
            Horizontal::Left => -1,
            Horizontal::Right => 1,
        }
    }

    pub const fn opposite(&self) -> Self {
        match self {
            Horizontal::Left => Horizontal::Right,
            Horizontal::Right => Horizontal::Left,
        }
    }
}

/// Represents vertical direction component
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Vertical {
    Up,
    Down,
}

impl Vertical {
    pub const fn delta(&self) -> isize {
        match self {
            Vertical::Up => -1,
            Vertical::Down => 1,
        }
    }

    pub const fn opposite(&self) -> Self {
        match self {
            Vertical::Up => Vertical::Down,
            Vertical::Down => Vertical::Up,
        }
    }
}

/// Helper struct to iterate through win line directions
#[derive(Clone, Copy, Debug)]
pub struct WinDirectionIterator {
    index: usize,
}

impl WinDirectionIterator {
    pub const fn new() -> Self {
        WinDirectionIterator { index: 0 }
    }
}

impl Default for WinDirectionIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for WinDirectionIterator {
    type Item = (Direction, Direction);

    fn next(&mut self) -> Option<Self::Item> {
        const WIN_LINES: [Direction; 4] = [
            Direction::East,
            Direction::South,
            Direction::Southeast,
            Direction::Southwest,
        ];

        if self.index < WIN_LINES.len() {
            let dir = WIN_LINES[self.index];
            self.index += 1;
            Some((dir, dir.opposite()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_all() {
        let all = Direction::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn test_direction_cardinal() {
        let cardinal = Direction::cardinal();
        assert_eq!(cardinal.len(), 4);
        assert!(cardinal.contains(&Direction::North));
        assert!(!cardinal.contains(&Direction::Northeast));
    }

    #[test]
    fn test_direction_diagonal() {
        let diagonal = Direction::diagonal();
        assert_eq!(diagonal.len(), 4);
        assert!(diagonal.contains(&Direction::Northeast));
        assert!(!diagonal.contains(&Direction::North));
    }

    #[test]
    fn test_direction_deltas() {
        assert_eq!(Direction::North.deltas(), (-1, 0));
        assert_eq!(Direction::East.deltas(), (0, 1));
        assert_eq!(Direction::Southeast.deltas(), (1, 1));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::Northeast.opposite(), Direction::Southwest);
    }

    #[test]
    fn test_direction_clockwise() {
        assert_eq!(Direction::North.clockwise(), Direction::East);
        assert_eq!(Direction::East.clockwise(), Direction::South);
        assert_eq!(Direction::Northeast.clockwise(), Direction::Southeast);
    }

    #[test]
    fn test_direction_counter_clockwise() {
        assert_eq!(Direction::North.counter_clockwise(), Direction::West);
        assert_eq!(Direction::West.counter_clockwise(), Direction::South);
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
    fn test_direction_is_north() {
        assert!(Direction::North.is_north());
        assert!(Direction::Northeast.is_north());
        assert!(!Direction::South.is_north());
    }

    #[test]
    fn test_direction_as_angle() {
        assert_eq!(Direction::North.as_angle(), 0.0);
        assert_eq!(Direction::East.as_angle(), 90.0);
        assert_eq!(Direction::South.as_angle(), 180.0);
        assert_eq!(Direction::West.as_angle(), 270.0);
    }

    #[test]
    fn test_direction_from_angle() {
        assert_eq!(Direction::from_angle_degrees(0.0), Some(Direction::North));
        assert_eq!(Direction::from_angle_degrees(90.0), Some(Direction::East));
        assert_eq!(Direction::from_angle_degrees(45.0), Some(Direction::Northeast));
        assert_eq!(Direction::from_angle_degrees(100.0), Some(Direction::Southeast));
        assert_eq!(Direction::from_angle_degrees(360.0), Some(Direction::North));
    }

    #[test]
    fn test_direction_from_negative_angle() {
        assert_eq!(
            Direction::from_angle_degrees(-90.0),
            Some(Direction::West)
        );
        assert_eq!(
            Direction::from_angle_degrees(-45.0),
            Some(Direction::Northwest)
        );
    }

    #[test]
    fn test_direction_from_str() {
        assert_eq!(Direction::from_str("N").unwrap(), Direction::North);
        assert_eq!(Direction::from_str("south").unwrap(), Direction::South);
        assert_eq!(Direction::from_str("ne").unwrap(), Direction::Northeast);
        assert!(Direction::from_str("invalid").is_err());
    }

    #[test]
    fn test_direction_char() {
        assert_eq!(Direction::North.char(), 'N');
        assert_eq!(Direction::East.char(), 'E');
    }

    #[test]
    fn test_direction_arrow() {
        assert_eq!(Direction::North.arrow(), '↑');
        assert_eq!(Direction::East.arrow(), '→');
        assert_eq!(Direction::South.arrow(), '↓');
    }

    #[test]
    fn test_direction_combine() {
        assert_eq!(
            Direction::North.combine(Direction::East),
            Some(Direction::Northeast)
        );
        assert_eq!(
            Direction::North.combine(Direction::South),
            Some(Direction::North)
        );
    }

    #[test]
    fn test_horizontal() {
        assert_eq!(Direction::East.horizontal(), Some(Horizontal::Right));
        assert_eq!(Direction::West.horizontal(), Some(Horizontal::Left));
        assert_eq!(Direction::North.horizontal(), None);
    }

    #[test]
    fn test_vertical() {
        assert_eq!(Direction::North.vertical(), Some(Vertical::Up));
        assert_eq!(Direction::South.vertical(), Some(Vertical::Down));
        assert_eq!(Direction::East.vertical(), None);
    }

    #[test]
    fn test_horizontal_delta() {
        assert_eq!(Horizontal::Left.delta(), -1);
        assert_eq!(Horizontal::Right.delta(), 1);
    }

    #[test]
    fn test_vertical_delta() {
        assert_eq!(Vertical::Up.delta(), -1);
        assert_eq!(Vertical::Down.delta(), 1);
    }

    #[test]
    fn test_horizontal_opposite() {
        assert_eq!(Horizontal::Left.opposite(), Horizontal::Right);
    }

    #[test]
    fn test_vertical_opposite() {
        assert_eq!(Vertical::Up.opposite(), Vertical::Down);
    }

    #[test]
    fn test_win_direction_iterator() {
        let mut iter = WinDirectionIterator::new();
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }
}

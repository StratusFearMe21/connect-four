// core/piece.rs - Piece types and properties
use crate::core::cell::Cell;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Represents a piece that can be placed on the board
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub cell: Cell,
    pub owner: u8,  // Player number (1 or 2)
    pub special: SpecialType,
}

impl Piece {
    /// Creates a new regular piece
    pub fn new(cell: Cell, owner: u8) -> Self {
        Piece {
            cell,
            owner,
            special: SpecialType::None,
        }
    }

    /// Creates a regular player 1 piece
    pub const fn player1() -> Self {
        Piece {
            cell: Cell::Player1,
            owner: 1,
            special: SpecialType::None,
        }
    }

    /// Creates a regular player 2 piece
    pub const fn player2() -> Self {
        Piece {
            cell: Cell::Player2,
            owner: 2,
            special: SpecialType::None,
        }
    }

    /// Creates a piece with a special type
    pub fn with_special(cell: Cell, owner: u8, special: SpecialType) -> Self {
        Piece {
            cell,
            owner,
            special,
        }
    }

    /// Creates a blocker piece
    pub fn blocker(owner: u8) -> Self {
        Piece {
            cell: Cell::Blocker,
            owner,
            special: SpecialType::Blocker,
        }
    }

    /// Creates an explosive piece
    pub fn explosive(owner: u8) -> Self {
        Piece {
            cell: Cell::Explosive,
            owner,
            special: SpecialType::Explosive,
        }
    }

    /// Creates a bomb piece
    pub fn bomb(owner: u8) -> Self {
        Piece {
            cell: Cell::Bomb,
            owner,
            special: SpecialType::Bomb,
        }
    }

    /// Returns the cell type
    pub const fn cell_type(&self) -> Cell {
        self.cell
    }

    /// Returns the owner player number
    pub const fn owner(&self) -> u8 {
        self.owner
    }

    /// Returns the special type
    pub const fn special_type(&self) -> SpecialType {
        self.special
    }

    /// Returns true if this is a special piece
    pub const fn is_special(&self) -> bool {
        self.special != SpecialType::None
    }

    /// Returns true if this piece can be destroyed
    pub const fn is_destroyable(&self) -> bool {
        !matches!(self.special, SpecialType::Indestructible)
    }

    /// Returns true if this piece counts toward wins
    pub const fn can_win(&self) -> bool {
        matches!(self.cell, Cell::Player1 | Cell::Player2)
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cell)
    }
}

/// Special piece types
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum SpecialType {
    /// No special ability
    None,
    /// Blocks win detection
    Blocker,
    /// Destroys adjacent pieces when placed
    Explosive,
    /// Cannot be destroyed
    Indestructible,
    /// Clears entire column when placed
    Bomb,
    /// Acts as wildcard for any player
    Wild,
    /// Doubles the value of a winning line
    Double,
    /// Freezes a cell for one turn
    Freeze,
    /// Teleports to a random empty cell
    Teleport,
    /// Swaps with another piece
    Swap,
}

impl SpecialType {
    /// Returns the name of this special type
    pub const fn name(&self) -> &'static str {
        match self {
            SpecialType::None => "None",
            SpecialType::Blocker => "Blocker",
            SpecialType::Explosive => "Explosive",
            SpecialType::Indestructible => "Indestructible",
            SpecialType::Bomb => "Bomb",
            SpecialType::Wild => "Wild",
            SpecialType::Double => "Double",
            SpecialType::Freeze => "Freeze",
            SpecialType::Teleport => "Teleport",
            SpecialType::Swap => "Swap",
        }
    }

    /// Returns a description of this special ability
    pub const fn description(&self) -> &'static str {
        match self {
            SpecialType::None => "No special ability",
            SpecialType::Blocker => "Blocks win detection for adjacent cells",
            SpecialType::Explosive => "Destroys adjacent pieces when placed",
            SpecialType::Indestructible => "Cannot be destroyed",
            SpecialType::Bomb => "Clears entire column when placed",
            SpecialType::Wild => "Counts as either player for win detection",
            SpecialType::Double => "Doubles the value of a winning line",
            SpecialType::Freeze => "Freezes a cell for one turn",
            SpecialType::Teleport => "Moves to a random empty cell after placement",
            SpecialType::Swap => "Swaps positions with another piece",
        }
    }

    /// Returns true if this special type modifies the board
    pub const fn modifies_board(&self) -> bool {
        matches!(
            self,
            SpecialType::Explosive
                | SpecialType::Bomb
                | SpecialType::Teleport
                | SpecialType::Swap
        )
    }
}

/// Piece stack for managing multiple pieces in a column
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PieceStack {
    pieces: Vec<Piece>,
    max_height: usize,
}

impl PieceStack {
    /// Creates a new empty piece stack
    pub fn new(max_height: usize) -> Self {
        PieceStack {
            pieces: Vec::with_capacity(max_height),
            max_height,
        }
    }

    /// Returns the height of the stack
    pub fn height(&self) -> usize {
        self.pieces.len()
    }

    /// Returns the maximum height
    pub fn max_height(&self) -> usize {
        self.max_height
    }

    /// Returns true if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    /// Returns true if the stack is full
    pub fn is_full(&self) -> bool {
        self.pieces.len() >= self.max_height
    }

    /// Returns the top piece
    pub fn top(&self) -> Option<&Piece> {
        self.pieces.last()
    }

    /// Returns the bottom piece
    pub fn bottom(&self) -> Option<&Piece> {
        self.pieces.first()
    }

    /// Pushes a piece onto the stack
    pub fn push(&mut self, piece: Piece) -> Result<(), String> {
        if self.is_full() {
            return Err("Stack is full".to_string());
        }
        self.pieces.push(piece);
        Ok(())
    }

    /// Pops a piece from the stack
    pub fn pop(&mut self) -> Option<Piece> {
        self.pieces.pop()
    }

    /// Peeks at a specific position in the stack
    pub fn peek(&self, index: usize) -> Option<&Piece> {
        self.pieces.get(index)
    }

    /// Clears all pieces from the stack
    pub fn clear(&mut self) {
        self.pieces.clear();
    }

    /// Returns an iterator over the pieces
    pub fn iter(&self) -> impl Iterator<Item = &Piece> {
        self.pieces.iter()
    }

    /// Counts pieces of a specific owner
    pub fn count_owner(&self, owner: u8) -> usize {
        self.pieces.iter().filter(|p| p.owner == owner).count()
    }
}

/// Piece inventory for managing available special pieces
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PieceInventory {
    regular: u32,
    blockers: u32,
    explosives: u32,
    bombs: u32,
    wilds: u32,
}

impl PieceInventory {
    /// Creates a new inventory with unlimited regular pieces
    pub fn new() -> Self {
        PieceInventory {
            regular: u32::MAX,
            blockers: 0,
            explosives: 0,
            bombs: 0,
            wilds: 0,
        }
    }

    /// Creates an inventory with limited special pieces
    pub fn with_limited(
        blockers: u32,
        explosives: u32,
        bombs: u32,
        wilds: u32,
    ) -> Self {
        PieceInventory {
            regular: u32::MAX,
            blockers,
            explosives,
            bombs,
            wilds,
        }
    }

    /// Returns the count of regular pieces (always unlimited)
    pub const fn regular_count(&self) -> u32 {
        self.regular
    }

    /// Returns the count of blockers
    pub const fn blockers(&self) -> u32 {
        self.blockers
    }

    /// Returns the count of explosives
    pub const fn explosives(&self) -> u32 {
        self.explosives
    }

    /// Returns the count of bombs
    pub const fn bombs(&self) -> u32 {
        self.bombs
    }

    /// Returns the count of wilds
    pub const fn wilds(&self) -> u32 {
        self.wilds
    }

    /// Uses a special piece, returning it if available
    pub fn use_special(&mut self, special: SpecialType, owner: u8) -> Option<Piece> {
        match special {
            SpecialType::Blocker if self.blockers > 0 => {
                self.blockers -= 1;
                Some(Piece::blocker(owner))
            }
            SpecialType::Explosive if self.explosives > 0 => {
                self.explosives -= 1;
                Some(Piece::explosive(owner))
            }
            SpecialType::Bomb if self.bombs > 0 => {
                self.bombs -= 1;
                Some(Piece::bomb(owner))
            }
            SpecialType::Wild if self.wilds > 0 => {
                self.wilds -= 1;
                Some(Piece::with_special(Cell::Wild, owner, SpecialType::Wild))
            }
            _ => None,
        }
    }

    /// Adds special pieces to the inventory
    pub fn add_special(&mut self, special: SpecialType, count: u32) {
        match special {
            SpecialType::Blocker => {
                self.blockers = self.blockers.saturating_add(count);
            }
            SpecialType::Explosive => {
                self.explosives = self.explosives.saturating_add(count);
            }
            SpecialType::Bomb => {
                self.bombs = self.bombs.saturating_add(count);
            }
            SpecialType::Wild => {
                self.wilds = self.wilds.saturating_add(count);
            }
            _ => {}
        }
    }
}

impl Default for PieceInventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_new() {
        let piece = Piece::new(Cell::Player1, 1);
        assert_eq!(piece.cell_type(), Cell::Player1);
        assert_eq!(piece.owner(), 1);
    }

    #[test]
    fn test_piece_player1() {
        let piece = Piece::player1();
        assert_eq!(piece.cell_type(), Cell::Player1);
        assert_eq!(piece.owner(), 1);
    }

    #[test]
    fn test_piece_is_special() {
        let regular = Piece::player1();
        let special = Piece::explosive(1);
        assert!(!regular.is_special());
        assert!(special.is_special());
    }

    #[test]
    fn test_special_type_name() {
        assert_eq!(SpecialType::None.name(), "None");
        assert_eq!(SpecialType::Explosive.name(), "Explosive");
    }

    #[test]
    fn test_piece_stack() {
        let mut stack = PieceStack::new(6);
        assert!(stack.is_empty());
        assert!(!stack.is_full());

        stack.push(Piece::player1()).unwrap();
        assert_eq!(stack.height(), 1);
        assert!(!stack.is_empty());

        let top = stack.top();
        assert!(top.is_some());
    }

    #[test]
    fn test_piece_stack_full() {
        let mut stack = PieceStack::new(2);
        stack.push(Piece::player1()).unwrap();
        stack.push(Piece::player2()).unwrap();
        assert!(stack.is_full());

        let result = stack.push(Piece::player1());
        assert!(result.is_err());
    }

    #[test]
    fn test_piece_inventory() {
        let inventory = PieceInventory::with_limited(3, 2, 1, 0);
        assert_eq!(inventory.blockers(), 3);
        assert_eq!(inventory.explosives(), 2);
    }

    #[test]
    fn test_piece_inventory_use_special() {
        let mut inventory = PieceInventory::with_limited(3, 2, 1, 0);
        let piece = inventory.use_special(SpecialType::Blocker, 1);
        assert!(piece.is_some());
        assert_eq!(inventory.blockers(), 2);
    }

    #[test]
    fn test_piece_inventory_use_special_none() {
        let mut inventory = PieceInventory::new();
        let piece = inventory.use_special(SpecialType::Blocker, 1);
        assert!(piece.is_none());
    }
}

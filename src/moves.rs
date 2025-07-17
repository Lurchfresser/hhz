use crate::bit_boards::*;
use crate::board::{Board, Piece, PieceKind};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::slice::Iter;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Move {
    mask: u16,
}

impl Move {
    /// Mask for the source ("from") bits.
    const SRC_MASK: u16 = 0b0000_0000_0011_1111;
    /// Mask for the destination ("to") bits.
    const DST_MASK: u16 = 0b0000_1111_1100_0000;
    /// Mask for the flag (promotions, captures, etc.) bits.
    const FLG_MASK: u16 = 0b1111_0000_0000_0000;
    /// Start index of destination bits.
    const DST_BITS: u16 = 6;
    /// Start index of flag bits.
    const FLG_BITS: u16 = 12;

    const FLAG_QUIET: u16 = 0 << Self::FLG_BITS;
    const FLAG_CASTLE_SHORT: u16 = 2 << Self::FLG_BITS;
    const FLAG_CASTLE_LONG: u16 = 3 << Self::FLG_BITS;
    const FLAG_CAPTURE: u16 = 4 << Self::FLG_BITS;
    const FLAG_EP_CAPTURE: u16 = 5 << Self::FLG_BITS;
    const FLAG_PROMO_KNIGHT: u16 = 8 << Self::FLG_BITS;
    const FLAG_PROMO_BISHOP: u16 = 9 << Self::FLG_BITS;
    const FLAG_PROMO_ROOK: u16 = 10 << Self::FLG_BITS;
    const FLAG_PROMO_QUEEN: u16 = 11 << Self::FLG_BITS;
    const FLAG_CAPTURE_PROMO_KNIGHT: u16 = 12 << Self::FLG_BITS;
    const FLAG_CAPTURE_PROMO_BISHOP: u16 = 13 << Self::FLG_BITS;
    const FLAG_CAPTURE_PROMO_ROOK: u16 = 14 << Self::FLG_BITS;
    const FLAG_CAPTURE_PROMO_QUEEN: u16 = 15 << Self::FLG_BITS;

    #[inline(always)]
    pub fn new(from: usize, to: usize, capture: bool) -> Self {
        let mask = from as u16 | (to as u16) << Self::DST_BITS | Self::FLAG_QUIET;
        if capture {
            Self {
                mask: mask | Self::FLAG_CAPTURE,
            }
        } else {
            Self { mask }
        }
    }

    #[inline(always)]
    pub fn castles(is_king_side: bool, white_to_move: bool) -> Self {
        let from = if white_to_move { 4 } else { 60 };
        let to = if white_to_move {
            if is_king_side {
                WHITE_KINGSIDE_CASTLE_INDEX
            } else {
                WHITE_QUEENSIDE_CASTLE_INDEX
            }
        } else if is_king_side {
            BLACK_KINGSIDE_CASTLE_INDEX
        } else {
            BLACK_QUEENSIDE_CASTLE_INDEX
        };

        let flag = if is_king_side {
            Self::FLAG_CASTLE_SHORT
        } else {
            Self::FLAG_CASTLE_LONG
        };

        Self {
            mask: from as u16 | (to as u16) << Self::DST_BITS | flag,
        }
    }

    #[inline(always)]
    pub fn en_passant(from: usize, to: usize) -> Self {
        Self {
            mask: from as u16 | (to as u16) << Self::DST_BITS | Self::FLAG_EP_CAPTURE,
        }
    }

    #[inline(always)]
    pub fn promotion(piece_kind: PieceKind, from_index: usize, to_index: usize) -> Self {
        let flag = match piece_kind {
            PieceKind::Knight => Self::FLAG_PROMO_KNIGHT,
            PieceKind::Bishop => Self::FLAG_PROMO_BISHOP,
            PieceKind::Rook => Self::FLAG_PROMO_ROOK,
            PieceKind::Queen => Self::FLAG_PROMO_QUEEN,
            _ => panic!("Invalid promotion piece"),
        };

        Self {
            mask: from_index as u16 | (to_index as u16) << Self::DST_BITS | flag,
        }
    }

    #[inline(always)]
    pub fn capture_promotion(piece_kind: PieceKind, from_index: usize, to_index: usize) -> Self {
        let flag = match piece_kind {
            PieceKind::Knight => Self::FLAG_CAPTURE_PROMO_KNIGHT,
            PieceKind::Bishop => Self::FLAG_CAPTURE_PROMO_BISHOP,
            PieceKind::Rook => Self::FLAG_CAPTURE_PROMO_ROOK,
            PieceKind::Queen => Self::FLAG_CAPTURE_PROMO_QUEEN,
            _ => panic!("Invalid promotion piece"),
        };

        Self {
            mask: from_index as u16 | (to_index as u16) << Self::DST_BITS | flag,
        }
    }

    #[inline(always)]
    pub fn from(&self) -> usize {
        (self.mask & Self::SRC_MASK) as usize
    }

    #[inline(always)]
    pub fn to(&self) -> usize {
        ((self.mask & Self::DST_MASK) >> Self::DST_BITS) as usize
    }

    #[inline(always)]
    pub fn flag(&self) -> u16 {
        self.mask & Self::FLG_MASK
    }

    #[inline(always)]
    pub fn is_promotion(&self) -> bool {
        let flag = self.flag();
        //todo. DONT USE RANGE, IT IS SLOW
        (Self::FLAG_PROMO_KNIGHT..=Self::FLAG_CAPTURE_PROMO_QUEEN).contains(&flag)
    }

    #[inline(always)]
    pub fn promotion_piece(&self) -> Option<PieceKind> {
        match self.flag() {
            Self::FLAG_PROMO_KNIGHT | Self::FLAG_CAPTURE_PROMO_KNIGHT => Some(PieceKind::Knight),
            Self::FLAG_PROMO_BISHOP | Self::FLAG_CAPTURE_PROMO_BISHOP => Some(PieceKind::Bishop),
            Self::FLAG_PROMO_ROOK | Self::FLAG_CAPTURE_PROMO_ROOK => Some(PieceKind::Rook),
            Self::FLAG_PROMO_QUEEN | Self::FLAG_CAPTURE_PROMO_QUEEN => Some(PieceKind::Queen),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_capture(&self) -> bool {
        let flag = self.flag();
        flag == Self::FLAG_CAPTURE
            || flag == Self::FLAG_EP_CAPTURE
            || (flag >= Self::FLAG_CAPTURE_PROMO_KNIGHT && flag <= Self::FLAG_CAPTURE_PROMO_QUEEN)
    }

    #[inline(always)]
    pub fn is_castle(&self) -> bool {
        let flag = self.flag();
        flag == Self::FLAG_CASTLE_SHORT || flag == Self::FLAG_CASTLE_LONG
    }

    #[inline(always)]
    pub fn is_castle_short(&self) -> bool {
        self.flag() == Self::FLAG_CASTLE_SHORT
    }

    #[inline(always)]
    pub fn is_en_passant(&self) -> bool {
        self.flag() == Self::FLAG_EP_CAPTURE
    }

    #[inline(always)]
    pub fn is_castle_long(&self) -> bool {
        self.flag() == Self::FLAG_CASTLE_LONG
    }

    pub fn to_uci(&self) -> String {
        let from_square = square_index_to_square(self.from());
        let to_square = square_index_to_square(self.to());
        let mut uci = format!(
            "{}{}",
            square_to_algebraic(from_square),
            square_to_algebraic(to_square)
        );

        // Add promotion letter if this is a promotion move
        match self.flag() {
            // Regular promotions
            Self::FLAG_PROMO_KNIGHT | Self::FLAG_CAPTURE_PROMO_KNIGHT => uci.push('n'),
            Self::FLAG_PROMO_BISHOP | Self::FLAG_CAPTURE_PROMO_BISHOP => uci.push('b'),
            Self::FLAG_PROMO_ROOK | Self::FLAG_CAPTURE_PROMO_ROOK => uci.push('r'),
            Self::FLAG_PROMO_QUEEN | Self::FLAG_CAPTURE_PROMO_QUEEN => uci.push('q'),
            _ => {}
        }

        uci
    }
    #[inline(always)]
    pub fn resets_clock(&self, board: &Board) -> bool {
        self.is_capture() || matches!(board.pieces[self.from()], Piece::Pawn {..})
    }
}

pub fn square_to_algebraic(square: Square) -> String {
    format!("{}{}", ((square.file + 97) as u8) as char, square.rank + 1)
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("algebraic", &self.to_uci())
            .field("from", &self.from())
            .field("to", &self.to())
            .field("mask", &format!("{:#06x}", self.mask))
            // Add any other fields you want to see
            .finish()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

pub const MAX_NUM_MOVES: usize = 218;

// --- START: Corrected MoveList for proper debugging ---

// Changed to a standard struct with a private field to enforce encapsulation.
// This forces the debugger to use our custom `Debug` implementation.
pub struct MoveList {
    moves: arrayvec::ArrayVec<Move, MAX_NUM_MOVES>,
}

// For owned Move values
impl FromIterator<Move> for MoveList {
    fn from_iter<I: IntoIterator<Item=Move>>(iter: I) -> Self {
        let mut movelist = MoveList::default();
        for m in iter {
            movelist.push(m);
        }
        movelist
    }
}

impl<'a> FromIterator<&'a Move> for MoveList {
    fn from_iter<I: IntoIterator<Item=&'a Move>>(iter: I) -> Self {
        let mut movelist = MoveList::default();
        for &m in iter {
            movelist.push(m);
        }
        movelist
    }
}
/// Custom Debug implementation for MoveList.
impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // This is the key change: we pass an iterator of `&Move` objects.
        // The debugger can then render each `Move` interactively using its own `Debug` impl.
        // Your previous version mapped them to Strings, which the debugger cannot expand.
        f.debug_list()
            .entries(self.moves.iter().map(|m| m.to_uci()))
            .finish()
    }
}

/// Implement Default to allow creating a new, empty MoveList with `MoveList::default()`.
impl Default for MoveList {
    fn default() -> Self {
        Self {
            moves: arrayvec::ArrayVec::new(),
        }
    }
}

/// Implement Deref to allow the MoveList wrapper to be used transparently
/// like the underlying ArrayVec (e.g., for iteration, `len()`, `is_empty()`).
impl Deref for MoveList {
    type Target = arrayvec::ArrayVec<Move, MAX_NUM_MOVES>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.moves
    }
}

/// Implement DerefMut to allow mutating the MoveList wrapper transparently
/// (e.g., calling `moves.push()`).
impl DerefMut for MoveList {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moves
    }
}

/// Implementation for consuming iteration (e.g., `for move in move_list`).
impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = arrayvec::IntoIter<Move, MAX_NUM_MOVES>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves.into_iter()
    }
}

/// Implementation for shared-reference iteration (e.g., `for move in &move_list`).
impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves.iter()
    }
}

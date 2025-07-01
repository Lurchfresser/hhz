use crate::bit_boards::*;
use regex::Regex;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::slice::Iter;

pub const MAX_NUM_MOVES: usize = 218;

// --- START: Corrected MoveList for proper debugging ---

// Changed to a standard struct with a private field to enforce encapsulation.
// This forces the debugger to use our custom `Debug` implementation.
pub struct MoveList {
    moves: arrayvec::ArrayVec<Move, MAX_NUM_MOVES>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingRights {
    All,
    OnlyKingSide,
    OnlyQueenSide,
    None,
}

struct PinAndCheckInfos {
    sliding_checkers: u64,
    stop_check_targets: u64,
    bishop_pinned_pieces: u64,
    rook_pinned_pieces: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board {
    white_pawns: u64,
    white_knights: u64,
    white_bishops: u64,
    white_rooks: u64,
    white_queens: u64,
    white_king: u64,

    white_pieces: u64,

    black_pawns: u64,
    black_knights: u64,
    black_bishops: u64,
    black_rooks: u64,
    black_queens: u64,
    black_king: u64,

    black_pieces: u64,

    all_pieces: u64,

    en_passant_target: u64,

    white_castling_rights: CastlingRights,
    black_castling_rights: CastlingRights,

    halfmove_clock: u32,
    fullmove_number: u32,

    white_to_move: bool,
}

impl Board {
    pub fn from_fen(fen: &str) -> Self {
        // Parse the FEN string and initialize the board
        // This is a placeholder implementation
        let mut white_pawns = 0;
        let mut white_knights = 0;
        let mut white_bishops = 0;
        let mut white_rooks = 0;
        let mut white_queens = 0;
        let mut white_king = 0;

        let mut black_pawns = 0;
        let mut black_knights = 0;
        let mut black_bishops = 0;
        let mut black_rooks = 0;
        let mut black_queens = 0;
        let mut black_king = 0;

        let mut file: i32 = 1;
        let mut rank: i32 = 8;

        for c in fen.chars() {
            let square = ((rank - 1) * 8 + (file)) as u64;
            match c {
                '1'..='8' => {
                    file += c.to_string().parse::<i32>().unwrap();
                }
                'p' => {
                    black_pawns |= 1 << (square - 1);
                    file += 1;
                }
                'r' => {
                    black_rooks |= 1 << (square - 1);
                    file += 1;
                }
                'n' => {
                    black_knights |= 1 << (square - 1);
                    file += 1;
                }
                'b' => {
                    black_bishops |= 1 << (square - 1);
                    file += 1;
                }
                'q' => {
                    black_queens |= 1 << (square - 1);
                    file += 1;
                }
                'k' => {
                    black_king |= 1 << (square - 1);
                    file += 1;
                }
                'P' => {
                    white_pawns |= 1 << (square - 1);
                    file += 1;
                }
                'R' => {
                    white_rooks |= 1 << (square - 1);
                    file += 1;
                }
                'N' => {
                    white_knights |= 1 << (square - 1);
                    file += 1;
                }
                'B' => {
                    white_bishops |= 1 << (square - 1);
                    file += 1;
                }
                'Q' => {
                    white_queens |= 1 << (square - 1);
                    file += 1;
                }
                'K' => {
                    white_king |= 1 << (square - 1);
                    file += 1;
                }
                '/' => {
                    file = 1; // Reset file for the next
                    rank -= 1; // Move to the next rank
                }
                ' ' => break,
                _ => panic!("Invalid FEN character: {}", c),
            }
        }

        let white_pieces =
            white_pawns | white_knights | white_bishops | white_rooks | white_queens | white_king;
        let black_pieces =
            black_pawns | black_knights | black_bishops | black_rooks | black_queens | black_king;
        let all_pieces = white_pieces | black_pieces;

        // Parse the rest of the FEN string for additional information
        let mut parts = fen.split_whitespace();

        parts.next(); // Skip the piece placement part

        let side_move_str = parts.next().unwrap();
        assert!(
            side_move_str == "w" || side_move_str == "b",
            "Invalid side to move in FEN: {}",
            fen
        );
        let white_to_move = side_move_str == "w";

        let castling_rights_str = parts.next().unwrap();
        assert!(
            Regex::new("(-|K?Q?k?q?)")
                .unwrap()
                .is_match(castling_rights_str),
            "Invalid castling rights in FEN: {}",
            fen
        );

        let white_castling_rights = if castling_rights_str.contains("KQ") {
            CastlingRights::All
        } else if castling_rights_str.contains("Q") {
            CastlingRights::OnlyQueenSide
        } else if castling_rights_str.contains("K") {
            CastlingRights::OnlyKingSide
        } else {
            CastlingRights::None
        };

        let black_castling_rights = if castling_rights_str.contains("kq") {
            CastlingRights::All
        } else if castling_rights_str.contains("q") {
            CastlingRights::OnlyQueenSide
        } else if castling_rights_str.contains("k") {
            CastlingRights::OnlyKingSide
        } else {
            CastlingRights::None
        };

        let en_passant_target = if let Some(s) = parts.next() {
            if s == "-" {
                0u64
            } else {
                // Convert the square to a bitboard position
                let file = s.chars().next().unwrap() as u8 - b'a';
                let rank = s.chars().nth(1).unwrap() as u8 - b'1';
                1u64 << (rank * 8 + file)
            }
        } else {
            0u64
        };

        let halfmove_clock = parts.next().unwrap().parse::<u32>().unwrap();
        let fullmove_number = parts.next().unwrap().parse::<u32>().unwrap();

        Board {
            white_pawns,
            white_knights,
            white_bishops,
            white_rooks,
            white_queens,
            white_king,
            white_pieces,
            black_pawns,
            black_knights,
            black_bishops,
            black_rooks,
            black_queens,
            black_king,
            black_pieces,
            all_pieces,
            white_to_move,
            en_passant_target,
            white_castling_rights,
            black_castling_rights,
            halfmove_clock,
            fullmove_number,
        }
    }

    //TODO: merge with gen-pseudo-legal-move-gen
    pub fn get_enemy_pawn_and_knight_checkers(&self) -> u64 {
        let king_index = if self.white_to_move {
            bitboard_to_square_index(self.white_king)
        } else {
            bitboard_to_square_index(self.black_king)
        };

        let enemy_pawns = if self.white_to_move {
            self.black_pawns
        } else {
            self.white_pawns
        };

        let enemy_knights = if self.white_to_move {
            self.black_knights
        } else {
            self.white_knights
        };

        #[allow(non_snake_case)]
        let ATTACKS_LOOKUP = if self.white_to_move {
            &WHITE_FREE_PAWN_ATTACKS_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ATTACKS_LOOKUP
        };

        let pawn_attacks = ATTACKS_LOOKUP[king_index] & enemy_pawns;

        let knight_attacks = FREE_KNIGHT_LOOKUP[king_index] & enemy_knights;

        knight_attacks | pawn_attacks
    }

    //TODO: add pawn structure lookup table
    pub fn gen_pawn_attack_squares(&self, for_white: bool) -> u64 {
        let mut pawns = if for_white {
            self.white_pawns
        } else {
            self.black_pawns
        };

        #[allow(non_snake_case)]
        let ATTACKS_LOOKUP = if for_white {
            &WHITE_FREE_PAWN_ATTACKS_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ATTACKS_LOOKUP
        };

        let mut pawn_attacks = 0u64;
        while pawns != 0 {
            let pawn_index = pop_lsb(&mut pawns);
            // 1. Generate attacks
            pawn_attacks |= ATTACKS_LOOKUP[pawn_index];
        }
        pawn_attacks
    }

    pub fn generate_pawn_moves(
        &self,
        moves: &mut MoveList,
        rook_pinned_pieces: u64,
        bishop_pinned_pieces: u64,
        to_mask: u64,
    ) {
        let mut pawns = if self.white_to_move {
            self.white_pawns
        } else {
            self.black_pawns
        };

        let enemy_pieces = if self.white_to_move {
            self.black_pieces
        } else {
            self.white_pieces
        };

        let own_king = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };

        #[allow(non_snake_case)]
        let ADVANCE_LOOKUP = if self.white_to_move {
            &WHITE_FREE_PAWN_ADVANCE_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ADVANCE_LOOKUP
        };

        #[allow(non_snake_case)]
        let ATTACKS_LOOKUP = if self.white_to_move {
            &WHITE_FREE_PAWN_ATTACKS_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ATTACKS_LOOKUP
        };

        while pawns != 0 {
            let mut moves_for_pawn = 0u64;
            let pawn_index = pop_lsb(&mut pawns);
            let pawn_bit_board = square_index_to_bitboard(pawn_index);

            // 1. Generate attacks
            moves_for_pawn |= ATTACKS_LOOKUP[pawn_index]
            //                      mask out illegal en passant captures
                & (enemy_pieces | (self.en_passant_target & !rook_pinned_pieces));

            // 2. Generate advances
            let blockers = self.all_pieces ^ pawn_bit_board;
            let advances = ADVANCE_LOOKUP[pawn_index];

            if self.white_to_move {
                // For white, shift blockers UP to see if they block the double advance
                let invalid_advances = blockers | (blockers << 8);
                moves_for_pawn |= advances & !invalid_advances;
            } else {
                // For black, shift blockers DOWN to see if they block the double advance
                let invalid_advances = blockers | (blockers >> 8);
                moves_for_pawn |= advances & !invalid_advances;
            }

            if pawn_bit_board & bishop_pinned_pieces != 0 {
                if NORTH_EAST_LOOKUP[pawn_index] & own_king != 0 {
                    moves_for_pawn &= NORTH_EAST_LOOKUP[pawn_index]
                } else {
                    moves_for_pawn &= NORTH_WEST_LOOKUP[pawn_index]
                }
            } else if pawn_bit_board & rook_pinned_pieces != 0 {
                if HORIZONTALS_LOOKUP[pawn_index] & own_king != 0 {
                    moves_for_pawn &= HORIZONTALS_LOOKUP[pawn_index]
                } else {
                    moves_for_pawn &= VERTICALSS_LOOKUP[pawn_index]
                }
            }

            moves_for_pawn &= to_mask;

            let promotion_rank = if self.white_to_move { RANK_8 } else { RANK_1 };

            while moves_for_pawn != 0 {
                let to_index = pop_lsb(&mut moves_for_pawn);
                let to_bit_board = square_index_to_bitboard(to_index);
                if to_bit_board & promotion_rank != 0 {
                    self.push_promotion_moves(moves, to_index, pawn_index);
                } else {
                    //TODO: add en passant
                    moves.push(Move::new(pawn_index, to_index));
                }
            }
        }
    }

    #[inline(always)]
    fn push_promotion_moves(&self, moves: &mut MoveList, to_index: usize, from_index: usize) {
        let is_capture = (self.all_pieces) & square_index_to_bitboard(to_index) != 0;

        if is_capture {
            moves.push(Move::capture_promotion(
                PieceKind::Queen,
                from_index,
                to_index,
            ));
            moves.push(Move::capture_promotion(
                PieceKind::Knight,
                from_index,
                to_index,
            ));
            moves.push(Move::capture_promotion(
                PieceKind::Rook,
                from_index,
                to_index,
            ));
            moves.push(Move::capture_promotion(
                PieceKind::Bishop,
                from_index,
                to_index,
            ));
        } else {
            moves.push(Move::promotion(PieceKind::Queen, from_index, to_index));
            moves.push(Move::promotion(PieceKind::Knight, from_index, to_index));
            moves.push(Move::promotion(PieceKind::Rook, from_index, to_index));
            moves.push(Move::promotion(PieceKind::Bishop, from_index, to_index));
        }
    }

    pub fn generate_knight_attack_squares(&self, for_white: bool) -> u64 {
        let mut knights = if for_white {
            self.white_knights
        } else {
            self.black_knights
        };
        let mut knight_attacks = 0u64;
        while knights != 0 {
            let knight_index = pop_lsb(&mut knights);
            knight_attacks |= FREE_KNIGHT_LOOKUP[knight_index];
        }
        knight_attacks
    }

    pub fn generate_knight_moves(
        &self,
        moves: &mut MoveList,
        all_pinned_pieces: u64,
        to_mask: u64,
    ) {
        let mut unpinned_knights = if self.white_to_move {
            self.white_knights
        } else {
            self.black_knights
        } & !all_pinned_pieces;

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        while unpinned_knights != 0 {
            let knight_index = pop_lsb(&mut unpinned_knights) as usize;
            let mut knight_attacks = (FREE_KNIGHT_LOOKUP[knight_index] & !own_pieces) & to_mask;
            while knight_attacks != 0 {
                let to_index = pop_lsb(&mut knight_attacks) as usize;
                moves.push(Move::new(knight_index, to_index))
            }
        }
    }

    /// masks out enemy king, because is used for legal king moves
    pub fn generate_bishop_and_queen_attack_squares(&self, for_white: bool) -> u64 {
        let mut bishops = if for_white {
            self.white_bishops | self.white_queens
        } else {
            self.black_bishops | self.black_queens
        };

        let enemy_king = if for_white {
            self.black_king
        } else {
            self.white_king
        };

        let mut bishop_attacks = 0u64;
        while bishops != 0 {
            let bishop_index = pop_lsb(&mut bishops);
            let bishop_attacks_looked_up =
                get_bishop_moves(bishop_index, self.all_pieces ^ enemy_king);
            bishop_attacks |= bishop_attacks_looked_up
        }
        bishop_attacks
    }

    pub fn generate_bishop_and_queen_moves(
        &self,
        moves: &mut MoveList,
        bishop_pinned_pieces: u64,
        rook_pinned_pieces: u64,
        to_mask: u64,
    ) {
        let mut bishops = if self.white_to_move {
            self.white_bishops | self.white_queens
        } else {
            self.black_bishops | self.black_queens
        };

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        let own_king = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };

        while bishops != 0 {
            let square_index = pop_lsb(&mut bishops) as usize;
            let bit_board = square_index_to_bitboard(square_index as usize);
            if bit_board & rook_pinned_pieces != 0 {
                // cant move along rook pin
                continue;
            }

            let bishop_attacks_looked_up = get_bishop_moves(square_index, self.all_pieces);
            let mut bishop_attacks = bishop_attacks_looked_up & !own_pieces;

            if (bit_board & bishop_pinned_pieces) != 0 {
                if NORTH_EAST_LOOKUP[square_index as usize] & own_king != 0 {
                    bishop_attacks &= NORTH_EAST_LOOKUP[square_index as usize]
                } else if NORTH_WEST_LOOKUP[square_index as usize] & own_king != 0 {
                    bishop_attacks &= NORTH_WEST_LOOKUP[square_index as usize]
                } else {
                    panic!("No overlap with king, even though pinned")
                }
            }

            bishop_attacks &= to_mask;
            while bishop_attacks != 0 {
                let to_index = pop_lsb(&mut bishop_attacks) as usize;
                moves.push(Move::new(square_index, to_index));
            }
        }
    }

    pub fn generate_rook_and_queen_attack_squares(&self, for_white: bool) -> u64 {
        let mut rooks = if for_white {
            self.white_rooks | self.white_queens
        } else {
            self.black_rooks | self.black_queens
        };

        let enemy_king = if for_white {
            self.black_king
        } else {
            self.white_king
        };

        let mut rook_attacks = 0u64;

        while rooks != 0 {
            let rook_index = pop_lsb(&mut rooks) as usize;
            rook_attacks |= get_rook_moves(rook_index as u32, self.all_pieces ^ enemy_king);
        }
        rook_attacks
    }

    pub fn generate_rook_and_queen_moves(
        &self,
        moves: &mut MoveList,
        bishop_pinned_pieces: u64,
        rook_pinned_pieces: u64,
        to_mask: u64,
    ) -> u64 {
        let mut rooks = if self.white_to_move {
            self.white_rooks | self.white_queens
        } else {
            self.black_rooks | self.black_queens
        };

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        let own_king = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };

        let mut all_rook_attacks = 0;

        while rooks != 0 {
            let rook_index = pop_lsb(&mut rooks) as usize;
            let rook_bit_board = square_index_to_bitboard(rook_index);
            if rook_bit_board & bishop_pinned_pieces != 0 {
                continue;
            }
            let rook_attacks_looked_up = get_rook_moves(rook_index as u32, self.all_pieces);
            let mut rook_attacks = rook_attacks_looked_up & !own_pieces;
            if rook_bit_board & rook_pinned_pieces != 0 {
                if HORIZONTALS_LOOKUP[rook_index] & own_king != 0 {
                    rook_attacks &= HORIZONTALS_LOOKUP[rook_index]
                } else {
                    rook_attacks &= VERTICALSS_LOOKUP[rook_index]
                }
            }
            rook_attacks &= to_mask;
            all_rook_attacks |= rook_attacks;
            while rook_attacks != 0 {
                let to_index = pop_lsb(&mut rook_attacks);
                moves.push(Move::new(rook_index, to_index as usize))
            }
        }
        all_rook_attacks
    }

    #[inline(always)]
    pub fn generate_king_attack_squares(&self, for_white: bool) -> u64 {
        let king_square = if for_white {
            self.white_king
        } else {
            self.black_king
        };

        FREE_KING_LOOKUP[bitboard_to_square_index(king_square)]
    }

    pub fn generate_king_moves(
        &self,
        moves: &mut MoveList,
        enemy_attack_square: u64,
        to_mask: u64,
    ) {
        let king_index = if self.white_to_move {
            bitboard_to_square_index(self.white_king)
        } else {
            bitboard_to_square_index(self.black_king)
        };

        let own_pieces = if (self.white_to_move) {
            self.white_pieces
        } else {
            self.black_pieces
        };

        let mut legal_king_moves =
            FREE_KING_LOOKUP[king_index] & (!own_pieces) & (!enemy_attack_square) & to_mask;

        while legal_king_moves != 0 {
            let to_index = pop_lsb(&mut legal_king_moves);

            moves.push(Move::new(king_index, to_index))
        }
    }

    pub fn generate_castling_moves(&self, moves: &mut MoveList, enemy_attack_square: u64) {
        //function assumes king is not in check
        let castling_rights = if self.white_to_move {
            self.white_castling_rights
        } else {
            self.black_castling_rights
        };

        let king_side_mask = if self.white_to_move {
            WHITE_KINGSIDE_CASTLING_MASK
        } else {
            BLACK_KINGSIDE_CASTLING_MASK
        };

        let queen_side_check_mask = if self.white_to_move {
            WHITE_QUEENSIDE_CASTLING_CHECK_MASK
        } else {
            BLACK_QUEENSIDE_CASTLING_CHECK_MASK
        };

        let queen_side_free_squares_mask = if self.white_to_move {
            WHITE_QUEENSIDE_CASTLING_FREE_SQUARES_MASK
        } else {
            BLACK_QUEENSIDE_CASTLING_FREE_SQUARES_MASK
        };

        match castling_rights {
            CastlingRights::All => {
                if (queen_side_check_mask & enemy_attack_square)
                    | (self.all_pieces & queen_side_free_squares_mask)
                    == 0
                {
                    moves.push(Move::castles(false, self.white_to_move));
                }
                if king_side_mask & (enemy_attack_square | self.all_pieces) == 0 {
                    moves.push(Move::castles(true, self.white_to_move));
                }
            }
            CastlingRights::OnlyKingSide => {
                if king_side_mask & (enemy_attack_square | self.all_pieces) == 0 {
                    moves.push(Move::castles(true, self.white_to_move));
                }
            }
            CastlingRights::OnlyQueenSide => {
                if (queen_side_check_mask & enemy_attack_square)
                    | (self.all_pieces & queen_side_free_squares_mask)
                    == 0
                {
                    moves.push(Move::castles(false, self.white_to_move));
                }
            }
            CastlingRights::None => {}
        };
    }

    fn generate_pins_and_sliding_checkers(&self) -> PinAndCheckInfos {
        let mut stop_check_targets = 0u64;

        let kingsquare = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };
        let king_square_index = bitboard_to_square_index(kingsquare);

        let enemy_rooks_squares = if self.white_to_move {
            self.black_rooks
        } else {
            self.white_rooks
        };

        let enemy_bishop_squares = if self.white_to_move {
            self.black_bishops
        } else {
            self.white_bishops
        };

        let enemy_queens_squares = if self.white_to_move {
            self.black_queens
        } else {
            self.white_queens
        };

        let mut potential_rook_piners = FREE_ROOK_LOOKUP[bitboard_to_square_index(kingsquare)]
            & (enemy_queens_squares | enemy_rooks_squares);

        let mut rook_pinned_pieces = 0u64;
        let mut sliding_checkers = 0u64;

        while potential_rook_piners != 0 {
            let rook_or_queen_square_index = pop_lsb(&mut potential_rook_piners);
            let ray = ROOK_SQUARE_TO_SQUARE_RAY_LOOKUP
                [rook_or_queen_square_index * 64 + king_square_index];
            // we need to check for all pieces, so no enemy pieces block the pin
            // this is later masked out to 0 with the "my_pieces & ray" instruction
            let pieces_between = ray & self.all_pieces;
            match pieces_between.count_ones() {
                0 => {
                    //Check detected
                    sliding_checkers |= square_index_to_bitboard(rook_or_queen_square_index);
                    stop_check_targets |= ray | sliding_checkers;
                }
                1 => {
                    // let my_pieces = if self.white_to_move {
                    //     self.white_pieces
                    // } else {
                    //     self.black_pieces
                    // };
                    //TODO: maybe also divide in 2 bitboards (now discoverers and pins together)
                    let pins = self.all_pieces & ray;
                    rook_pinned_pieces |= pins;
                }
                // for detecting illegal en passant captures, see:
                // 8/6k1/8/K1Pp1r2/8/8/8/8 w - - 0 1
                2 => 'match_arm: {
                    // only care for same rank
                    if ((rook_or_queen_square_index as i32) - (king_square_index as i32)).abs() >= 8
                    {
                        break 'match_arm;
                    }
                    let own_pawns = if self.white_to_move {
                        self.white_pawns
                    } else {
                        self.black_pawns
                    };
                    let maybe_own_pawn = own_pawns & ray;
                    if maybe_own_pawn.count_ones() != 1 {
                        break 'match_arm;
                    }
                    let pawn_attacks = if self.white_to_move {
                        WHITE_FREE_PAWN_ATTACKS_LOOKUP
                    } else {
                        BLACK_FREE_PAWN_ATTACKS_LOOKUP
                    };
                    rook_pinned_pieces |= (pawn_attacks[bitboard_to_square_index(maybe_own_pawn)]
                        & self.en_passant_target)
                }
                _ => {}
            }
        }

        let mut potential_bishop_piners = FREE_BISHOP_LOOKUP[bitboard_to_square_index(kingsquare)]
            & (enemy_queens_squares | enemy_bishop_squares);

        let mut bishop_pinned_pieces = 0u64;
        while potential_bishop_piners != 0 {
            let bishop_or_queen_square_index = pop_lsb(&mut potential_bishop_piners);
            let ray = BISHOP_SQUARE_TO_SQUARE_RAY_LOOKUP
                [bishop_or_queen_square_index * 64 + king_square_index];
            // we need to check for all pieces, so no enemy pieces block the pin
            // this is later masked out to 0 with the "my_pieces & ray" instruction
            let pieces_between = ray & self.all_pieces;
            if pieces_between.count_ones() == 0 {
                //Check detected
                sliding_checkers |= square_index_to_bitboard(bishop_or_queen_square_index);
                stop_check_targets |= ray | sliding_checkers;
            } else if pieces_between.count_ones() == 1 {
                let my_pieces = if self.white_to_move {
                    self.white_pieces
                } else {
                    self.black_pieces
                };
                //TODO: maybe also divide in 2 bitboards (now discoverers and pins together)
                let pins = self.all_pieces & ray;
                bishop_pinned_pieces |= pins;
            }
        }
        //todo!()
        //TODO: move in struct
        PinAndCheckInfos {
            sliding_checkers,
            stop_check_targets,
            bishop_pinned_pieces,
            rook_pinned_pieces,
        }
    }

    pub fn generate_legal_moves_temp(&self) -> MoveList {
        let mut moves = MoveList::default();

        let pin_and_check_infos = self.generate_pins_and_sliding_checkers();
        let all_pinned_pieces =
            pin_and_check_infos.bishop_pinned_pieces | pin_and_check_infos.rook_pinned_pieces;
        let checkers =
            pin_and_check_infos.sliding_checkers | self.get_enemy_pawn_and_knight_checkers();

        let enemy_pawn_attacks = self.gen_pawn_attack_squares(!self.white_to_move);
        let enemy_knight_attacks = self.generate_knight_attack_squares(!self.white_to_move);
        let enemy_bishop_and_queen_attacks =
            self.generate_bishop_and_queen_attack_squares(!self.white_to_move);
        let enemy_rook_and_queen_attacks =
            self.generate_rook_and_queen_attack_squares(!self.white_to_move);
        let enemy_king_attacks = self.generate_king_attack_squares(!self.white_to_move);
        let all_enemy_attack_squares = enemy_pawn_attacks
            | enemy_knight_attacks
            | enemy_bishop_and_queen_attacks
            | enemy_rook_and_queen_attacks
            | enemy_king_attacks;

        match checkers.count_ones() {
            //No checks
            0 => {
                self.generate_pawn_moves(
                    &mut moves,
                    pin_and_check_infos.rook_pinned_pieces,
                    pin_and_check_infos.bishop_pinned_pieces,
                    u64::MAX,
                );
                self.generate_knight_moves(&mut moves, all_pinned_pieces, u64::MAX);
                self.generate_bishop_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    u64::MAX,
                );
                self.generate_rook_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    u64::MAX,
                );
                self.generate_king_moves(&mut moves, all_enemy_attack_squares, u64::MAX);
                self.generate_castling_moves(&mut moves, all_enemy_attack_squares);
            }
            //One check, king and blocking moves or capturing checker only
            1 => {
                let stop_check_mask = checkers | pin_and_check_infos.stop_check_targets;
                self.generate_pawn_moves(
                    &mut moves,
                    pin_and_check_infos.rook_pinned_pieces,
                    pin_and_check_infos.bishop_pinned_pieces,
                    stop_check_mask,
                );
                self.generate_knight_moves(&mut moves, all_pinned_pieces, stop_check_mask);
                self.generate_bishop_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    stop_check_mask,
                );
                self.generate_rook_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    stop_check_mask,
                );
                // King should move out of the way and not to the check
                self.generate_king_moves(&mut moves, all_enemy_attack_squares, u64::MAX);
            }
            //2 checks, only king moves possible
            2 => {
                self.generate_king_moves(&mut moves, all_enemy_attack_squares, u64::MAX);
            }
            _ => panic!(
                "There cant be more than 2 checkers, but there are{}",
                checkers.count_ones()
            ),
        }

        moves
    }
}

impl Default for Board {
    fn default() -> Self {
        // Initialize the board to the starting position
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

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
    pub fn new(from: usize, to: usize) -> Self {
        let mask = from as u16 | (to as u16) << Self::DST_BITS | Self::FLAG_QUIET;
        Self { mask }
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
    pub fn capture(from: usize, to: usize) -> Self {
        Self {
            mask: from as u16 | (to as u16) << Self::DST_BITS | Self::FLAG_CAPTURE,
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
        flag >= Self::FLAG_PROMO_KNIGHT && flag <= Self::FLAG_CAPTURE_PROMO_QUEEN
    }

    #[inline(always)]
    pub fn is_capture(&self) -> bool {
        let flag = self.flag();
        flag == Self::FLAG_CAPTURE
            || flag == Self::FLAG_EP_CAPTURE
            || (flag >= Self::FLAG_CAPTURE_PROMO_KNIGHT && flag <= Self::FLAG_CAPTURE_PROMO_QUEEN)
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

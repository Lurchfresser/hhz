use crate::bit_boards::*;
use regex::Regex;
use std::fmt;
use std::fmt::{Display, Formatter};

pub const MAX_NUM_MOVES: usize = 218;
pub type MoveList = arrayvec::ArrayVec<Move, MAX_NUM_MOVES>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingRights {
    All,
    OnlyKingSide,
    OnlyQueenSide,
    None,
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
                    black_pawns |= 1 << square - 1;
                    file += 1;
                }
                'r' => {
                    black_rooks |= 1 << square - 1;
                    file += 1;
                }
                'n' => {
                    black_knights |= 1 << square - 1;
                    file += 1;
                }
                'b' => {
                    black_bishops |= 1 << square - 1;
                    file += 1;
                }
                'q' => {
                    black_queens |= 1 << square - 1;
                    file += 1;
                }
                'k' => {
                    black_king |= 1 << square - 1;
                    file += 1;
                }
                'P' => {
                    white_pawns |= 1 << square - 1;
                    file += 1;
                }
                'R' => {
                    white_rooks |= 1 << square - 1;
                    file += 1;
                }
                'N' => {
                    white_knights |= 1 << square - 1;
                    file += 1;
                }
                'B' => {
                    white_bishops |= 1 << square - 1;
                    file += 1;
                }
                'Q' => {
                    white_queens |= 1 << square - 1;
                    file += 1;
                }
                'K' => {
                    white_king |= 1 << square - 1;
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

    pub fn generate_pawn_moves(
        &self,
        moves: &MoveList,
        rook_pinned_pieces: u64,
        bishop_pinned_pieces: u64,
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
            let square_index = pop_lsb(&mut pawns) as usize;
            let pawn_bit_board = square_index_to_bitboard(square_index);

            // 1. Generate attacks
            moves_for_pawn |=
                ATTACKS_LOOKUP[square_index] & (enemy_pieces | self.en_passant_target);

            // 2. Generate advances
            let blockers = self.all_pieces ^ pawn_bit_board;
            let advances = ADVANCE_LOOKUP[square_index];

            if self.white_to_move {
                // For white, shift blockers UP to see if they block the double advance
                let invalid_advances = blockers | (blockers << 8);
                moves_for_pawn |= advances & !invalid_advances;
            } else {
                // For black, shift blockers DOWN to see if they block the double advance
                let invalid_advances = blockers | (blockers >> 8);
                moves_for_pawn |= advances & !invalid_advances;
            }
        }
    }

    pub fn generate_knight_moves(&self, moves: &mut MoveList, all_pinned_pieces: u64) {
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
            let mut knight_attacks = FREE_KNIGHT_LOOKUP[knight_index] & !own_pieces;
            while knight_attacks != 0 {
                let to_index = pop_lsb(&mut knight_attacks) as usize;
                moves.push(Move::new(knight_index, to_index))
            }
        }
    }

    pub fn generate_bishop_moves(
        &self,
        moves: &mut MoveList,
        bishop_pinned_pieces: u64,
        rook_pinned_pieces: u64,
    ) {
        let mut bishops = if self.white_to_move {
            self.white_bishops
        } else {
            self.black_bishops
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

            let bishop_attacks_looked_up = get_bishop_moves(square_index as u32, self.all_pieces);
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

            while bishop_attacks != 0 {
                let to_index = pop_lsb(&mut bishop_attacks) as usize;
                moves.push(Move::new(square_index, to_index));
            }
        }
    }

    pub fn generate_rook_moves(&self) {
        let mut rooks = if self.white_to_move {
            self.white_rooks
        } else {
            self.black_rooks
        };

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        while rooks != 0 {
            let rook_attacks_looked_up =
                get_rook_moves(pop_lsb(&mut rooks).try_into().unwrap(), self.all_pieces);
            let rook_attacks = rook_attacks_looked_up & !own_pieces;
            println!("rook attacks {}", rook_attacks);
        }
    }

    pub fn generate_king_moves<const IN_CHECK: bool>(&self, moves: MoveList) -> u64 {
        let king_moves = if self.white_to_move {
            FREE_KING_LOOKUP[self.white_king.trailing_zeros() as usize] & !self.white_pieces
        } else {
            FREE_KING_LOOKUP[self.black_king.trailing_zeros() as usize] & !self.black_pieces
        };

        if IN_CHECK { king_moves } else { king_moves }
    }

    pub fn generate_pins_and_sliding_checkers(&self) -> (u64, u64, u64) {
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
                [(rook_or_queen_square_index as usize) * 64 + king_square_index];
            // we need to check for all pieces, so no enemy pieces block the pin
            // this is later masked out to 0 with the "my_pieces & ray" instruction
            let pieces_between = ray & self.all_pieces;
            if pieces_between.count_ones() == 0 {
                //Check detected
                sliding_checkers |= square_index_to_bitboard(rook_or_queen_square_index as usize);
            } else if pieces_between.count_ones() == 1 {
                let my_pieces = if self.white_to_move {
                    self.white_pieces
                } else {
                    self.black_pieces
                };
                let pins = my_pieces & ray;
                rook_pinned_pieces |= pins;
                println!("rook pins: {}", pins);
            }
        }

        let mut potential_bishop_piners = FREE_BISHOP_LOOKUP[bitboard_to_square_index(kingsquare)]
            & (enemy_queens_squares | enemy_bishop_squares);

        let mut bishop_pinned_pieces = 064;
        while potential_bishop_piners != 0 {
            let bishop_or_queen_square_index = pop_lsb(&mut potential_bishop_piners);
            let ray = BISHOP_SQUARE_TO_SQUARE_RAY_LOOKUP
                [(bishop_or_queen_square_index as usize) * 64 + king_square_index];
            // we need to check for all pieces, so no enemy pieces block the pin
            // this is later masked out to 0 with the "my_pieces & ray" instruction
            let pieces_between = ray & self.all_pieces;
            if pieces_between.count_ones() == 0 {
                //Check detected
                sliding_checkers |= square_index_to_bitboard(bishop_or_queen_square_index as usize);
            } else if pieces_between.count_ones() == 1 {
                let my_pieces = if self.white_to_move {
                    self.white_pieces
                } else {
                    self.black_pieces
                };
                let pins = my_pieces & ray;
                bishop_pinned_pieces |= pins;
                println!("bishop pins: {}", pins);
            }
        }
        //todo!()
        //TODO: move in struct
        (sliding_checkers, bishop_pinned_pieces, rook_pinned_pieces)
    }

    pub fn generate_legal_moves_temp(&self) -> MoveList {
        let mut moves = MoveList::default();

        let (sliding_checkers, bishop_pinned_pieces, rook_pinned_pieces) =
            self.generate_pins_and_sliding_checkers();
        let all_pinned_pieces = bishop_pinned_pieces | rook_pinned_pieces;
        let checkers = sliding_checkers | self.get_enemy_pawn_and_knight_checkers();

        match checkers.count_ones() {
            //No checks
            0 => {
                // self.generate_pawn_moves(&moves, rook_pinned_pieces, bishop_pinned_pieces);
                self.generate_knight_moves(&mut moves, all_pinned_pieces);
                self.generate_bishop_moves(&mut moves, bishop_pinned_pieces, rook_pinned_pieces);
            }
            //One check, king and blocking moves only
            1 => {}
            //2 checks, only king moves possible
            2 => {
                // self.generate_king_moves::<true>();
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
    const FROM_MASK: u16 = 0b0000_0000_0011_1111;
    /// Mask for the destination ("to") bits.
    const TO_MASK: u16 = 0b0000_1111_1100_0000;
    const CAPTURE_MASK: u16 = 0b0001_000000_000000;

    const TO_BITS: u16 = 6;
    const FLAG_BITS: u16 = 12;
    #[inline(always)]
    pub fn new(from: usize, to: usize) -> Self {
        let mask = from | to << Self::TO_BITS;
        Self { mask: mask as u16 }
    }

    #[inline(always)]
    pub fn from(&self) -> usize {
        (self.mask & Self::FROM_MASK) as usize
    }

    #[inline(always)]
    pub fn to(&self) -> usize {
        ((self.mask & Self::TO_MASK) >> Self::TO_BITS) as usize
    }

    pub fn to_algebraic(&self) -> String {
        //TODO: Promotion and castling
        let from_square = square_index_to_square(self.from());
        let to_square = square_index_to_square(self.to());
        format!(
            "{}{}",
            square_to_algebraic(from_square),
            square_to_algebraic(to_square)
        )
    }
}

pub fn square_to_algebraic(square: Square) -> String {
    format!("{}{}", ((square.file + 97) as u8) as char, square.rank + 1)
}

impl fmt::Debug for Move {
    /// Debug formatting will call the [`fmt::Display`] implementation
    /// (taking into account the alternate formatter, if provided)
    /// and will also display it's [`MoveKind`] in a human-readable format.
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

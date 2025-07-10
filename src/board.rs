use crate::polyglot_zobrists::*;
use crate::{bit_boards::*, moves::square_to_algebraic};
use regex::Regex;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingRights {
    All,
    OnlyKingSide,
    OnlyQueenSide,
    None,
}

impl CastlingRights {
    pub fn remove_side(&self, castling_rights: CastlingRights) -> Self {
        match self {
            CastlingRights::All => match castling_rights {
                CastlingRights::All => CastlingRights::None,
                CastlingRights::OnlyKingSide => CastlingRights::OnlyQueenSide,
                CastlingRights::OnlyQueenSide => Self::OnlyKingSide,
                CastlingRights::None => CastlingRights::All,
            },
            CastlingRights::OnlyKingSide => match castling_rights {
                CastlingRights::All => CastlingRights::None,
                CastlingRights::OnlyKingSide => CastlingRights::None,
                CastlingRights::OnlyQueenSide => CastlingRights::OnlyKingSide,
                CastlingRights::None => CastlingRights::OnlyKingSide,
            },
            CastlingRights::OnlyQueenSide => match castling_rights {
                CastlingRights::All => CastlingRights::None,
                CastlingRights::OnlyKingSide => CastlingRights::OnlyQueenSide,
                CastlingRights::OnlyQueenSide => CastlingRights::None,
                CastlingRights::None => CastlingRights::OnlyQueenSide,
            },
            CastlingRights::None => CastlingRights::None,
        }
    }

    pub fn zobrist_hash(&self, for_white: bool) -> u64 {
        if for_white {
            match self {
                CastlingRights::All => ZOBRISTS_CASTLING_RIGHTS[0] ^ ZOBRISTS_CASTLING_RIGHTS[1],
                CastlingRights::OnlyKingSide => ZOBRISTS_CASTLING_RIGHTS[0],
                CastlingRights::OnlyQueenSide => ZOBRISTS_CASTLING_RIGHTS[1],
                CastlingRights::None => 0,
            }
        } else {
            match self {
                CastlingRights::All => ZOBRISTS_CASTLING_RIGHTS[2] ^ ZOBRISTS_CASTLING_RIGHTS[3],
                CastlingRights::OnlyKingSide => ZOBRISTS_CASTLING_RIGHTS[2],
                CastlingRights::OnlyQueenSide => ZOBRISTS_CASTLING_RIGHTS[3],
                CastlingRights::None => 0,
            }
        }
    }
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
pub enum Piece {
    None,
    Pawn { white: bool },
    Knight { white: bool },
    Bishop { white: bool },
    Rook { white: bool },
    Queen { white: bool },
    King { white: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board {
    pub white_pawns: u64,
    pub white_knights: u64,
    pub white_bishops: u64,
    pub white_rooks: u64,
    pub white_queens: u64,
    pub white_king: u64,

    pub white_pieces: u64,

    pub black_pawns: u64,
    pub black_knights: u64,
    pub black_bishops: u64,
    pub black_rooks: u64,
    pub black_queens: u64,
    pub black_king: u64,

    pub black_pieces: u64,

    pub all_pieces: u64,

    pub en_passant_target: u64,

    pub white_castling_rights: CastlingRights,
    pub black_castling_rights: CastlingRights,

    pub halfmove_clock: u8,
    pub full_move_number: u16,

    pub white_to_move: bool,

    pub pieces: [Piece; 64],

    pub repetition_lookup: [u64; 100],

    pub zobrist_hash: u64,
}

#[derive(Debug, Clone)]
pub enum FenError {
    InvalidCharacter(char),
    InvalidSideToMove(String),
    InvalidCastlingRights(String),
    MissingParts,
    InvalidEnPassant(String),
    InvalidHalfmoveClock(String),
    InvalidFullmoveNumber(String),
    InvalidRank,
    InvalidFile,
    InvalidNumericParse(String),
}

impl std::fmt::Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenError::InvalidCharacter(c) => write!(f, "Invalid FEN character: {}", c),
            FenError::InvalidSideToMove(s) => write!(f, "Invalid side to move in FEN: {}", s),
            FenError::InvalidCastlingRights(s) => {
                write!(f, "Invalid castling rights in FEN: {}", s)
            }
            FenError::MissingParts => write!(f, "FEN string is missing required parts"),
            FenError::InvalidEnPassant(s) => write!(f, "Invalid en passant square: {}", s),
            FenError::InvalidHalfmoveClock(s) => write!(f, "Invalid halfmove clock: {}", s),
            FenError::InvalidFullmoveNumber(s) => write!(f, "Invalid fullmove number: {}", s),
            FenError::InvalidRank => write!(f, "Invalid rank in FEN"),
            FenError::InvalidFile => write!(f, "Invalid file in FEN"),
            FenError::InvalidNumericParse(s) => write!(f, "Could not parse numeric value: {}", s),
        }
    }
}

impl std::error::Error for FenError {}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
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

        let mut pieces = [Piece::None; 64];

        let mut file: i32 = 0; // Changed to 0-based indexing
        let mut rank: i32 = 7; // Changed to 0-based indexing, top rank is 7

        let mut zobrist_hash = 0u64;

        // Parse piece placement
        for c in fen.chars() {
            if rank < 0 || rank > 7 {
                // Updated range check
                return Err(FenError::InvalidRank);
            }
            if file < 0 || file > 8 {
                // Updated range check
                return Err(FenError::InvalidFile);
            }

            let square_index = (rank * 8 + file) as usize;
            let bit_position = square_index;

            match c {
                '1'..='8' => {
                    let skip_count = c.to_digit(10).unwrap() as i32;
                    file += skip_count;
                }
                'p' => {
                    black_pawns |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Pawn { white: false };
                    zobrist_hash ^= ZOBRISTS_BLACK_PAWNS[square_index];
                    file += 1;
                }
                'r' => {
                    black_rooks |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Rook { white: false };
                    zobrist_hash ^= ZOBRISTS_BLACK_ROOKS[square_index];
                    file += 1;
                }
                'n' => {
                    black_knights |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Knight { white: false };
                    zobrist_hash ^= ZOBRISTS_BLACK_KNIGHTS[square_index];
                    file += 1;
                }
                'b' => {
                    black_bishops |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Bishop { white: false };
                    zobrist_hash ^= ZOBRISTS_BLACK_BISHOPS[square_index];
                    file += 1;
                }
                'q' => {
                    black_queens |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Queen { white: false };
                    zobrist_hash ^= ZOBRISTS_BLACK_QUEENS[square_index];
                    file += 1;
                }
                'k' => {
                    black_king |= 1u64 << bit_position;
                    pieces[square_index] = Piece::King { white: false };
                    zobrist_hash ^= ZOBRISTS_BLACK_KINGS[square_index];
                    file += 1;
                }
                'P' => {
                    white_pawns |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Pawn { white: true };
                    zobrist_hash ^= ZOBRISTS_WHITE_PAWNS[square_index];
                    file += 1;
                }
                'R' => {
                    white_rooks |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Rook { white: true };
                    zobrist_hash ^= ZOBRISTS_WHITE_ROOKS[square_index];
                    file += 1;
                }
                'N' => {
                    white_knights |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Knight { white: true };
                    zobrist_hash ^= ZOBRISTS_WHITE_KNIGHTS[square_index];
                    file += 1;
                }
                'B' => {
                    white_bishops |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Bishop { white: true };
                    zobrist_hash ^= ZOBRISTS_WHITE_BISHOPS[square_index];
                    file += 1;
                }
                'Q' => {
                    white_queens |= 1u64 << bit_position;
                    pieces[square_index] = Piece::Queen { white: true };
                    zobrist_hash ^= ZOBRISTS_WHITE_QUEENS[square_index];
                    file += 1;
                }
                'K' => {
                    white_king |= 1u64 << bit_position;
                    pieces[square_index] = Piece::King { white: true };
                    zobrist_hash ^= ZOBRISTS_WHITE_KINGS[square_index];
                    file += 1;
                }
                '/' => {
                    if file != 8 {
                        // Updated check
                        return Err(FenError::InvalidFile);
                    }
                    file = 0; // Reset to start of next rank
                    rank -= 1; // Move down one rank
                }
                ' ' => break,
                _ => return Err(FenError::InvalidCharacter(c)),
            }
        }

        let white_pieces =
            white_pawns | white_knights | white_bishops | white_rooks | white_queens | white_king;
        let black_pieces =
            black_pawns | black_knights | black_bishops | black_rooks | black_queens | black_king;
        let all_pieces = white_pieces | black_pieces;

        // Parse the rest of the FEN string
        let mut parts = fen.split_whitespace();
        parts.next(); // Skip the piece placement part

        // Parse active color
        let side_move_str = parts.next().ok_or(FenError::MissingParts)?;
        let white_to_move = match side_move_str {
            "w" => true,
            "b" => false,
            _ => return Err(FenError::InvalidSideToMove(side_move_str.to_string())),
        };
        if white_to_move {
            zobrist_hash ^= ZOBRISTS_WHITE_TO_MOVE;
        }

        // Parse castling rights
        let castling_rights_str = parts.next().ok_or(FenError::MissingParts)?;

        // Validate castling rights format
        if !Regex::new(r"^(-|K?Q?k?q?)$")
            .unwrap()
            .is_match(castling_rights_str)
        {
            return Err(FenError::InvalidCastlingRights(
                castling_rights_str.to_string(),
            ));
        }

        let white_castling_rights =
            if castling_rights_str.contains('K') && castling_rights_str.contains('Q') {
                CastlingRights::All
            } else if castling_rights_str.contains('Q') {
                CastlingRights::OnlyQueenSide
            } else if castling_rights_str.contains('K') {
                CastlingRights::OnlyKingSide
            } else {
                CastlingRights::None
            };

        let black_castling_rights =
            if castling_rights_str.contains('k') && castling_rights_str.contains('q') {
                CastlingRights::All
            } else if castling_rights_str.contains('q') {
                CastlingRights::OnlyQueenSide
            } else if castling_rights_str.contains('k') {
                CastlingRights::OnlyKingSide
            } else {
                CastlingRights::None
            };

        zobrist_hash ^=
            white_castling_rights.zobrist_hash(true) ^ black_castling_rights.zobrist_hash(false);

        // Parse en passant target
        let en_passant_str = parts.next().ok_or(FenError::MissingParts)?;
        let en_passant_target = if en_passant_str == "-" {
            0u64
        } else {
            if en_passant_str.len() != 2 {
                return Err(FenError::InvalidEnPassant(en_passant_str.to_string()));
            }
            let file_char = en_passant_str.chars().next().unwrap();
            let rank_char = en_passant_str.chars().nth(1).unwrap();

            if !('a'..='h').contains(&file_char) || !('1'..='8').contains(&rank_char) {
                return Err(FenError::InvalidEnPassant(en_passant_str.to_string()));
            }

            let file = file_char as u8 - b'a';
            let rank = rank_char as u8 - b'1';
            let ep_square_index = (rank * 8 + file) as usize;

            // Polyglot spec: hash only if a pawn can actually perform the capture.
            let can_capture_en_passant = if white_to_move {
                // White to move, EP target must be on rank 6. Black pawn must be on rank 5.
                // Check for white pawns on adjacent files on rank 5.
                let pawn_rank_mask = 1u64 << (ep_square_index - 8);
                let adjacent_files_mask = (pawn_rank_mask.wrapping_shl(1) & !FILE_H)
                    | (pawn_rank_mask.wrapping_shr(1) & !FILE_A);
                (white_pawns & adjacent_files_mask) != 0
            } else {
                // Black to move, EP target must be on rank 3. White pawn must be on rank 4.
                // Check for black pawns on adjacent files on rank 4.
                let pawn_rank_mask = 1u64 << (ep_square_index + 8);
                let adjacent_files_mask = (pawn_rank_mask.wrapping_shl(1) & !FILE_H)
                    | (pawn_rank_mask.wrapping_shr(1) & !FILE_A);
                (black_pawns & adjacent_files_mask) != 0
            };

            if can_capture_en_passant {
                zobrist_hash ^= ZOBRISTS_EN_PASSANT_FILE[file as usize];
                1u64 << ep_square_index
            } else {
                0
            }
        };

        // Parse halfmove clock
        let halfmove_str = parts.next().ok_or(FenError::MissingParts)?;
        let halfmove_clock = halfmove_str
            .parse::<u8>()
            .map_err(|_| FenError::InvalidHalfmoveClock(halfmove_str.to_string()))?;

        // Parse fullmove number
        let fullmove_str = parts.next().ok_or(FenError::MissingParts)?;
        let fullmove_number = fullmove_str
            .parse::<u16>()
            .map_err(|_| FenError::InvalidFullmoveNumber(fullmove_str.to_string()))?;

        Ok(Board {
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
            full_move_number: fullmove_number,
            pieces,
            zobrist_hash,
            repetition_lookup: [0; 100],
        })
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(90);

        // 1. Piece placement
        for rank in (0..8).rev() {
            let mut empty_squares = 0;
            for file in 0..8 {
                let index = rank * 8 + file;
                let piece = self.pieces[index];

                if piece == Piece::None {
                    empty_squares += 1;
                } else {
                    if empty_squares > 0 {
                        fen.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }
                    let piece_char = match piece {
                        Piece::Pawn { white: true } => 'P',
                        Piece::Knight { white: true } => 'N',
                        Piece::Bishop { white: true } => 'B',
                        Piece::Rook { white: true } => 'R',
                        Piece::Queen { white: true } => 'Q',
                        Piece::King { white: true } => 'K',
                        Piece::Pawn { white: false } => 'p',
                        Piece::Knight { white: false } => 'n',
                        Piece::Bishop { white: false } => 'b',
                        Piece::Rook { white: false } => 'r',
                        Piece::Queen { white: false } => 'q',
                        Piece::King { white: false } => 'k',
                        Piece::None => unreachable!(),
                    };
                    fen.push(piece_char);
                }
            }
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // 2. Active color
        fen.push(' ');
        fen.push(if self.white_to_move { 'w' } else { 'b' });

        // 3. Castling availability
        fen.push(' ');
        let mut castling_str = String::new();
        match self.white_castling_rights {
            CastlingRights::All => castling_str.push_str("KQ"),
            CastlingRights::OnlyKingSide => castling_str.push('K'),
            CastlingRights::OnlyQueenSide => castling_str.push('Q'),
            CastlingRights::None => {}
        }
        match self.black_castling_rights {
            CastlingRights::All => castling_str.push_str("kq"),
            CastlingRights::OnlyKingSide => castling_str.push('k'),
            CastlingRights::OnlyQueenSide => castling_str.push('q'),
            CastlingRights::None => {}
        }
        if castling_str.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castling_str);
        }

        let lookup = if self.white_to_move {
            &BLACK_FREE_PAWN_ADVANCE_LOOKUP
        } else {
            &WHITE_FREE_PAWN_ATTACKS_LOOKUP
        };
        let enemypawns = if self.white_to_move {
            self.black_pawns
        } else {
            self.white_pawns
        };
        // 4. En passant target square
        fen.push(' ');
        if self.en_passant_target != 0
            && lookup[bitboard_to_square_index(self.en_passant_target)] & enemypawns != 0
        {
            let ep_index = bitboard_to_square_index(self.en_passant_target);
            let ep_square = square_index_to_square(ep_index);
            fen.push_str(&square_to_algebraic(ep_square));
        } else {
            fen.push('-');
        }

        // 5. Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        // 6. Fullmove number
        fen.push(' ');
        fen.push_str(&self.full_move_number.to_string());

        fen
    }
}

impl Default for Board {
    fn default() -> Self {
        // Initialize the board to the starting position
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        match board {
            Ok(b) => b,
            Err(e) => panic!("{}", e),
        }
    }
}

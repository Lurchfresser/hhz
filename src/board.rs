use crate::bit_boards::*;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingRights {
    All,
    OnlyKingSide,
    OnlyQueenSide,
    None,
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

    en_passant_target: Option<u64>,

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

        let mut white_to_move = true;

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

        let white_pieces = white_pawns | white_knights | white_bishops | white_rooks | white_queens | white_king;
        let black_pieces = black_pawns | black_knights | black_bishops | black_rooks | black_queens | black_king;
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

        let en_passant_target = parts.next().and_then(|s| {
            if s == "-" {
                None
            } else {
                // Convert the square to a bitboard position
                let file = s.chars().next().unwrap() as u8 - b'a';
                let rank = s.chars().nth(1).unwrap() as u8 - b'1';
                Some(1 << (rank * 8 + file))
            }
        });

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
            let rook_attacks =
                get_rook_moves(pop_lsb(&mut rooks).try_into().unwrap(), self.all_pieces) & !own_pieces;
            println!("rook attacks {}", rook_attacks);
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        // Initialize the board to the starting position
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

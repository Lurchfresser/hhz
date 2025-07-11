use crate::board::{Board, Piece};
use crate::metrics::{SearchMetrics, TimingKind};

pub const PAWN_SCORE: i16 = 100;
pub const KNIGHT_SCORE: i16 = 300;
pub const BISHOP_SCORE: i16 = 320;
pub const ROOK_SCORE: i16 = 500;
pub const QUEEN_SCORE: i16 = 900;

pub fn pieces_score(piece: Piece) -> i16 {
    match piece {
        Piece::None => 0,
        Piece::Pawn { white } => {
            if white {
                PAWN_SCORE
            } else {
                -PAWN_SCORE
            }
        }
        Piece::Knight { white } => {
            if white {
                KNIGHT_SCORE
            } else {
                -KNIGHT_SCORE
            }
        }
        Piece::Bishop { white } => {
            if white {
                BISHOP_SCORE
            } else {
                -BISHOP_SCORE
            }
        }
        Piece::Rook { white } => {
            if white {
                ROOK_SCORE
            } else {
                -ROOK_SCORE
            }
        }
        Piece::Queen { white } => {
            if white {
                QUEEN_SCORE
            } else {
                -QUEEN_SCORE
            }
        }
        Piece::King { .. } => 0, // King is not scored in this evaluation
    }
}

pub fn eval(board: &Board) -> i16 {
    SearchMetrics::change_timing_kind(TimingKind::Evaluation);
    let score = board.score();
    let mobility_score: i16 = board.gen_pawn_attack_squares(true).count_ones() as i16
        - board.gen_pawn_attack_squares(false).count_ones() as i16
        + board.generate_knight_attack_squares(true).count_ones() as i16
        - board.generate_knight_attack_squares(false).count_ones() as i16
        + board
            .generate_bishop_and_queen_attack_squares(true)
            .count_ones() as i16
        - board
            .generate_bishop_and_queen_attack_squares(false)
            .count_ones() as i16
        + board
            .generate_rook_and_queen_attack_squares(true)
            .count_ones() as i16
        - board
            .generate_rook_and_queen_attack_squares(false)
            .count_ones() as i16;
    score + (mobility_score as i16)
}

trait PiecesScore {
    fn score(&self) -> i16;
}

impl PiecesScore for Board {
    fn score(&self) -> i16 {
        let mut score: i16 = 0;
        score += (self.white_pawns.count_ones() as i16) * PAWN_SCORE;
        score += (self.white_knights.count_ones() as i16) * KNIGHT_SCORE;
        score += (self.white_bishops.count_ones() as i16) * BISHOP_SCORE;
        score += (self.white_rooks.count_ones() as i16) * ROOK_SCORE;
        score += (self.white_queens.count_ones() as i16) * QUEEN_SCORE;
        score -= (self.black_pawns.count_ones() as i16) * PAWN_SCORE;
        score -= (self.black_knights.count_ones() as i16) * KNIGHT_SCORE;
        score -= (self.black_bishops.count_ones() as i16) * BISHOP_SCORE;
        score -= (self.black_rooks.count_ones() as i16) * ROOK_SCORE;
        score -= (self.black_queens.count_ones() as i16) * QUEEN_SCORE;
        score
    }
}

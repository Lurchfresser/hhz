use crate::board::{Board, Piece, PieceKind};
use crate::metrics::{SearchMetrics, TimingKind};

pub const PAWN_SCORE: i32 = 100;
pub const KNIGHT_SCORE: i32 = 300;
pub const BISHOP_SCORE: i32 = 320;
pub const ROOK_SCORE: i32 = 500;
pub const QUEEN_SCORE: i32 = 900;

pub fn pieces_score(piece: Piece) -> i32 {
    match piece {
        Piece::None => 0,
        Piece::Pawn { white } => if white { PAWN_SCORE } else { -PAWN_SCORE },
        Piece::Knight { white } => if white { KNIGHT_SCORE } else { -KNIGHT_SCORE },
        Piece::Bishop { white } => if white { BISHOP_SCORE } else { -BISHOP_SCORE },
        Piece::Rook { white } => if white { ROOK_SCORE } else { -ROOK_SCORE },
        Piece::Queen { white } => if white { QUEEN_SCORE } else { -QUEEN_SCORE },
        Piece::King { .. } => 0, // King is not scored in this evaluation
    }
}

pub fn eval(board: &Board) -> i32 {
    SearchMetrics::change_timing_kind(TimingKind::Evaluation);
    let score = board.score();
    score
}

trait PiecesScore {
    fn score(&self) -> i32;
}

impl PiecesScore for Board {
    fn score(&self) -> i32 {
        let mut score: i32 = 0;
        score += (self.white_pawns.count_ones() as i32) * PAWN_SCORE;
        score += (self.white_knights.count_ones() as i32) * KNIGHT_SCORE;
        score += (self.white_bishops.count_ones() as i32) * BISHOP_SCORE;
        score += (self.white_rooks.count_ones() as i32) * ROOK_SCORE;
        score += (self.white_queens.count_ones() as i32) * QUEEN_SCORE;
        score -= (self.black_pawns.count_ones() as i32) * PAWN_SCORE;
        score -= (self.black_knights.count_ones() as i32) * KNIGHT_SCORE;
        score -= (self.black_bishops.count_ones() as i32) * BISHOP_SCORE;
        score -= (self.black_rooks.count_ones() as i32) * ROOK_SCORE;
        score -= (self.black_queens.count_ones() as i32) * QUEEN_SCORE;
        score
    }
}
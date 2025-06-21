use chessie::*;

use crate::metrics;
use crate::metrics::SearchMetrics;

pub const PAWN_SCORE: i32 = 100;
pub const KNIGHT_SCORE: i32 = 300;
pub const BISHOP_SCORE: i32 = 320;
pub const ROOK_SCORE: i32 = 500;
pub const QUEEN_SCORE: i32 = 900;

pub fn pieces_score(piece_kind: PieceKind) -> i32 {
    match piece_kind {
        PieceKind::Pawn => PAWN_SCORE,
        PieceKind::Knight => KNIGHT_SCORE,
        PieceKind::Bishop => BISHOP_SCORE,
        PieceKind::Rook => ROOK_SCORE,
        PieceKind::Queen => QUEEN_SCORE,
        PieceKind::King => 0, // King is not scored in this evaluation
    }
}

pub fn eval(game: &Game) -> i32 {
    SearchMetrics::start_evaluation_timing();
    let score = game.score();
    SearchMetrics::stop_evaluation_timing();
    score
}

trait PiecesScore {
    fn score(&self) -> i32;
}

impl PiecesScore for Board {
    fn score(&self) -> i32 {
        let mut score: i32 = 0;
        score += i32::from(self.pawns(Color::White).population()) * PAWN_SCORE;
        score += i32::from(self.knights(Color::White).population()) * KNIGHT_SCORE;
        score += i32::from(self.bishops(Color::White).population()) * BISHOP_SCORE;
        score += i32::from(self.rooks(Color::White).population()) * ROOK_SCORE;
        score += i32::from(self.queens(Color::White).population()) * QUEEN_SCORE;
        score -= i32::from(self.pawns(Color::Black).population()) * PAWN_SCORE;
        score -= i32::from(self.knights(Color::Black).population()) * KNIGHT_SCORE;
        score -= i32::from(self.bishops(Color::Black).population()) * BISHOP_SCORE;
        score -= i32::from(self.rooks(Color::Black).population()) * ROOK_SCORE;
        score -= i32::from(self.queens(Color::Black).population()) * QUEEN_SCORE;
        score
    }
}
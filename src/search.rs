use crate::metrics::{SearchMetrics, TimingGuard};

use chessie::{Board, Color, Game, Move};

pub fn search_entry(game: &Game, depth: u32) -> Option<Move> {
    SearchMetrics::increment_normal_search_entries();
    // Initialize metrics if not already done

    let _guard = TimingGuard::new_search();

    let legal_moves = game.get_legal_moves();

    if legal_moves.is_empty() {
        return None; // No legal moves available
    }

    SearchMetrics::increment_positions_generated(legal_moves.len() as u64);

    let mut best_move = None;

    let maximize_score = game.position().side_to_move() == Color::White;

    let mut best_score = if maximize_score {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };

    let mut alpha = i32::MIN + 1;
    let mut beta = i32::MAX - 1;

    for move_ in legal_moves {
        let new_game = game.with_move_made(move_);

        let score = min_max_search(&new_game, depth, alpha, beta);

        if maximize_score && score > best_score {
            best_score = score;
            best_move = Some(move_);
            alpha = best_score.max(alpha);
        } else if !maximize_score && score < best_score {
            best_score = score;
            best_move = Some(move_);
            beta = best_score.min(beta);
        }
    }

    best_move
}

fn min_max_search(game: &Game, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
    SearchMetrics::increment_normal_search_entries();

    let maximize_score = game.position().side_to_move() == Color::White;

    let mut best_score = if maximize_score {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };

    let legal_moves = game.get_legal_moves();

    SearchMetrics::increment_positions_generated(legal_moves.len() as u64);

    match check_game_result(game) {
        GameResult::WhiteWins => return i32::MAX - 1,

        GameResult::BlackWins => return i32::MIN + 1,

        GameResult::Draw(_) => return 0,

        GameResult::Ongoing => {}
    }

    if depth == 0 {
        return q_search(game, alpha, beta);
    }

    for move_ in legal_moves {
        let new_game = game.with_move_made(move_);

        let score = min_max_search(&new_game, depth - 1, alpha, beta);

        if (maximize_score && score > best_score) || (!maximize_score && score < best_score) {
            best_score = score;

            if maximize_score {
                alpha = best_score.max(alpha);
            } else {
                beta = best_score.min(beta);
            }
            if beta <= alpha {
                break; // Beta cut-off
            }
        }
    }

    best_score
}

fn q_search(game: &Game, mut alpha:i32, mut beta:i32) -> i32 {
    SearchMetrics::increment_q_search_entries();

    let maximize_score = game.position().side_to_move() == Color::White;

    let mut best_score = if maximize_score {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };

    let legal_captures = game.into_iter().only_captures().collect::<Vec<_>>();

    SearchMetrics::increment_positions_generated(legal_captures.len() as u64);

    if legal_captures.is_empty() {
        return evaluate_board(game);
    }

    for move_ in legal_captures {
        let new_game = game.with_move_made(move_);

        let score = q_search(&new_game, alpha, beta);

        if (score > best_score && maximize_score) || (score < best_score && !maximize_score) {
            best_score = score;
            
            if maximize_score {
                alpha = best_score.max(alpha);
            } else {
                beta = best_score.min(beta);
            }
            if beta <= alpha {
                break; // Beta cut-off
            }
        }
    }

    best_score
}

pub fn evaluate_board(game: &Game) -> i32 {
    let _guard = TimingGuard::new_evaluation();

    // A simple evaluation function that counts material balance

    game.board().score()
}

pub trait PiecesScore {
    fn score(&self) -> i32;
}

impl PiecesScore for Board {
    fn score(&self) -> i32 {
        let mut score: i32 = 0;
        score += i32::from(self.pawns(Color::White).population()) * 100;
        score += i32::from(self.knights(Color::White).population()) * 300;
        score += i32::from(self.bishops(Color::White).population()) * 320;
        score += i32::from(self.rooks(Color::White).population()) * 500;
        score += i32::from(self.queens(Color::White).population()) * 900;
        score -= i32::from(self.pawns(Color::Black).population()) * 100;
        score -= i32::from(self.knights(Color::Black).population()) * 300;
        score -= i32::from(self.bishops(Color::Black).population()) * 320;
        score -= i32::from(self.rooks(Color::Black).population()) * 500;
        score -= i32::from(self.queens(Color::Black).population()) * 900;
        score
    }
}

#[derive(Debug, PartialEq)]
enum GameResult {
    Ongoing,
    WhiteWins,
    BlackWins,
    Draw(DrawReason),
}

#[derive(Debug, PartialEq)]
enum DrawReason {
    Stalemate,
    FiftyMoveRule,
    InsufficientMaterial,
    Repetition,
}

fn check_game_result(game: &Game) -> GameResult {
    // Check for draw conditions first
    if game.can_draw_by_fifty() {
        return GameResult::Draw(DrawReason::FiftyMoveRule);
    }

    if game.can_draw_by_insufficient_material() {
        return GameResult::Draw(DrawReason::InsufficientMaterial);
    }

    //TODO:
    let is_repetition = false;
    if is_repetition {
        return GameResult::Draw(DrawReason::Repetition);
    }

    // Check if there are any legal moves
    let legal_moves = game.get_legal_moves();

    if legal_moves.is_empty() {
        if game.is_in_check() {
            // Checkmate - the opponent wins
            match game.side_to_move() {
                Color::White => GameResult::BlackWins,
                Color::Black => GameResult::WhiteWins,
            }
        } else {
            // Stalemate
            GameResult::Draw(DrawReason::Stalemate)
        }
    } else {
        GameResult::Ongoing
    }
}

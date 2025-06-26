use crate::eval::{eval, pieces_score};
use crate::metrics::{SearchMetrics, TimingKind};

use chessie::{Color, Game, Move, MoveList};

pub fn search_entry(game: &Game, depth: u32) -> Option<Move> {
    SearchMetrics::increment_normal_search_entries();
    // Initialize metrics if not already done

    SearchMetrics::change_timing_kind(TimingKind::Search);
    let maximize_score = game.position().side_to_move() == Color::White;

    let legal_moves = sort_moves(game.get_legal_moves(), game.clone(), maximize_score);

    if legal_moves.is_empty() {
        return None; // No legal moves available
    }

    SearchMetrics::increment_positions_generated(legal_moves.len() as u64);

    let mut best_move = None;

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
    SearchMetrics::change_timing_kind(TimingKind::Search);

    SearchMetrics::increment_normal_search_entries();

    let maximize_score = game.position().side_to_move() == Color::White;

    let mut best_score = if maximize_score {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };

    SearchMetrics::change_timing_kind(TimingKind::MoveGen);

    let legal_moves_unordered = game.get_legal_moves();

    match check_game_result(game, legal_moves_unordered.len()) {
        GameResult::WhiteWins => return i32::MAX - 1,

        GameResult::BlackWins => return i32::MIN + 1,

        GameResult::Draw(_) => return 0,

        GameResult::Ongoing => {}
    }

    SearchMetrics::change_timing_kind(TimingKind::MoveOrdering);

    let legal_moves = sort_moves(legal_moves_unordered, game.clone(), maximize_score);

    SearchMetrics::increment_positions_generated(legal_moves.len() as u64);

    SearchMetrics::change_timing_kind(TimingKind::Search);

    if depth == 0 {
        SearchMetrics::change_timing_kind(TimingKind::QSearch);
        let q_search_score = q_search(game, alpha, beta);
        SearchMetrics::change_timing_kind(TimingKind::Search);
        return q_search_score;
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
                SearchMetrics::increment_alpha_beta_cutoffs();
                break; // Beta cut-off
            }
        }
    }

    best_score
}

fn q_search(game: &Game, mut alpha: i32, mut beta: i32) -> i32 {
    SearchMetrics::increment_q_search_entries();

    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    //TODO: make function on board
    let maximize_score = game.position().side_to_move() == Color::White;

    let mut best_score = if maximize_score {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };

    SearchMetrics::change_timing_kind(TimingKind::MoveGen);

    //TODO: better stalemate detection
    let legal_moves: MoveList = game.get_legal_moves();

    SearchMetrics::increment_positions_generated(legal_moves.len() as u64);

    match check_game_result(game, legal_moves.len()) {
        GameResult::WhiteWins => return i32::MAX - 1,

        GameResult::BlackWins => return i32::MIN + 1,

        GameResult::Draw(_) => return 0,

        GameResult::Ongoing => {}
    }

    let legal_captures_unordered: MoveList = game.into_iter().only_captures().collect();

    SearchMetrics::change_timing_kind(TimingKind::MoveOrdering);

    let legal_captures = sort_moves(legal_captures_unordered, game.clone(), maximize_score);

    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    if legal_captures.is_empty() {
        SearchMetrics::change_timing_kind(TimingKind::Evaluation);
        let evaluation = eval(game);
        SearchMetrics::change_timing_kind(TimingKind::QSearch);
        return evaluation;
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
                SearchMetrics::increment_alpha_beta_cutoffs();
                break; // Beta cut-off
            }
        }
    }

    best_score
}

fn sort_moves(moves: MoveList, game: Game, is_maximizing: bool) -> MoveList {
    // Sort moves based on some heuristic, e.g., captures first, then checks, etc.
    let mut sorted_moves = moves;
    sorted_moves.sort_by_key(|m| {
        let from = game
            .kind_at(m.from())
            .map_or_else(|| 0, |piece| pieces_score(piece));
        let to = game
            .kind_at(m.to())
            .map_or_else(|| 0, |piece| pieces_score(piece));
        if is_maximizing { to - from } else { from - to }
    });
    sorted_moves
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

fn check_game_result(game: &Game, num_legal_moves: usize) -> GameResult {
    if num_legal_moves == 0 {
        return if game.is_in_check() {
            // Checkmate - the opponent wins
            match game.side_to_move() {
                Color::White => GameResult::BlackWins,
                Color::Black => GameResult::WhiteWins,
            }
        } else {
            // Stalemate
            GameResult::Draw(DrawReason::Stalemate)
        };
    }

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

    GameResult::Ongoing
}

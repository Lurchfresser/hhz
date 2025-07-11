use crate::board::Board;
use crate::eval::{eval, pieces_score};
use crate::metrics::TimingKind::Search;
use crate::metrics::{SearchMetrics, TimingKind};
use crate::moves::{Move, MoveList};

pub fn search_entry(board: &Board, depth: u8) -> Option<Move> {
    SearchMetrics::increment_normal_search_entries();
    // Initialize metrics if not already done

    SearchMetrics::change_timing_kind(TimingKind::Search);
    let maximize_score = board.white_to_move;

    let legal_moves = sort_moves(board.generate_legal_moves_temp(), board, maximize_score);

    if legal_moves.is_empty() {
        return None; // No legal moves available
    }

    SearchMetrics::increment_normal_search_positions_generated(legal_moves.len() as u64);

    let mut best_move = None;

    let mut best_score = if maximize_score {
        //TODO: define const values, with better ranges
        i16::MIN + 1
    } else {
        //TODO: define const values, with better ranges
        i16::MAX - 1
    };

    //TODO: define const values, with better ranges
    let mut alpha = i16::MIN + 1;
    //TODO: define const values, with better ranges
    let mut beta = i16::MAX - 1;

    for move_ in legal_moves {
        let new_game = board.make_move_temp(move_);

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

fn min_max_search(board: &Board, depth: u8, mut alpha: i16, mut beta: i16) -> i16 {
    if depth == 0 {
        SearchMetrics::change_timing_kind(TimingKind::QSearch);
        let q_search_score = q_search(board, alpha, beta);
        SearchMetrics::change_timing_kind(TimingKind::Search);
        return q_search_score;
    }

    SearchMetrics::change_timing_kind(TimingKind::Search);

    SearchMetrics::increment_normal_search_entries();

    let maximize_score = board.white_to_move;

    let mut best_score = if maximize_score {
        //TDO: define const values
        i16::MIN + 1
    } else {
        //TDO: define const values
        i16::MAX - 1
    };

    SearchMetrics::change_timing_kind(TimingKind::NormalMoveGen);

    let legal_moves_unordered = board.generate_legal_moves_temp();

    match check_game_result::<false>(board, legal_moves_unordered.len()) {
        //TDO: define const values
        GameResult::WhiteWins => return i16::MAX - 2,

        //TDO: define const values
        GameResult::BlackWins => return i16::MIN + 2,

        GameResult::Draw(_) => return 0,

        GameResult::Ongoing => {}
    }

    SearchMetrics::change_timing_kind(TimingKind::NormalMoveOrdering);

    let legal_moves = sort_moves(legal_moves_unordered, board, maximize_score);

    SearchMetrics::increment_normal_search_positions_generated(legal_moves.len() as u64);

    SearchMetrics::change_timing_kind(TimingKind::Search);
    #[cfg(feature = "metrics")]
    let mut best_move_found_at_index: Option<usize> = None;

    let mut i = 0;
    while i < legal_moves.len() {
        let move_ = legal_moves[i];

        let new_game = board.make_move_temp(move_);

        let score = min_max_search(&new_game, depth - 1, alpha, beta);

        if (maximize_score && score > best_score) || (!maximize_score && score < best_score) {
            #[cfg(feature = "metrics")]
            {
                best_move_found_at_index = Some(i);
            }
            best_score = score;

            if maximize_score {
                alpha = best_score.max(alpha);
            } else {
                beta = best_score.min(beta);
            }
            if beta <= alpha {
                // The move that caused the cutoff is at index 'i'. We add its 1-based index.
                SearchMetrics::add_to_normal_search_sum_of_cutoff_indices((i + 1) as u64);
                SearchMetrics::increment_normal_search_cutoffs();
                break; // Beta cut-off
            }
        }
        i += 1;
    }

    #[cfg(feature = "metrics")]
    if let Some(final_best_index) = best_move_found_at_index {
        SearchMetrics::increment_normal_search_nodes_with_best_move();
        if final_best_index == 0 {
            // Note: This calls the standard function, not your maybe_increment version.
            SearchMetrics::increment_normal_search_best_move_first_count();
        }
    }

    best_score
}

fn q_search(board: &Board, mut alpha: i16, mut beta: i16) -> i16 {
    SearchMetrics::increment_q_search_entries();

    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    let maximize_score = board.white_to_move;

    SearchMetrics::change_timing_kind(TimingKind::Evaluation);
    let stand_pat = eval(board);
    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    let mut best_score = stand_pat;

    if maximize_score {
        alpha = best_score.max(alpha);
    } else {
        beta = best_score.min(beta);
    }
    if beta <= alpha {
        SearchMetrics::increment_stand_pat_cutoffs();
        return best_score;
    }

    SearchMetrics::change_timing_kind(TimingKind::QMoveGen);

    //TODO: better stalemate detection
    let legal_moves: MoveList = board.generate_legal_moves_temp();

    SearchMetrics::increment_q_search_positions_generated(legal_moves.len() as u64);

    match check_game_result::<false>(board, legal_moves.len()) {
        //TODO:
        GameResult::WhiteWins => return i16::MAX - 2,

        //TODO:
        GameResult::BlackWins => return i16::MIN + 2,

        GameResult::Draw(_) => return 0,

        GameResult::Ongoing => {}
    }

    let legal_captures_unordered: MoveList = legal_moves
        .iter()
        .filter(|m| m.is_capture())
        .copied()
        .collect();

    SearchMetrics::change_timing_kind(TimingKind::QMoveOrdering);

    let legal_captures = sort_moves(legal_captures_unordered, board, maximize_score);

    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    if legal_captures.is_empty() {
        return stand_pat;
    }

    #[cfg(feature = "metrics")]
    let mut best_move_found_at_index: Option<usize> = None;

    let mut i = 0;
    while i < legal_captures.len() {
        let move_ = legal_captures[i];

        let new_game = board.make_move_temp(move_);
        let score = q_search(&new_game, alpha, beta);

        let is_new_best =
            (maximize_score && score > best_score) || (!maximize_score && score < best_score);

        if is_new_best {
            best_score = score;
            // The current move's index is `i`.
            #[cfg(feature = "metrics")]
            {
                best_move_found_at_index = Some(i);
            }

            if maximize_score {
                alpha = best_score.max(alpha);
            } else {
                beta = best_score.min(beta);
            }

            if beta <= alpha {
                // The move that caused the cutoff is at index 'i'. We add its 1-based index.
                SearchMetrics::add_to_q_search_sum_of_cutoff_indices((i + 1) as u64);
                SearchMetrics::increment_q_search_cutoffs();
                break; // Beta cut-off
            }
        }

        i += 1;
    }

    #[cfg(feature = "metrics")]
    if let Some(final_best_index) = best_move_found_at_index {
        // --- FIX #1: Increment the correct denominator ---
        SearchMetrics::increment_q_search_nodes_with_best_move();

        // --- FIX #2: Only increment the numerator if the condition is met ---
        if final_best_index == 0 {
            SearchMetrics::increment_q_search_best_move_first_count();
        }
    }

    best_score
}

fn sort_moves(moves: MoveList, board: &Board, is_maximizing: bool) -> MoveList {
    // Sort moves based on some heuristic, e.g., captures first, then checks, etc.
    let mut sorted_moves = moves;
    sorted_moves.sort_by_key(|m| {
        let from = pieces_score(board.pieces[m.from()]);
        let to = pieces_score(board.pieces[m.to()]);
        if is_maximizing { to - from } else { from - to }
    });
    sorted_moves
}

#[derive(Debug, PartialEq)]
pub enum GameResult {
    Ongoing,
    WhiteWins,
    BlackWins,
    Draw(DrawReason),
}

#[derive(Debug, PartialEq)]
pub enum DrawReason {
    Stalemate,
    FiftyMoveRule,
    InsufficientMaterial,
    Repetition,
}

pub fn check_game_result<const DETECT_THREE_FOLD: bool>(
    board: &Board,
    num_legal_moves: usize,
) -> GameResult {
    if num_legal_moves == 0 {
        return if board.in_check_temp() {
            // Checkmate - the opponent wins
            match board.white_to_move {
                true => GameResult::BlackWins,
                false => GameResult::WhiteWins,
            }
        } else {
            // Stalemate
            GameResult::Draw(DrawReason::Stalemate)
        };
    }

    // Check for draw conditions first
    if board.halfmove_clock >= 100 {
        return GameResult::Draw(DrawReason::FiftyMoveRule);
    }

    if board.is_draw_by_insufficient_material() {
        return GameResult::Draw(DrawReason::InsufficientMaterial);
    }

    let mut i = i16::from(board.halfmove_clock) - 4;

    // Counter for how many times we've seen the current position in the history.
    let mut count = 0;

    while i >= 0 {
        // Compare the current hash with a historical hash.
        if board.zobrist_hash == board.repetition_lookup[i as usize] {
            // A match was found!

            if !DETECT_THREE_FOLD {
                // --- In Search Optimization ---
                // We only need to find one previous occurrence to score this
                // node as a draw. If we can reach a position for the 2nd time,
                // we assume we can force the 3rd.
                return GameResult::Draw(DrawReason::Repetition);
            } else {
                // --- Strict Rule Check ---
                // We are checking the actual game state. We need to find
                // two previous occurrences to confirm a threefold repetition.
                count += 1;
                if count >= 2 {
                    return GameResult::Draw(DrawReason::Repetition);
                }
            }
        }
        // A position can only repeat when it's the same side to move.
        // Stepping by 2 ensures this.
        i -= 2;
    }

    GameResult::Ongoing
}

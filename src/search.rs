use crate::board::Board;
use crate::eval::{eval, pieces_score};
use crate::metrics::{SearchMetrics, TimingKind};
use crate::moves::{Move, MoveList};
use crate::tt_table::{NodeType, TT_Table};
use std::option::Option;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const MIN_SCORE: i16 = i16::MIN + 2;
const MAX_SCORE: i16 = i16::MAX - 1;

const WHITE_WINS: i16 = MAX_SCORE - 1;
const BLACK_WINS: i16 = MIN_SCORE + 1;

const SEARCH_CANCELED: i16 = i16::MIN;

pub fn search_entry(
    board: &Board,
    depth: u8,
    tt_table: &mut TT_Table,
    repetition_lookup: &mut [u64; 100],
    num_resetting_moves: u8,
    should_search: &Arc<AtomicBool>,
) -> Option<Move> {
    SearchMetrics::increment_normal_search_entries();
    // Initialize metrics if not already done

    SearchMetrics::change_timing_kind(TimingKind::Search);
    let maximize_score = board.white_to_move;

    let mut legal_moves = board.generate_legal_moves_temp();

    if legal_moves.is_empty() {
        return None; // No legal moves available
    }

    SearchMetrics::increment_normal_search_positions_generated(legal_moves.len() as u64);

    let mut best_move = None;

    let mut best_score = if maximize_score { MIN_SCORE } else { MAX_SCORE };

    let mut alpha = MIN_SCORE;
    let mut beta = MAX_SCORE;

    sort_moves(
        &mut legal_moves,
        board,
        maximize_score,
        tt_table,
        alpha,
        beta,
    );

    for _move in legal_moves {
        if should_search.load(Ordering::Relaxed) == false {
            return None;
        }
        let new_board = board.make_move_temp(&_move);
        let new_num_resetting_moves = if _move.resets_clock(board) {
            num_resetting_moves + 1
        } else {
            num_resetting_moves
        };
        let score = if _move.resets_clock(board) {
            min_max_search(
                &new_board,
                depth,
                alpha,
                beta,
                tt_table,
                &mut [board.zobrist_hash; 100],
                new_num_resetting_moves,
                should_search,
            )
        } else {
            repetition_lookup[(board.halfmove_clock + 1) as usize] = board.zobrist_after(&_move);
            min_max_search(
                &new_board,
                depth,
                alpha,
                beta,
                tt_table,
                repetition_lookup,
                new_num_resetting_moves,
                should_search,
            )
        };

        if score == SEARCH_CANCELED {
            return None;
        }

        if maximize_score && score > best_score {
            best_score = score;
            best_move = Some(_move);
            alpha = best_score.max(alpha);
        } else if !maximize_score && score < best_score {
            best_score = score;
            best_move = Some(_move);
            beta = best_score.min(beta);
        }
    }
    // we store depth + 1, because we pass it directly to minmax search
    // all Root Nodes are pv nodes, because a beta cutoff can never occur and alpha is always raised
    // see: https://www.chessprogramming.org/Node_Types#PV-Nodes
    tt_table.insert(
        board.zobrist_hash,
        best_score,
        depth + 1,
        NodeType::PvNode,
        best_move.unwrap(),
        board.halfmove_clock,
        num_resetting_moves,
    );
    best_move
}

fn min_max_search(
    board: &Board,
    depth: u8,
    mut alpha: i16,
    mut beta: i16,
    tt_table: &mut TT_Table,
    //TODO: look if board can also be used
    repetition_lookup: &mut [u64; 100],
    num_resetting_moves: u8,
    should_search: &Arc<AtomicBool>,
) -> i16 {
    if should_search.load(Ordering::Relaxed) == false {
        return SEARCH_CANCELED;
    }
    if depth == 0 {
        SearchMetrics::change_timing_kind(TimingKind::QSearch);
        let q_search_score = q_search(
            board,
            alpha,
            beta,
            tt_table,
            repetition_lookup,
            num_resetting_moves,
            should_search,
        );
        SearchMetrics::change_timing_kind(TimingKind::Search);
        return q_search_score;
    }

    SearchMetrics::change_timing_kind(TimingKind::Search);

    SearchMetrics::increment_normal_search_entries();

    let maximize_score = board.white_to_move;

    let mut best_score = if maximize_score {
        //TODO: define const values
        alpha
    } else {
        //TODO: define const values
        beta
    };

    SearchMetrics::change_timing_kind(TimingKind::NormalMoveGen);

    let mut legal_moves = board.generate_legal_moves_temp();

    match check_game_result::<false>(board, repetition_lookup, legal_moves.len()) {
        GameResult::WhiteWins => return WHITE_WINS,

        GameResult::BlackWins => return BLACK_WINS,

        GameResult::Draw(_) => return 0,

        GameResult::Ongoing => {}
    }

    SearchMetrics::increment_normal_search_tt_probes();
    if let Some(tt_hit) = tt_table.probe(board.zobrist_hash) {
        SearchMetrics::increment_normal_search_tt_hits();
        //TODO: increment in move ordering
        // SearchMetrics::increment_normal_search_tt_hits();
        if tt_hit.depth() >= depth {
            let tt_score = tt_hit.eval();
            match tt_hit.node_type() {
                NodeType::PvNode => {
                    SearchMetrics::increment_normal_search_tt_cutoffs();
                    return tt_score;
                }
                NodeType::CutNode => {
                    if (maximize_score && tt_score >= beta)
                        || (!maximize_score && tt_score <= alpha)
                    {
                        SearchMetrics::increment_normal_search_tt_cutoffs();
                        return tt_score;
                    }
                }
                NodeType::AllNode => {
                    if (maximize_score && tt_score < alpha) || (!maximize_score && tt_score > beta)
                    {
                        SearchMetrics::increment_normal_search_tt_cutoffs();
                        return tt_score;
                    }
                }
            }
        }
    }

    SearchMetrics::change_timing_kind(TimingKind::NormalMoveOrdering);

    sort_moves(
        &mut legal_moves,
        board,
        maximize_score,
        tt_table,
        alpha,
        beta,
    );

    SearchMetrics::increment_normal_search_positions_generated(legal_moves.len() as u64);

    SearchMetrics::change_timing_kind(TimingKind::Search);
    #[cfg(feature = "metrics")]
    let mut best_move_found_at_index: Option<usize> = None;
    // all node = all nodes searched
    let mut node_type = NodeType::AllNode;
    let mut best_move: Move = Move::null_move();

    let mut i = 0;
    while i < legal_moves.len() {
        let _move = legal_moves[i];
        let new_game = board.make_move_temp(&_move);
        let new_num_resetting_moves = if _move.resets_clock(board) {
            num_resetting_moves + 1
        } else {
            num_resetting_moves
        };
        let score = if _move.resets_clock(board) {
            min_max_search(
                &new_game,
                depth - 1,
                alpha,
                beta,
                tt_table,
                &mut [board.zobrist_hash; 100],
                new_num_resetting_moves,
                should_search,
            )
        } else {
            repetition_lookup[(board.halfmove_clock + 1) as usize] = board.zobrist_after(&_move);
            min_max_search(
                &new_game,
                depth - 1,
                alpha,
                beta,
                tt_table,
                repetition_lookup,
                new_num_resetting_moves,
                should_search,
            )
        };
        if score == SEARCH_CANCELED {
            return SEARCH_CANCELED;
        }

        if (maximize_score && score > best_score) || (!maximize_score && score < best_score) {
            #[cfg(feature = "metrics")]
            {
                best_move_found_at_index = Some(i);
            }
            best_move = _move;
            best_score = score;
            node_type = NodeType::PvNode;

            if maximize_score {
                alpha = best_score.max(alpha);
            } else {
                beta = best_score.min(beta);
            }
            if beta <= alpha {
                node_type = NodeType::CutNode;
                best_move = _move;
                // The move that caused the cutoff is at index 'i'. We add its 1-based index.
                SearchMetrics::add_to_normal_search_sum_of_cutoff_indices((i + 1) as u64);
                SearchMetrics::increment_normal_search_cutoffs();
                break; // Beta cut-off
            }
        }
        i += 1;
    }

    tt_table.insert(
        board.zobrist_hash,
        best_score,
        depth,
        node_type,
        best_move,
        board.halfmove_clock,
        num_resetting_moves,
    );

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

fn q_search(
    board: &Board,
    mut alpha: i16,
    mut beta: i16,
    tt_table: &mut TT_Table,
    repetition_lookup: &mut [u64; 100],
    num_resetting_moves: u8,
    should_search: &Arc<AtomicBool>,
) -> i16 {
    if should_search.load(Ordering::Relaxed) == false {
        return SEARCH_CANCELED;
    }
    SearchMetrics::increment_q_search_entries();

    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    let maximize_score = board.white_to_move;

    SearchMetrics::change_timing_kind(TimingKind::Evaluation);
    let stand_pat = eval(board);
    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    //TODO: not in check
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

    match check_game_result::<false>(board, repetition_lookup, legal_moves.len()) {
        //TODO:
        GameResult::WhiteWins => return WHITE_WINS,

        //TODO:
        GameResult::BlackWins => return BLACK_WINS,

        GameResult::Draw(_) => return 0,

        GameResult::Ongoing => {}
    }

    SearchMetrics::increment_q_search_tt_probes();
    if let Some(tt_hit) = tt_table.probe(board.zobrist_hash) {
        //TODO: increment in move ordering
        // SearchMetrics::increment_normal_search_tt_hits();
        SearchMetrics::increment_q_search_tt_hits();
        //
        let tt_score = tt_hit.eval();
        match tt_hit.node_type() {
            NodeType::PvNode => {
                SearchMetrics::increment_q_search_tt_cutoffs();
                return tt_score;
            }
            NodeType::CutNode => {
                if (maximize_score && tt_score >= beta) || (!maximize_score && tt_score <= alpha) {
                    SearchMetrics::increment_q_search_tt_cutoffs();
                    return tt_score;
                }
            }
            NodeType::AllNode => {
                if (maximize_score && tt_score < alpha) || (!maximize_score && tt_score > beta) {
                    SearchMetrics::increment_q_search_tt_cutoffs();
                    return tt_score;
                }
            }
        }
    }

    let mut legal_captures: MoveList = legal_moves
        .iter()
        .filter(|m| m.is_capture())
        .copied()
        .collect();

    SearchMetrics::change_timing_kind(TimingKind::QMoveOrdering);

    sort_moves(
        &mut legal_captures,
        board,
        maximize_score,
        tt_table,
        alpha,
        beta,
    );

    SearchMetrics::change_timing_kind(TimingKind::QSearch);

    if legal_captures.is_empty() {
        return stand_pat;
    }

    #[cfg(feature = "metrics")]
    let mut best_move_found_at_index: Option<usize> = None;
    let mut node_type = NodeType::AllNode;
    let mut best_move: Move = Move::null_move();
    let mut i = 0;
    while i < legal_captures.len() {
        let _move = legal_captures[i];
        let new_bard = board.make_move_temp(&_move);
        let new_num_resetting_moves = if _move.resets_clock(board) {
            num_resetting_moves + 1
        } else {
            num_resetting_moves
        };
        let score = if _move.resets_clock(board) {
            q_search(
                &new_bard,
                alpha,
                beta,
                tt_table,
                &mut [board.zobrist_hash; 100],
                new_num_resetting_moves,
                should_search,
            )
        } else {
            repetition_lookup[(board.halfmove_clock + 1) as usize] = board.zobrist_after(&_move);
            q_search(
                &new_bard,
                alpha,
                beta,
                tt_table,
                repetition_lookup,
                new_num_resetting_moves,
                should_search,
            )
        };

        if score == SEARCH_CANCELED {
            return SEARCH_CANCELED;
        }

        let is_new_best =
            (maximize_score && score > best_score) || (!maximize_score && score < best_score);

        if is_new_best {
            node_type = NodeType::PvNode;
            best_score = score;
            best_move = _move;
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
                node_type = NodeType::CutNode;
                best_move = _move;
                // The move that caused the cutoff is at index 'i'. We add its 1-based index.
                SearchMetrics::add_to_q_search_sum_of_cutoff_indices((i + 1) as u64);
                SearchMetrics::increment_q_search_cutoffs();
                break; // Beta cut-off
            }
        }

        i += 1;
    }

    tt_table.insert(
        board.zobrist_hash,
        best_score,
        0,
        node_type,
        best_move,
        board.halfmove_clock,
        num_resetting_moves,
    );

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

fn sort_moves(
    moves: &mut MoveList,
    board: &Board,
    is_maximizing: bool,
    tt_table: &TT_Table,
    alpha: i16,
    beta: i16,
) {
    let hash_move_option = tt_table.probe(board.zobrist_hash).map(|h| h.best_move());

    moves.sort_by_cached_key(|m| {
        const HASH_MOVE_SCORE: i32 = 8_000_000;
        //lok at graph http://www.netlib.org/utk/lsi/pcwLSI/text/node351.html
        // pv node, most left, then cut nodes should be preffered, all nodes hould be searched last
        //https://www.chessprogramming.org/Node_Types
        const PV_MOVE_SCORE: i32 = 3_000_000;
        const CUT_MOVE_SCORE: i32 = 1_500_000;
        const ALL_MOVE_SCORE: i32 = -1_000_000;
        const CAPTURE_BASE_SCORE: i32 = 500_000;

        if let Some(Some(hash_move)) = hash_move_option {
            if *m == hash_move {
                return -HASH_MOVE_SCORE;
            }
        }

        let mut score = 0;

        // --- HIERARCHY LEVEL 1: PV MOVE ---
        if let Some(tt_hit) = tt_table.probe(board.zobrist_after(m)) {
            SearchMetrics::increment_pv_nodes_found_in_move_ordering();
            let tt_type = tt_hit.node_type();
            let tt_score = tt_hit.eval();
            if tt_type == NodeType::PvNode {
                // DO NOT return early. Assign the score.
                score = PV_MOVE_SCORE;
            } else if tt_type == NodeType::CutNode && (is_maximizing && tt_hit.eval() >= beta)
                || (!is_maximizing && tt_hit.eval() <= alpha)
            {
                score += CUT_MOVE_SCORE;
            } else if tt_type == NodeType::CutNode && (is_maximizing && tt_score < alpha)
                || (!is_maximizing && tt_score > beta)
            {
                score += ALL_MOVE_SCORE;
            }
        }
        // --- HIERARCHY LEVEL 2: CAPTURES ---
        if score == 0 && m.is_capture() {
            // Only check if not already a PV move
            let victim_value = pieces_score(board.pieces[m.to()]).abs() as i32;
            let attacker_value = pieces_score(board.pieces[m.from()]).abs() as i32;

            score = CAPTURE_BASE_SCORE + (victim_value * 10 - attacker_value);
        }
        -score
    });
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
    //TODO: measure, if using zobrist is much faster
    repetition_lookup: &[u64; 100],
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
        if board.zobrist_hash == repetition_lookup[i as usize] {
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

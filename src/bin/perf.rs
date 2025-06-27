use hhz::board::Board;
use hhz::const_move_gen::*;
use std::collections::HashSet;

fn main() {
    // let fen = "8/8/8/8/3K1N1R/8/8/8 w - - 0 1";
    // let fen = "3r4/5N2/8/8/1B5B/8/1R5q/3K4 w - - 0 1";
    let fen = "r3k2r/8/1r4N1/8/8/8/7P/R3K2R w KQkq - 0 1";

    recursive_check(1, fen)
}

fn recursive_check(depth: i32, fen: &str) {
    if depth <= 0 {
        return;
    };

    let board = Board::from_fen(fen);
    let mut chessie_board = chessie::Game::from_fen(fen).unwrap();

    let moves = board.generate_legal_moves_temp();
    let my_move_count = moves.len();

    let chessie_moves = chessie_board.get_legal_moves();
    let chessie_move_count = chessie_moves.len();

    let mut my_move_set: HashSet<String> = HashSet::new();
    for _move in moves {
        if my_move_set.contains(&_move.to_algebraic()) {
            panic!("Double move generated. Move: {}", _move);
        }
        my_move_set.insert(_move.to_algebraic());
        // println!("my: {}", _move.to_algebraic());
    }

    let mut chessie_move_set: HashSet<String> = HashSet::new();
    for _move in chessie_moves {
        let mut move_not_found = false;
        let uci = _move.to_uci();
        chessie_move_set.insert(uci.clone());
        // println!("chessie: {}", uci);
        if !my_move_set.contains(&uci) && !move_not_found {
            println!("Did not find move {}", _move.to_uci());
            move_not_found = true;
        } else {
            let fen = chessie_board.with_move_made(_move).to_fen();
            recursive_check(depth - 1, &fen);
        }
    }

    for _move in my_move_set {
        if !chessie_move_set.contains(&_move) {
            println!("I found and illigal move: {}", _move);
        }
    }

    if (chessie_move_count != my_move_count) {
        panic!(
            "chessie found {} moves, but I found {}, at fen {}",
            chessie_move_count, my_move_count, fen
        );
    }
}

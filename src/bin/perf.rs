use hhz::board::Board;
use hhz::const_move_gen::*;
use std::collections::HashSet;

fn main() {
    // let fen = "8/8/8/8/3K1N1R/8/8/8 w - - 0 1";
    let fen = "3r4/5N2/8/8/1B5B/8/1R5q/3K4 w - - 0 1";
    let board = Board::from_fen(fen);
    let chessie_board = chessie::Game::from_fen(fen).unwrap();

    let moves = board.generate_legal_moves_temp();
    let chessie_moves = chessie_board.get_legal_moves();

    let mut my_move_set: HashSet<String> = HashSet::new();
    for _move in moves {
        my_move_set.insert(_move.to_algebraic());
    }

    for _move in chessie_moves {
        if !my_move_set.contains(&_move.to_uci()) {
            println!("Did not find move {}", _move.to_uci());
        }
    }

    println!("hi");
}

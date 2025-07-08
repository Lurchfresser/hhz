use hhz::board::Board;
use hhz::moves::Move;
use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    let file_content =
        fs::read_to_string("assets/test-fens.txt").expect("Should have been able to read the file");

    let fens: Vec<&str> = file_content.split("\n").collect();

    for fen in fens {
        let chessie_board = chessie::Game::from_fen(fen)
            .expect(&("chessie fen paniced. Fen: ".to_owned() + fen + "Error: "));
        let my_board =
            Board::from_fen(fen).expect(&("my fen paniced. Fen: ".to_owned() + fen + "Error: "));
        recursive_check(3, chessie_board, my_board);
    }

    // let fen = "1r2k2r/1b4bq/8/8/8/8/8/R3K1BR w KQk - 2 2";
    // let chessie_board = chessie::Game::from_fen(fen).unwrap();
    // let my_board = Board::from_fen(fen);
    // recursive_check(1, chessie_board, my_board);
}

fn recursive_check(depth: i32, chessie_board: chessie::Game, my_board: Board) {
    if depth <= 0 {
        return;
    };

    let moves = my_board.generate_legal_moves_temp();
    let my_move_count = moves.len();

    let chessie_moves = chessie_board.get_legal_moves();
    let chessie_move_count = chessie_moves.len();

    let mut my_move_set: HashMap<String, Move> = HashMap::new();
    for _move in moves {
        if my_move_set.get(&_move.to_uci()).is_some() {
            panic!("Double move generated. Move: {}", _move);
        }
        my_move_set.insert(_move.to_uci(), _move);
        // println!("my: {}", _move.to_algebraic());
    }

    let mut chessie_move_set: HashSet<String> = HashSet::new();
    for _move in chessie_moves {
        let mut move_not_found = false;
        let uci = _move.to_uci();
        chessie_move_set.insert(uci.clone());
        if my_move_set.get(&uci).is_none() && !move_not_found {
            println!("Did not find move {}", _move.to_uci());
            move_not_found = true;
        } else {
            let current_fen = chessie_board.to_fen();
            let new_chessie_board = chessie_board.with_move_made(_move);
            let my_move = my_move_set.get(&_move.to_uci()).unwrap();
            let chessie_uci = _move.to_uci();
            let my_uci = my_move.to_uci();
            let my_new_board = my_board.make_move_temp(*my_move);
            // assert_eq!(new_chessie_board.to_fen(), my_new_board.to_fen());
            recursive_check(depth - 1, new_chessie_board, my_new_board);
        }
    }

    for _move in my_move_set.values() {
        if !chessie_move_set.contains(&*_move.to_uci()) {
            println!("I found and illigal move: {}", _move);
        }
    }

    if (chessie_move_count != my_move_count) {
        panic!(
            "chessie found {} moves, but I found {}, at fen {}",
            chessie_move_count,
            my_move_count,
            chessie_board.to_fen()
        );
    }
}

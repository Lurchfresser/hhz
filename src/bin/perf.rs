use hhz::board::Board;
use hhz::const_move_gen::*;

fn main() {
    let board = Board::from_fen("4r3/8/8/8/8/3p2b1/4PP2/4K3 w - - 0 1");

    let moves = board.generate_legal_moves_temp();

    for _move in moves {
        println!("move: {}", _move)
    }

    println!("hi");
}

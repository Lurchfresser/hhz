use hhz::board::Board;
use hhz::bit_boards::*;

fn main() {
    let board = Board::from_fen("8/8/8/8/8/rr6/kRR5/R1R5 b - - 0 1");

    board.generate_king_moves();
}
use hhz::board::Board;
use hhz::bit_boards::*;

fn main() {
    let board = Board::from_fen("8/8/8/2r5/1r6/3N4/1R6/2R5 w - - 0 1");

    board.generate_knight_moves();
}
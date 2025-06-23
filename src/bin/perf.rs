use hhz::board::Board;
use hhz::bit_boards::*;

fn main() {
    let board = Board::from_fen("7r/3p3p/2Pr2P1/8/8/8/8/8 b - - 0 1");

    board.generate_pawn_moves();
}
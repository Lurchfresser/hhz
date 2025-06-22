use hhz::board::Board;
use hhz::bit_boards::*;

fn main() {
    let board = Board::from_fen("b6R/1b6/8/8/8/8/1R3N2/7N w - - 0 1");

    let test = gen_king_moves();

    board.generate_rook_moves();
}
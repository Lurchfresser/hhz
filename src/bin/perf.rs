use hhz::board::Board;
use hhz::const_move_gen::{
    gen_free_rook_mask_edges_removed, gen_free_rook_moves, gen_rook_square_to_square_ray,
};

fn main() {
    let board = Board::from_fen("1q6/1P6/4r3/8/1K2r1r1/1P6/1R6/1r2r3 w - - 0 1");

    board.generate_pins();
    
    println!("hi");
}

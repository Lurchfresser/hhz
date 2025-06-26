use hhz::board::Board;
use hhz::const_move_gen::*;

fn main() {
    let board = Board::from_fen("b7/4r2q/2P3P1/8/3rKP1q/3P4/6p1/1b5b w - - 0 1");

    board.generate_pins();
    // let test = gen_free_bishop_moves();
    
    println!("hi");
}

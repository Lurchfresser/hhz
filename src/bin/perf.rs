use hhz::board::Board;

fn main() {
    let board = Board::from_fen("7B/1q3p2/8/3B4/8/8/P7/B6p w - - 0 1");

    board.generate_bishop_moves();


    println!("hi");
}
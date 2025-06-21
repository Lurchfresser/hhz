use hhz::board::Board;

fn main() {
    let fen_strings = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppp1p/8/6pP/8/8/PPPPPPP1/RNBQKBNR w KQkq g6 0 1"
    ];

    for fen in fen_strings {
        let board = Board::from_fen(fen);
        println!("FEN: {}", fen);
        // Here you would call the function to parse the FEN string
        // For example: let board = Board::from_fen(fen);
        // println!("{:?}", board);
    }
}
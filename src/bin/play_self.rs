use hhz::board::Board;
use hhz::search::search_entry;

pub fn main() {
    let mut board = Board::from_fen("1R3b1r/p1R3pp/1k2N3/3p1p2/n7/4P3/P1P1BPPP/6K1 b - - 11 30").unwrap();
    loop {
        let current_millisecond = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let next_move = search_entry(&board, 3).expect(&("Failed to search move, at position ".to_owned() + &board.to_fen().to_string()));
        let passed_millisecond = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - current_millisecond;
        println!(
            "Next move: {} found in {} ms",
            next_move, passed_millisecond
        );
        board = board.make_move_temp(next_move);
    }
}

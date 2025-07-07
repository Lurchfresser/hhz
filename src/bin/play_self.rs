use hhz::board::Board;
use hhz::search::search_entry;

pub fn main() {
    let mut board = Board::default();
    loop {
        let current_millisecond = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let next_move = search_entry(&board, 3).expect("Failed to search move");
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

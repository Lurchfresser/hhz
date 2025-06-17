use chessie::prelude::*;
use hhz::search::search_entry;

pub fn main(){
    let mut game = Game::default();
    loop {
        let current_millisecond = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let next_move = search_entry(&game, 3).expect("Failed to search move");
        let passed_millisecond = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() - current_millisecond;
        println!("Next move: {} found in {} ms", next_move, passed_millisecond);
        game.make_move(next_move);
    }
}
use chessie::prelude::*;

fn main() {
    let mut game = Game::default();

    let mut is_human_playing = true;

    loop {
        let legal_moves = game.get_legal_moves();
        if is_human_playing {
            for move_ in legal_moves {
                println!("Legal move: {}", move_);
            }
            println!("Enter your move (in UCI format):");
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let input = input.trim();

            if let Ok(move_) = Move::from_uci(game.position(), input) {
                is_human_playing = false; // Switch to AI after human move
                game.make_move(move_);
            } else {
                println!("Invalid move format, try again.");
            }
        } else {
            // AI's turn using a simple search algorithm
            if let Some(best_move) = simple_search(&game, 3) {
                println!("AI plays: {}", best_move);
                game.make_move(best_move);
                is_human_playing = true; // Switch back to human after AI move
            } else {
                println!("No legal moves available for AI.");
            }
        }
    }
}

fn simple_search(game: &Game, depth: u32) -> Option<Move> {
    if depth == 0 {
        return None; // Base case, no more depth to search
    }

    let legal_moves = game.get_legal_moves();
    if legal_moves.is_empty() {
        return None; // No legal moves available
    }

    // For simplicity, just return the first legal move
    Some(legal_moves[0])
}

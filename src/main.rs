use std::str::FromStr;

use chessie::prelude::*;
use dotenv::dotenv;
use futures_util::StreamExt;
use licheszter::{client::Licheszter, models::board::BoardState};

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load environment variables from .env file
    // Create a new Licheszter with your account token
    let client = Licheszter::builder()
        .with_authentication(dotenv::var("LICHESS_API_TOKEN").expect("LICHESS_API_TOKEN not set"))
        .build();

    // ...or open the event stream
    let mut events = client.connect().await.unwrap();
    while let Ok(event) = events.next().await.unwrap() {
        println!("Event: {:?}", event);
        match event {
            licheszter::models::board::Event::GameStart { game } => {
                let result = client.bot_game_connect(&game.id).await;
                let mut chessie_game = Game::from_fen(&game.fen).unwrap();
                match result {
                    Ok(mut stream) => {
                        println!("Connected to game stream: {}", game.id);
                        // Here you can handle the game stream, e.g., read moves and respond
                        while let Some(Ok(stream_event)) = stream.next().await {
                            println!("Stream event: {:?}", stream_event);
                            match stream_event {
                                BoardState::GameFull(game_full) => {
                                    if (game.is_my_turn) {
                                        let depth = 3; // Set the search depth
                                        if let Some(best_move) = simple_search(&chessie_game, depth)
                                        {
                                            println!("Best move: {}", best_move);
                                            let best_move_uci = best_move.to_uci();
                                            // Send the best move back to the game
                                            let response = client.bot_play_move(
                                                &game.id,
                                                &best_move_uci,
                                                false,
                                            );
                                            match response.await {
                                                Ok(_) => println!("Move sent: {}", best_move),
                                                Err(e) => eprintln!("Failed to send move: {}", e),
                                            }
                                        } else {
                                            println!("No legal moves available.");
                                        }
                                    }
                                }
                                BoardState::GameState(game_state) => {
                                    let chessie_result =
                                        chessie_game.make_move_uci(&game_state.moves.split(" ").last().unwrap());
                                    if let Err(e) = chessie_result {
                                        eprintln!("Failed to make move: {}", e);
                                        panic!();
                                    }
                                    let depth = 3; // Set the search depth
                                    if let Some(best_move) = simple_search(&chessie_game, depth) {
                                        println!("Best move: {}", best_move);
                                        let best_move_uci = best_move.to_uci();
                                        // Send the best move back to the game
                                        let response =
                                            client.bot_play_move(&game.id, &best_move_uci, false);
                                        match response.await {
                                            Ok(_) => println!("Move sent: {}", best_move),
                                            Err(e) => eprintln!("Failed to send move: {}", e),
                                        }
                                    } else {
                                        println!("No legal moves available.");
                                    }
                                }
                                BoardState::ChatLine(chat_line) => todo!(),
                                BoardState::OpponentGone(opponent_gone) => todo!(),
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to connect to game stream: {}", e),
                }
            }
            licheszter::models::board::Event::GameFinish { game } => {
                // Handle game finish event, e.g., log the result or clean up resources
                println!("Game finished: {}", game.id);
            },
            licheszter::models::board::Event::Challenge { challenge } => {
                // Here you can handle the challenge event, e.g., accept or decline it
                // For example, you could automatically accept challenges:
                let response = client.challenge_accept(&challenge.id);
                match response.await {
                    Ok(_) => println!("Challenge accepted: {}", challenge.id),
                    Err(e) => eprintln!("Failed to accept challenge: {}", e),
                }
            }
            licheszter::models::board::Event::ChallengeCanceled { challenge } => todo!(),
            licheszter::models::board::Event::ChallengeDeclined { challenge } => todo!(),
        }
    }
}

fn simple_search(game: &Game, depth: u32) -> Option<Move> {
    let legal_moves = game.get_legal_moves();
    if legal_moves.is_empty() {
        return None; // No legal moves available
    }
    return Some(*legal_moves.first().unwrap());
    // let mut best_move = None;
    // let mut best_score = i32::MIN + 1;
    // for move_ in legal_moves {
    //     let new_game = game.with_move_made(move_);
    //     let score = alpha_beta_search(&new_game, depth - 1, i32::MIN + 1, i32::MAX - 1);
    //     if score > best_score {
    //         best_score = score;
    //         best_move = Some(move_);
    //     }
    // }
    // best_move
}

fn alpha_beta_search(game: &Game, depth: u32, alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate_board(game); // Evaluate the board position
    }

    let mut alpha = alpha;

    let legal_moves = game.get_legal_moves();

    if legal_moves.is_empty() {
        if game.is_in_check() {
            return i32::MIN + 1; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }

    for move_ in legal_moves {
        let new_game = game.with_move_made(move_);
        let score = -alpha_beta_search(&new_game, depth - 1, -beta, -alpha);

        if score >= beta {
            return beta; // Beta cut-off
        }
        if score > alpha {
            alpha = score; // Update alpha
        }
    }

    alpha
}

pub fn evaluate_board(game: &Game) -> i32 {
    // A simple evaluation function that counts material balance
    game.board().score()
}

pub trait PiecesScore {
    fn score(&self) -> i32;
}

impl PiecesScore for Board {
    fn score(&self) -> i32 {
        let mut score: i32 = 0;
        score += i32::from(self.pawns(Color::White).population()) * 100;
        score += i32::from(self.knights(Color::White).population()) * 300;
        score += i32::from(self.bishops(Color::White).population()) * 320;
        score += i32::from(self.rooks(Color::White).population()) * 500;
        score += i32::from(self.queens(Color::White).population()) * 900;
        score -= i32::from(self.pawns(Color::Black).population()) * 100;
        score -= i32::from(self.knights(Color::Black).population()) * 300;
        score -= i32::from(self.bishops(Color::Black).population()) * 320;
        score -= i32::from(self.rooks(Color::Black).population()) * 500;
        score -= i32::from(self.queens(Color::Black).population()) * 900;
        score
    }
}
